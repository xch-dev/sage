use crate::{CredentialStore, credential_now};
use sage_permissions::{ApprovalBinding, CallerId};
use serde::Serialize;
use specta::Type;
use tokio::sync::oneshot;

#[derive(Debug)]
pub struct RpcApprovalRequest {
    pub caller: CallerId,
    pub method: String,
    pub body: serde_json::Value,
    pub fingerprint: Option<u32>,
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OperationReview {
    pub id: String,
    pub credential_id: String,
    pub name: String,
    pub method: String,
    /// Exact RPC parameters being authorized, presented as formatted JSON.
    pub parameters: String,
    pub fingerprint: Option<u32>,
    pub network: String,
    pub expires_at: u64,
}

pub(crate) struct PendingOperation {
    pub review: OperationReview,
    pub caller: CallerId,
    pub binding: String,
    pub sender: oneshot::Sender<Result<ApprovalBinding, String>>,
}

impl CredentialStore {
    pub(crate) fn enqueue_operation(
        &self,
        request: RpcApprovalRequest,
    ) -> Result<oneshot::Receiver<Result<ApprovalBinding, String>>, String> {
        let credential_id = match &request.caller {
            CallerId::Agent(id) | CallerId::Service(id) => id.clone(),
            CallerId::App(_) => return Err("Apps use their own approval presenter".into()),
        };
        let mut state = self.state.lock().unwrap();
        let now = credential_now();
        state
            .operations
            .retain(|_, p| p.review.expires_at > now && !p.sender.is_closed());
        if state.operations.len() >= 8
            || state
                .operations
                .values()
                .filter(|p| p.review.credential_id == credential_id)
                .count()
                >= 2
        {
            return Err("Too many pending operation requests".into());
        }
        let credential = state
            .saved
            .credentials
            .iter()
            .find(|c| c.info.id == credential_id)
            .ok_or("Credential was revoked")?;
        let policy = sage_permissions::rpc_policy(&request.method).ok_or("Unknown RPC method")?;
        let cap = policy.capability.ok_or("Owner credentials required")?;
        if !policy.approval || !credential.info.grant.capabilities.contains(&cap) {
            return Err("Operation permission not granted".into());
        }
        let id = crate::credentials::random_hex(16);
        let review = OperationReview {
            id: id.clone(),
            credential_id,
            name: credential.info.grant.name.clone(),
            method: request.method.clone(),
            parameters: serde_json::to_string_pretty(&request.body).map_err(|e| e.to_string())?,
            fingerprint: request.fingerprint,
            network: request.network,
            expires_at: now + 300,
        };
        let binding = crate::session::request_binding(&request.method, &request.body)
            .map_err(|e| e.to_string())?;
        let (sender, receiver) = oneshot::channel();
        state.operations.insert(
            id,
            PendingOperation {
                review,
                caller: request.caller,
                binding,
                sender,
            },
        );
        Ok(receiver)
    }

    pub fn operations(&self) -> Vec<OperationReview> {
        let mut state = self.state.lock().unwrap();
        state
            .operations
            .retain(|_, p| p.review.expires_at > credential_now() && !p.sender.is_closed());
        let mut reviews: Vec<_> = state
            .operations
            .values()
            .map(|p| p.review.clone())
            .collect();
        reviews.sort_by_key(|p| (p.expires_at, p.id.clone()));
        reviews
    }

    pub fn decide_operation(&self, id: &str, approved: bool) -> anyhow::Result<()> {
        let mut state = self.state.lock().unwrap();
        let pending = state
            .operations
            .remove(id)
            .ok_or_else(|| anyhow::anyhow!("Request is no longer pending"))?;
        if pending.review.expires_at <= credential_now() || pending.sender.is_closed() {
            anyhow::bail!("Request expired or client disconnected");
        }
        let decision = if approved {
            Ok(ApprovalBinding::new(
                pending.caller,
                pending.binding,
                pending.review.fingerprint,
                pending.review.expires_at * 1000,
            ))
        } else {
            Err("User rejected the RPC operation".into())
        };
        pending
            .sender
            .send(decision)
            .map_err(|_| anyhow::anyhow!("Client disconnected"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AppState, ConnectionGrant, ConnectionKind, RpcSession};
    use sage_permissions::{WalletCapability as Cap, WalletScope};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    fn setup() -> (
        tempfile::TempDir,
        AppState,
        CredentialStore,
        axum::http::HeaderMap,
        String,
    ) {
        let root = tempfile::tempdir().unwrap();
        let mut sage = sage::Sage::new(root.path(), true);
        sage.config.global.fingerprint = Some(7);
        let store = CredentialStore::open(root.path()).unwrap();
        let issued = store
            .issue(ConnectionGrant {
                name: "Test agent".into(),
                kind: ConnectionKind::Agent,
                capabilities: [Cap::WalletUpdateContact, Cap::WalletGetVersion].into(),
                wallet_scope: WalletScope::SelectedWallets {
                    fingerprints: vec![7],
                },
            })
            .unwrap();
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            "authorization",
            format!("Bearer {}", issued.token).parse().unwrap(),
        );
        let app = AppState {
            sage: Arc::new(Mutex::new(sage)),
            authenticator: Some(Arc::new(store.clone())),
        };
        (root, app, store, headers, issued.credential.id)
    }
    async fn pending(store: &CredentialStore) -> OperationReview {
        for _ in 0..100 {
            if let Some(review) = store.operations().first() {
                return review.clone();
            }
            tokio::task::yield_now().await;
        }
        panic!("operation did not queue");
    }
    #[tokio::test]
    async fn operation_waits_without_lock_and_can_be_approved_only_once() {
        let (_root, app, store, headers, _) = setup();
        let caller = app.clone();
        let task = tokio::spawn(async move {
            caller
                .session(&headers, "update_contact", &serde_json::json!({"id": 1}))
                .await
                .map(|_: RpcSession<'_>| ())
        });
        let review = pending(&store).await;
        assert_eq!(review.method, "update_contact");
        assert_eq!(review.fingerprint, Some(7));
        assert!(app.sage.try_lock().is_ok(), "approval must not lock Sage");
        assert!(!task.is_finished());
        store.decide_operation(&review.id, true).unwrap();
        assert!(store.decide_operation(&review.id, true).is_err());
        assert!(task.await.unwrap().is_ok());
        assert!(store.operations().is_empty());
    }
    #[tokio::test]
    async fn denial_revocation_rotation_and_network_changes_cancel_execution() {
        for scenario in ["deny", "revoke", "rotate", "network"] {
            let (_root, app, store, headers, id) = setup();
            let caller = app.clone();
            let task = tokio::spawn(async move {
                caller
                    .session(&headers, "update_contact", &serde_json::json!({"id": 1}))
                    .await
                    .map(|_| ())
            });
            let review = pending(&store).await;
            match scenario {
                "deny" => store.decide_operation(&review.id, false).unwrap(),
                "revoke" => {
                    store.revoke(&id).unwrap();
                }
                "rotate" => {
                    store.rotate(&id).unwrap();
                }
                "network" => {
                    // Simulate an owner network change while the dialog is open.
                    app.sage.lock().await.config.network.default_network = "testnet11".into();
                    store.decide_operation(&review.id, true).unwrap();
                }
                _ => unreachable!(),
            }
            assert!(
                task.await.unwrap().is_err(),
                "{scenario} must deny execution"
            );
        }
    }
    #[tokio::test]
    async fn owner_methods_and_missing_grants_do_not_prompt_and_disconnection_cleans_queue() {
        let (_root, app, store, headers, _) = setup();
        for method in ["delete_wallet", "get_secret_key", "send_cat"] {
            assert!(
                app.session(&headers, method, &serde_json::json!({}))
                    .await
                    .is_err()
            );
            assert!(store.operations().is_empty());
        }
        let task = tokio::spawn(async move {
            app.session(&headers, "update_contact", &serde_json::json!({}))
                .await
                .map(|_| ())
        });
        pending(&store).await;
        task.abort();
        let _ = task.await;
        assert!(store.operations().is_empty());
    }
}

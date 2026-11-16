//! The in-process RPC boundary shared by HTTP and Sage Apps. A session holds the wallet
//! lock across authorization and execution; it deliberately does not dereference to Sage.
use sage::Sage;
use sage_api::{self as api, wallet_connect::*};
use sage_api_macro::impl_endpoints;
use sage_permissions::{
    ApprovalBinding, Caller, CallerId, RpcScope, WalletCapability as Cap, rpc_policy,
};
use serde::Serialize;
use serde_json::Value;
use tokio::sync::MutexGuard;

#[derive(Debug)]
pub struct RpcSession<'a> {
    sage: MutexGuard<'a, Sage>,
    caller: Option<Caller>,
    invocation: String,
    approval: Option<ApprovalBinding>,
}

// Taking ownership lets this conversion be used directly with Result::map_err.
#[allow(clippy::needless_pass_by_value)]
fn denied(message: impl ToString) -> sage::Error {
    sage::Error::PermissionDenied(message.to_string())
}

impl<'a> RpcSession<'a> {
    /// Only for the host's authenticated owner connection (the existing mTLS RPC API).
    pub fn owner(sage: MutexGuard<'a, Sage>) -> Self {
        Self {
            sage,
            caller: None,
            invocation: String::new(),
            approval: None,
        }
    }

    /// `caller` and `approval` must come from the trusted host, not client-supplied JSON.
    pub fn scoped(
        sage: MutexGuard<'a, Sage>,
        caller: Caller,
        invocation: String,
        approval: Option<ApprovalBinding>,
    ) -> Self {
        Self {
            sage,
            caller: Some(caller),
            invocation,
            approval,
        }
    }

    pub fn current_wallet(&self) -> sage::Result<(api::GetWalletResponse, u32)> {
        let fingerprint = self.sage.config.global.fingerprint;
        if let Some(caller) = &self.caller {
            caller.require_wallet(fingerprint).map_err(denied)?;
            if ![Cap::WalletGetWallet, Cap::WalletGetSecretKey]
                .iter()
                .any(|c| caller.require(*c).is_ok())
            {
                return Err(denied("Reading wallet information requires a capability"));
            }
        }
        let fingerprint = fingerprint.ok_or(sage::Error::NotLoggedIn)?;
        Ok((
            self.sage.get_wallet(api::GetWallet {
                fingerprint: Some(fingerprint),
            })?,
            fingerprint,
        ))
    }

    pub async fn get_public_keys(&mut self, req: api::GetDerivations) -> sage::Result<Vec<String>> {
        self.authorize("get_public_keys", &req)?;
        Ok(self
            .sage
            .get_derivations(req)
            .await?
            .derivations
            .into_iter()
            .map(|record| record.public_key)
            .collect())
    }

    pub async fn get_asset_balance(
        &mut self,
        kind: Option<AssetCoinType>,
        asset_id: Option<String>,
    ) -> sage::Result<(String, String, u32)> {
        self.authorize(
            "get_asset_balance",
            &serde_json::json!({"type": kind, "assetId": asset_id}),
        )?;
        let mut confirmed = 0_u128;
        let mut spendable = 0_u128;
        let mut count = 0_u32;
        let mut offset = 0_u32;
        loop {
            let coins = self
                .sage
                .get_asset_coins(GetAssetCoins {
                    kind,
                    asset_id: asset_id.clone(),
                    included_locked: Some(true),
                    offset: Some(offset),
                    limit: Some(100),
                })
                .await?;
            let page_len = u32::try_from(coins.len()).map_err(denied)?;
            for record in coins {
                let amount =
                    record.coin.amount.to_u128().ok_or_else(|| {
                        sage::Error::InvalidAmount(record.coin.amount.to_string())
                    })?;
                confirmed = confirmed
                    .checked_add(amount)
                    .ok_or_else(|| sage::Error::InvalidAmount("asset balance overflow".into()))?;
                if !record.locked {
                    spendable = spendable.checked_add(amount).ok_or_else(|| {
                        sage::Error::InvalidAmount("asset balance overflow".into())
                    })?;
                    count = count
                        .checked_add(1)
                        .ok_or_else(|| sage::Error::InvalidAmount("coin count overflow".into()))?;
                }
            }
            if page_len < 100 {
                break;
            }
            offset = offset
                .checked_add(page_len)
                .ok_or_else(|| sage::Error::InvalidAmount("pagination overflow".into()))?;
        }
        Ok((confirmed.to_string(), spendable.to_string(), count))
    }

    fn filter_response<T: Serialize + serde::de::DeserializeOwned>(
        &self,
        method: &str,
        result: T,
    ) -> sage::Result<T> {
        if method == "get_wallets" {
            if let Some(caller) = &self.caller {
                let mut value = serde_json::to_value(result).map_err(denied)?;
                if let Some(wallets) = value.get_mut("wallets").and_then(Value::as_array_mut) {
                    wallets.retain(|wallet| {
                        wallet
                            .get("fingerprint")
                            .and_then(Value::as_u64)
                            .and_then(|fp| u32::try_from(fp).ok())
                            .is_some_and(|fp| caller.wallet_scope.allows(fp))
                    });
                }
                return serde_json::from_value(value).map_err(denied);
            }
        }
        if self.caller.is_none()
            || !matches!(
                method,
                "resolve_handle"
                    | "get_handle"
                    | "get_contacts"
                    | "get_contact"
                    | "lookup_address_contacts"
                    | "export_contacts"
            )
        {
            return Ok(result);
        }
        let mut value = serde_json::to_value(result).map_err(denied)?;
        redact_private_history(
            &mut value,
            self.caller
                .as_ref()
                .is_some_and(|c| matches!(c.id, CallerId::App(_))),
        );
        serde_json::from_value(value).map_err(denied)
    }

    /// Trusted GUI adapter hook: validate the original approval and bind its final payload.
    /// Call once, after applying the user's reviewed response, before executing the RPC.
    pub fn bind_approved_request(
        &mut self,
        method: &str,
        body: &impl Serialize,
    ) -> sage::Result<()> {
        if let (Some(caller), Some(approval)) = (&self.caller, &self.approval) {
            approval
                .validate(
                    &caller.id,
                    &self.invocation,
                    self.sage.config.global.fingerprint,
                    now_ms()?,
                )
                .map_err(denied)?;
            self.approval = Some(approval.with_request(request_binding(method, body)?));
        }
        Ok(())
    }

    pub(crate) fn network_context(&self) -> sage::Result<(String, String)> {
        let network = self.sage.network();
        Ok((
            network.name.clone(),
            serde_json::to_string(&network).map_err(denied)?,
        ))
    }

    /// Preflight without consuming approval; called while the Sage lock is held.
    pub(crate) fn preflight_rpc(
        &mut self,
        method: &str,
        body: &Value,
    ) -> sage::Result<(bool, Option<u32>)> {
        let caller = self
            .caller
            .as_ref()
            .ok_or_else(|| denied("Scoped caller required"))?;
        let policy = rpc_policy(method).ok_or_else(|| denied("Unknown RPC method"))?;
        let capability = policy
            .capability
            .ok_or_else(|| denied("This RPC method requires owner credentials"))?;
        caller.require(capability).map_err(denied)?;
        let fingerprint = match policy.scope {
            RpcScope::Global | RpcScope::WalletList => None,
            RpcScope::ActiveWallet => Some(
                caller
                    .require_wallet(self.sage.config.global.fingerprint)
                    .map_err(denied)?,
            ),
            RpcScope::TargetWallet => {
                let selected = body
                    .get("fingerprint")
                    .and_then(Value::as_u64)
                    .map(u32::try_from)
                    .transpose()
                    .map_err(denied)?;
                Some(
                    caller
                        .require_wallet(selected.or(self.sage.config.global.fingerprint))
                        .map_err(denied)?,
                )
            }
        };
        if method == "get_handles" {
            if body.get("refresh").and_then(Value::as_bool) == Some(true) {
                caller.require(Cap::WalletResolveHandle).map_err(denied)?;
            }
            if body.get("filter").and_then(Value::as_str) == Some("contacts") {
                caller.require(Cap::WalletGetContacts).map_err(denied)?;
            }
        }
        Ok((policy.approval, fingerprint))
    }

    fn authorize_rpc(&mut self, method: &str, body: &Value) -> sage::Result<()> {
        let (needs_approval, fingerprint) = self.preflight_rpc(method, body)?;
        if needs_approval {
            let caller = self.caller.as_ref().unwrap();
            self.approval
                .take()
                .ok_or_else(|| denied("GUI approval is required"))?
                .validate(
                    &caller.id,
                    &request_binding(method, body)?,
                    fingerprint,
                    now_ms()?,
                )
                .map_err(denied)?;
        }
        Ok(())
    }

    fn authorize(&mut self, method: &str, body: &impl Serialize) -> sage::Result<()> {
        let Some(caller) = &self.caller else {
            return Ok(());
        };
        let body = serde_json::to_value(body).map_err(denied)?;
        if !matches!(caller.id, CallerId::App(_)) && rpc_policy(method).is_some() {
            return self.authorize_rpc(method, &body);
        }
        let fingerprint = caller
            .require_wallet(self.sage.config.global.fingerprint)
            .map_err(denied)?;
        let capabilities: &[Cap] = match method {
            "get_wallet" => &[Cap::WalletGetWallet],
            "get_secret_key" => &[Cap::WalletGetSecretKey],
            "get_sync_status" => &[Cap::WalletGetSyncStatus],
            "get_version" => &[Cap::WalletGetVersion],
            "get_xch_usd_price" => &[Cap::WalletGetXchUsdPrice],
            "check_address" => &[Cap::WalletCheckAddress],
            "get_derivations" => &[Cap::WalletGetDerivations],
            "get_public_keys" => &[Cap::WalletGetPublicKeys],
            "get_spendable_coin_count" => &[Cap::WalletGetSpendableCoinCount],
            "get_coins_by_ids" => &[Cap::WalletGetCoinsByIds],
            "get_coins" => &[Cap::WalletGetCoins],
            "get_pending_transactions" => &[Cap::WalletGetPendingTransactions],
            "get_transaction" => &[Cap::WalletGetTransaction],
            "get_transactions" => &[Cap::WalletGetTransactions],
            "filter_unlocked_coins" => &[Cap::WalletFilterUnlockedCoins],
            "get_asset_coins" => &[Cap::WalletGetAssetCoins],
            "get_asset_balance" => &[Cap::WalletGetAssetBalance],
            "get_contacts" | "lookup_address_contacts" => &[Cap::WalletGetContacts],
            "resolve_handle" => &[Cap::WalletResolveHandle, Cap::WalletSendXch],
            "get_handle" => &[Cap::WalletResolveHandle],
            "get_handles" | "lookup_address_handles" => &[Cap::WalletGetHandles],
            "send_xch" => &[Cap::WalletSendXch],
            "view_coin_spends" | "sign_coin_spends" => &[Cap::WalletSignCoinSpends],
            "sign_message_with_public_key" => &[Cap::WalletSignMessage],
            "send_transaction_immediately" => &[Cap::WalletSendTransaction],
            _ => return Err(denied("Method is not available to scoped callers")),
        };
        if !capabilities.iter().any(|cap| caller.require(*cap).is_ok()) {
            return Err(denied("Required capability has not been granted"));
        }
        // Explicit wallet selectors cannot escape the active, scoped wallet.
        if let Some(selected) = body.get("fingerprint").and_then(Value::as_u64)
            && selected != u64::from(fingerprint)
        {
            return Err(denied("Request selects a different wallet"));
        }
        if method == "get_handles" {
            if body.get("refresh").and_then(Value::as_bool) == Some(true) {
                caller.require(Cap::WalletResolveHandle).map_err(denied)?;
            }
            if body.get("filter").and_then(Value::as_str) == Some("contacts") {
                caller.require(Cap::WalletGetContacts).map_err(denied)?;
            }
        }
        let signing = matches!(
            method,
            "send_xch" | "sign_coin_spends" | "sign_message_with_public_key"
        );
        if signing && self.sage.is_watch_only() {
            return Err(denied(
                "Watch-only wallets cannot sign transactions or messages",
            ));
        }
        let auto_send = method == "send_xch"
            && caller.capabilities.contains(&Cap::WalletSendXchAutoSubmit)
            && !body
                .get("address")
                .and_then(Value::as_str)
                .is_some_and(|address| address.trim().starts_with('@'));
        if (signing && !auto_send) || method == "get_secret_key" {
            let approval = self
                .approval
                .as_ref()
                .ok_or_else(|| denied("GUI approval is required"))?;
            approval
                .validate_wallet(
                    &caller.id,
                    &request_binding(method, &body)?,
                    fingerprint,
                    now_ms()?,
                )
                .map_err(denied)?;
            // An approved session cannot execute the signing request twice.
            self.approval = None;
        }
        Ok(())
    }
}

impl_endpoints! {
    impl RpcSession<'_> {
        (repeat pub maybe_async fn endpoint(&mut self, req: api::Endpoint) -> sage::Result<api::EndpointResponse> {
            self.authorize(endpoint_string, &req)?;
            let result = self.sage.endpoint(req) maybe_await?;
            self.filter_response(endpoint_string, result)
        })
    }
}

macro_rules! wallet_connect_methods {
    ($($method:ident: $request:ident => $response:ident),* $(,)?) => {
        impl RpcSession<'_> {
            $(pub async fn $method(&mut self, req: $request) -> sage::Result<$response> {
                self.authorize(stringify!($method), &req)?;
                self.sage.$method(req).await
            })*
        }
    };
}

wallet_connect_methods! {
    filter_unlocked_coins: FilterUnlockedCoins => FilterUnlockedCoinsResponse,
    get_asset_coins: GetAssetCoins => GetAssetCoinsResponse,
    sign_message_with_public_key: SignMessageWithPublicKey => SignMessageWithPublicKeyResponse,
    send_transaction_immediately: SendTransactionImmediately => SendTransactionImmediatelyResponse,
}

fn now_ms() -> sage::Result<u64> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(denied)?
        .as_millis();
    u64::try_from(now).map_err(denied)
}

pub(crate) fn request_binding(method: &str, body: &impl Serialize) -> sage::Result<String> {
    serde_json::to_string(&(method, serde_json::to_value(body).map_err(denied)?)).map_err(denied)
}

fn redact_private_history(value: &mut Value, mask_contact_ids: bool) {
    match value {
        Value::Object(object) => {
            let is_contact = object.contains_key("recipient") && object.contains_key("name");
            for (key, value) in object.iter_mut() {
                if matches!(
                    key.as_str(),
                    "previous_address" | "last_sent_address" | "changed_from"
                ) {
                    *value = Value::Null;
                } else if mask_contact_ids && is_contact && key == "id" {
                    // Contact IDs are host-only mutation handles, not part of a read grant.
                    *value = Value::from(0);
                } else {
                    redact_private_history(value, mask_contact_ids);
                }
            }
        }
        Value::Array(values) => values
            .iter_mut()
            .for_each(|v| redact_private_history(v, mask_contact_ids)),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sage_permissions::{CallerId, WalletScope};
    use tokio::sync::Mutex;

    fn caller(capabilities: &[Cap]) -> Caller {
        Caller {
            id: CallerId::App("app-one".into()),
            capabilities: capabilities.iter().copied().collect(),
            wallet_scope: WalletScope::SelectedWallets {
                fingerprints: vec![7],
            },
        }
    }

    fn wallet() -> (tempfile::TempDir, Mutex<Sage>) {
        let dir = tempfile::tempdir().unwrap();
        let mut sage = Sage::new(dir.path(), true);
        sage.config.global.fingerprint = Some(7);
        (dir, Mutex::new(sage))
    }

    #[tokio::test]
    async fn scoped_rpc_denies_missing_capabilities_unlisted_methods_and_wallet_escapes() {
        let (_dir, sage) = wallet();
        let mut rpc = RpcSession::scoped(
            sage.lock().await,
            caller(&[Cap::WalletGetWallet]),
            String::new(),
            None,
        );
        assert!(rpc.get_version(api::GetVersion {}).is_err());
        assert!(rpc.get_wallets(api::GetWallets {}).is_err());
        assert!(
            rpc.get_secret_key(api::GetSecretKey { fingerprint: 7 })
                .is_err()
        );
        assert!(
            rpc.get_wallet(api::GetWallet {
                fingerprint: Some(8)
            })
            .is_err()
        );
        assert!(
            rpc.get_wallet(api::GetWallet {
                fingerprint: Some(7)
            })
            .is_ok()
        );
        drop(rpc);
        sage.lock().await.config.global.fingerprint = Some(8);
        let mut rpc = RpcSession::scoped(
            sage.lock().await,
            caller(&[Cap::WalletGetWallet]),
            String::new(),
            None,
        );
        assert!(
            rpc.get_wallet(api::GetWallet { fingerprint: None })
                .is_err()
        );
    }

    #[tokio::test]
    async fn summary_grants_do_not_expose_underlying_coin_or_derivation_records() {
        let (_dir, sage) = wallet();
        let mut rpc = RpcSession::scoped(
            sage.lock().await,
            caller(&[Cap::WalletGetAssetBalance, Cap::WalletGetPublicKeys]),
            String::new(),
            None,
        );
        assert!(rpc.authorize("get_asset_balance", &()).is_ok());
        assert!(rpc.authorize("get_public_keys", &()).is_ok());
        assert!(rpc.authorize("get_asset_coins", &()).is_err());
        assert!(rpc.authorize("get_derivations", &()).is_err());
    }

    #[tokio::test]
    async fn gui_approval_is_bound_to_final_rpc_payload_and_consumed_once() {
        let (_dir, sage) = wallet();
        let caller = caller(&[Cap::WalletGetSecretKey]);
        let binding = ApprovalBinding::new(
            caller.id.clone(),
            "reviewed bridge request".into(),
            Some(7),
            u64::MAX,
        );
        let mut rpc = RpcSession::scoped(
            sage.lock().await,
            caller,
            "reviewed bridge request".into(),
            Some(binding),
        );
        let request = api::GetSecretKey { fingerprint: 7 };
        // Merely holding an approval for a bridge request does not authorize arbitrary RPCs.
        assert!(rpc.authorize("get_secret_key", &request).is_err());
        rpc.bind_approved_request("get_secret_key", &request)
            .unwrap();
        assert!(
            rpc.authorize("get_secret_key", &api::GetSecretKey { fingerprint: 8 })
                .is_err()
        );
        assert!(rpc.authorize("get_secret_key", &request).is_ok());
        assert!(rpc.authorize("get_secret_key", &request).is_err());
    }

    #[tokio::test]
    async fn stale_approvals_cannot_override_wallet_scope_expiry_or_revoked_grants() {
        let (_dir, sage) = wallet();
        for (capabilities, id, wallet, expires) in [
            (vec![], CallerId::App("app-one".into()), Some(7), u64::MAX),
            (
                vec![Cap::WalletGetSecretKey],
                CallerId::App("other".into()),
                Some(7),
                u64::MAX,
            ),
            (
                vec![Cap::WalletGetSecretKey],
                CallerId::App("app-one".into()),
                Some(8),
                u64::MAX,
            ),
            (
                vec![Cap::WalletGetSecretKey],
                CallerId::App("app-one".into()),
                Some(7),
                0,
            ),
        ] {
            let binding = ApprovalBinding::new(id, "reviewed".into(), wallet, expires);
            let mut rpc = RpcSession::scoped(
                sage.lock().await,
                caller(&capabilities),
                "reviewed".into(),
                Some(binding),
            );
            let req = api::GetSecretKey { fingerprint: 7 };
            let bind = rpc.bind_approved_request("get_secret_key", &req);
            assert!(bind.is_err() || rpc.authorize("get_secret_key", &req).is_err());
        }
    }

    #[tokio::test]
    async fn capability_never_substitutes_for_gui_approval() {
        let (_dir, sage) = wallet();
        let mut rpc = RpcSession::scoped(
            sage.lock().await,
            caller(&[
                Cap::WalletSendXch,
                Cap::WalletSignCoinSpends,
                Cap::WalletSignMessage,
            ]),
            String::new(),
            None,
        );
        for method in [
            "send_xch",
            "sign_coin_spends",
            "sign_message_with_public_key",
        ] {
            assert!(
                rpc.authorize(method, &serde_json::json!({"address": "xch1recipient"}))
                    .is_err()
            );
        }
        assert!(rpc.authorize("view_coin_spends", &()).is_ok());
    }

    #[tokio::test]
    async fn automatic_sends_still_require_handle_review() {
        let (_dir, sage) = wallet();
        let mut rpc = RpcSession::scoped(
            sage.lock().await,
            caller(&[Cap::WalletSendXch, Cap::WalletSendXchAutoSubmit]),
            String::new(),
            None,
        );
        assert!(
            rpc.authorize("send_xch", &serde_json::json!({"address": "xch1recipient"}))
                .is_ok()
        );
        assert!(
            rpc.authorize("send_xch", &serde_json::json!({"address": " @alice"}))
                .is_err()
        );
    }

    #[test]
    fn scoped_responses_remove_contact_ids_and_handle_history() {
        let mut value = serde_json::json!({"contacts": [{"id": 42, "name": "Alice", "recipient": "@alice", "resolution": {"address": "current", "previous_address": "old", "last_sent_address": "paid", "changed_from": "old"}}]});
        redact_private_history(&mut value, true);
        assert_eq!(value["contacts"][0]["id"], 0);
        assert_eq!(value["contacts"][0]["resolution"]["address"], "current");
        assert!(value["contacts"][0]["resolution"]["last_sent_address"].is_null());
    }
}

#[cfg(test)]
mod http_tests {
    use super::*;
    use axum::{
        body::Body,
        http::{HeaderMap, Request, StatusCode},
    };
    use sage_permissions::{CallerId, WalletScope};
    use std::sync::{Arc, Mutex as StdMutex};
    use tokio::sync::Mutex;
    use tower::ServiceExt;

    #[derive(Debug)]
    struct Auth(StdMutex<Option<Caller>>);

    impl crate::RpcAuthenticator for Auth {
        fn authenticate(&self, headers: &HeaderMap) -> Result<Caller, String> {
            if headers.get("authorization").and_then(|v| v.to_str().ok())
                != Some("Bearer test-credential")
            {
                return Err("Missing or invalid credential".into());
            }
            self.0
                .lock()
                .unwrap()
                .clone()
                .ok_or_else(|| "Caller revoked".into())
        }
    }

    fn request(path: &str, authenticated: bool) -> Request<Body> {
        let mut request = Request::builder()
            .method("POST")
            .uri(path)
            .header("content-type", "application/json");
        if authenticated {
            request = request.header("authorization", "Bearer test-credential");
        }
        request.body(Body::from("{}")).unwrap()
    }

    #[tokio::test]
    async fn scoped_http_authenticates_every_request_and_never_falls_back_to_owner() {
        let dir = tempfile::tempdir().unwrap();
        let mut sage = Sage::new(dir.path(), true);
        sage.config.global.fingerprint = Some(7);
        let sage = Arc::new(Mutex::new(sage));
        let auth = Arc::new(Auth(StdMutex::new(Some(Caller {
            id: CallerId::Agent("reader".into()),
            capabilities: [Cap::WalletGetVersion].into(),
            wallet_scope: WalletScope::SelectedWallets {
                fingerprints: vec![7],
            },
        }))));
        let router = crate::make_scoped_router(sage.clone(), auth.clone());
        assert_eq!(
            router
                .clone()
                .oneshot(request("/get_version", false))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            router
                .clone()
                .oneshot(request("/get_version", true))
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
        assert_eq!(
            router
                .clone()
                .oneshot(request("/get_wallets", true))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        sage.lock().await.config.global.fingerprint = Some(8);
        assert_eq!(
            router
                .clone()
                .oneshot(request("/get_version", true))
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
        sage.lock().await.config.global.fingerprint = Some(7);
        *auth.0.lock().unwrap() = None;
        assert_eq!(
            router
                .oneshot(request("/get_version", true))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            crate::make_router(sage)
                .oneshot(request("/get_version", false))
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
    }
}

#[cfg(test)]
mod full_rpc_tests {
    use super::*;
    use sage_permissions::{RPC_POLICIES, WalletScope};
    use tokio::sync::Mutex;

    fn agent(capabilities: Vec<Cap>) -> Caller {
        Caller {
            id: CallerId::Agent("agent".into()),
            capabilities: capabilities.into_iter().collect(),
            wallet_scope: WalletScope::SelectedWallets {
                fingerprints: vec![7],
            },
        }
    }
    #[tokio::test]
    async fn all_rpc_policies_enforce_grants_scope_and_owner_boundary() {
        let root = tempfile::tempdir().unwrap();
        let mut sage = Sage::new(root.path(), true);
        sage.config.global.fingerprint = Some(7);
        let sage = Mutex::new(sage);
        for policy in RPC_POLICIES {
            let mut session = RpcSession::scoped(
                sage.lock().await,
                agent(policy.capability.into_iter().collect()),
                String::new(),
                None,
            );
            let result = session.preflight_rpc(policy.method, &serde_json::json!({}));
            assert_eq!(
                result.is_ok(),
                policy.capability.is_some(),
                "{}",
                policy.method
            );
            if let Ok((approval, _)) = result {
                assert_eq!(approval, policy.approval, "{}", policy.method);
            }
            drop(session);
            let mut ungranted =
                RpcSession::scoped(sage.lock().await, agent(vec![]), String::new(), None);
            assert!(
                ungranted
                    .preflight_rpc(policy.method, &serde_json::json!({}))
                    .is_err(),
                "{}",
                policy.method
            );
        }
        sage.lock().await.config.global.fingerprint = None;
        let mut global = RpcSession::scoped(
            sage.lock().await,
            agent(vec![
                Cap::WalletGetVersion,
                Cap::WalletGetNfts,
                Cap::WalletGetWallet,
            ]),
            String::new(),
            None,
        );
        assert!(
            global
                .preflight_rpc("get_version", &serde_json::json!({}))
                .is_ok()
        );
        assert!(
            global
                .preflight_rpc("get_nfts", &serde_json::json!({}))
                .is_err()
        );
        assert!(
            global
                .preflight_rpc("get_wallet", &serde_json::json!({"fingerprint":7}))
                .is_ok()
        );
        assert!(
            global
                .preflight_rpc("get_wallet", &serde_json::json!({"fingerprint":8}))
                .is_err()
        );
    }
    #[tokio::test]
    async fn rpc_approval_binds_payload_wallet_and_can_execute_only_once() {
        let root = tempfile::tempdir().unwrap();
        let mut sage = Sage::new(root.path(), true);
        sage.config.global.fingerprint = Some(7);
        let sage = Mutex::new(sage);
        let body = serde_json::json!({"address":"recipient", "amount":"123", "fee":"1"});
        for (fp, input, expected) in [
            (Some(7), body.clone(), true),
            (Some(8), body.clone(), false),
            (Some(7), serde_json::json!({"amount":"999"}), false),
        ] {
            sage.lock().await.config.global.fingerprint = fp;
            let caller = agent(vec![Cap::WalletSendCat]);
            let approval = ApprovalBinding::new(
                caller.id.clone(),
                request_binding("send_cat", &body).unwrap(),
                Some(7),
                u64::MAX,
            );
            let mut session =
                RpcSession::scoped(sage.lock().await, caller, String::new(), Some(approval));
            assert_eq!(session.authorize("send_cat", &input).is_ok(), expected);
            assert!(session.authorize("send_cat", &input).is_err());
        }
    }
    #[tokio::test]
    async fn wallet_list_respects_scope_and_rpc_contact_ids_remain_usable() {
        let root = tempfile::tempdir().unwrap();
        let sage = Mutex::new(Sage::new(root.path(), true));
        let session = RpcSession::scoped(
            sage.lock().await,
            agent(vec![Cap::WalletGetWallets]),
            String::new(),
            None,
        );
        let result: Value = session
            .filter_response(
                "get_wallets",
                serde_json::json!({"wallets":[{"fingerprint":7},{"fingerprint":8}]}),
            )
            .unwrap();
        assert_eq!(result["wallets"], serde_json::json!([{"fingerprint":7}]));
        let contact: Value = session.filter_response("get_contact", serde_json::json!({"id":1,"recipient":"address","name":"Friend","previous_address":"private"})).unwrap();
        assert_eq!(contact["id"], 1);
        assert!(contact["previous_address"].is_null());
    }
}

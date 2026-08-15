use std::pin::Pin;

use anyhow::Result;
use tauri::{AppHandle, State};

use crate::{AppMutationDraft, AppsDbTx, AppsHostState, SageApp, SharedSageApp};

pub(crate) struct AppMutationManager<'a> {
    #[allow(dead_code)]
    app_handle: &'a AppHandle,
    apps_state: &'a State<'a, AppsHostState>,
}

pub(crate) struct AppMutationContext {
    draft: AppMutationDraft,
    tx: AppsDbTx,
}

impl AppMutationContext {
    pub(crate) fn draft(&self) -> &AppMutationDraft {
        &self.draft
    }

    pub(crate) fn draft_mut(&mut self) -> &mut AppMutationDraft {
        &mut self.draft
    }

    pub(crate) fn tx(&mut self) -> &mut AppsDbTx {
        &mut self.tx
    }

    fn into_parts(self) -> (AppMutationDraft, AppsDbTx) {
        (self.draft, self.tx)
    }
}

impl<'a> AppMutationManager<'a> {
    pub(crate) fn new(app_handle: &'a AppHandle, apps_state: &'a State<'a, AppsHostState>) -> Self {
        Self {
            app_handle,
            apps_state,
        }
    }

    pub(crate) async fn mutate_shared_app<T>(
        &self,
        app: &SharedSageApp,
        f: impl for<'m> FnOnce(
            &'m mut AppMutationContext,
        ) -> Pin<Box<dyn Future<Output = Result<T>> + Send + 'm>>
        + Send,
    ) -> Result<T, String>
    where
        T: Send,
    {
        let app_id = app.id();

        let mut tx = self
            .apps_state
            .db
            .begin_immediate()
            .await
            .map_err(|err| format!("failed to begin app mutation transaction: {err}"))?;

        let current = tx
            .load_user_app(&app_id)
            .await
            .map_err(|err| format!("failed to load current app state: {err}"))?
            .into_sage_app();

        let draft = AppMutationDraft::new(
            current
                .clone_for_rollback()
                .map_err(|err| format!("failed to clone app draft: {err}"))?,
        );

        let mut ctx = AppMutationContext { draft, tx };

        let value = match f(&mut ctx).await {
            Ok(value) => value,
            Err(err) => {
                ctx.tx.rollback().await;
                return Err(err.to_string());
            }
        };

        let (draft, mut tx) = ctx.into_parts();

        if let Err(err) = Self::validate(draft.app()) {
            tx.rollback().await;
            return Err(err);
        }

        let Some(draft_user_app) = draft.app().as_user() else {
            tx.rollback().await;
            return Err("only user apps can be persisted by app mutation manager".to_string());
        };

        if let Err(err) = tx.persist_user_app(draft_user_app).await {
            tx.rollback().await;
            return Err(format!("failed to persist app draft: {err}"));
        }

        let reloaded = match tx.load_user_app(&app_id).await {
            Ok(app) => app.into_sage_app(),
            Err(err) => {
                tx.rollback().await;
                return Err(format!("failed to reload app draft: {err}"));
            }
        };

        if let Err(err) = Self::assert_round_trip(draft.app(), &reloaded) {
            tx.rollback().await;
            return Err(err);
        }

        if let Err(err) = tx.commit().await {
            return Err(format!("failed to commit app mutation transaction: {err}"));
        }

        app.replace_committed(reloaded);

        Ok(value)
    }

    fn validate(app: &SageApp) -> Result<(), String> {
        if app.is_system() {
            return Ok(());
        }

        let common = app.common();

        if common.has_external_access() && common.has_secret_access() {
            return Err(
                "app permissions cannot include both external access and sensitive secret access"
                    .to_string(),
            );
        }

        if common.has_external_access() && common.origin_webview_storage_may_contain_secrets() {
            return Err(
                "app permissions cannot include external access while origin webview storage may contain secrets"
                    .to_string(),
            );
        }

        Ok(())
    }

    fn assert_round_trip(expected: &SageApp, actual: &SageApp) -> Result<(), String> {
        let expected = expected
            .as_user()
            .ok_or_else(|| "expected app is not a user app".to_string())?;

        let actual = actual
            .as_user()
            .ok_or_else(|| "loaded app is not a user app".to_string())?;

        let expected = serde_json::to_value(expected)
            .map_err(|err| format!("failed to serialize expected app state: {err}"))?;

        let actual = serde_json::to_value(actual)
            .map_err(|err| format!("failed to serialize loaded app state: {err}"))?;

        if expected != actual {
            return Err("app DB round-trip mismatch".to_string());
        }

        Ok(())
    }
}

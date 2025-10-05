use sage_api::{
    RegisterWebhook, RegisterWebhookResponse, UnregisterWebhook, UnregisterWebhookResponse,
};

use crate::{Result, Sage};

impl Sage {
    pub async fn register_webhook(
        &mut self,
        req: RegisterWebhook,
    ) -> Result<RegisterWebhookResponse> {
        let webhook_id = self
            .webhook_manager
            .register_webhook(req.url, req.event_types)
            .await;

        // Save to config
        self.save_webhooks_config().await?;

        Ok(RegisterWebhookResponse { webhook_id })
    }

    pub async fn unregister_webhook(
        &mut self,
        req: UnregisterWebhook,
    ) -> Result<UnregisterWebhookResponse> {
        self.webhook_manager
            .unregister_webhook(&req.webhook_id)
            .await;

        // Save to config
        self.save_webhooks_config().await?;

        Ok(UnregisterWebhookResponse {})
    }
}

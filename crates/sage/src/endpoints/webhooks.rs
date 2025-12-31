use sage_api::{
    GetWebhooks, GetWebhooksResponse, RegisterWebhook, RegisterWebhookResponse, UnregisterWebhook,
    UnregisterWebhookResponse, UpdateWebhook, UpdateWebhookResponse,
};
use sage_config::WebhookEntry;

use crate::{Error, Result, Sage};

impl Sage {
    pub async fn register_webhook(
        &mut self,
        req: RegisterWebhook,
    ) -> Result<RegisterWebhookResponse> {
        let webhook_id = self
            .webhook_manager
            .register_webhook(req.url, req.event_types, req.secret)
            .await;

        self.save_webhooks_config().await?;

        Ok(RegisterWebhookResponse { webhook_id })
    }

    pub async fn unregister_webhook(
        &mut self,
        req: UnregisterWebhook,
    ) -> Result<UnregisterWebhookResponse> {
        let removed = self
            .webhook_manager
            .unregister_webhook(&req.webhook_id)
            .await;

        if !removed {
            return Err(Error::UnknownWebhook(req.webhook_id));
        }

        self.save_webhooks_config().await?;

        Ok(UnregisterWebhookResponse {})
    }

    pub async fn get_webhooks(&mut self, _req: GetWebhooks) -> Result<GetWebhooksResponse> {
        let webhooks = self.webhook_manager.list_webhooks().await;
        Ok(GetWebhooksResponse {
            webhooks: webhooks
                .into_iter()
                .map(|w| WebhookEntry {
                    id: w.id,
                    url: w.url,
                    events: w.events,
                    enabled: w.active,
                    secret: None, // Don't expose secret in API responses
                    last_delivered_at: w.last_delivered_at,
                    last_delivery_attempt_at: w.last_delivery_attempt_at,
                })
                .collect(),
        })
    }

    pub async fn update_webhook(&mut self, req: UpdateWebhook) -> Result<UpdateWebhookResponse> {
        let updated = self
            .webhook_manager
            .update_webhook(&req.webhook_id, req.enabled)
            .await;

        if !updated {
            return Err(Error::UnknownWebhook(req.webhook_id));
        }

        self.save_webhooks_config().await?;

        Ok(UpdateWebhookResponse {})
    }
}

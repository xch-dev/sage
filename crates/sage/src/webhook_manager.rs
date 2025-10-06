use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub id: String,
    pub url: String,
    /// None means "all events, including future ones"
    pub events: Option<Vec<String>>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEventPayload {
    pub id: String,
    pub fingerprint: Option<u32>,
    pub network: String,
    pub event_type: String,
    pub timestamp: i64,
    pub data: serde_json::Value,
}

// Webhook manager
#[derive(Debug, Clone)]
pub struct WebhookManager {
    webhooks: Arc<RwLock<HashMap<String, WebhookConfig>>>,
    client: Client,
    fingerprint: Arc<RwLock<Option<u32>>>,
    network: Arc<RwLock<String>>,
}

impl Default for WebhookManager {
    fn default() -> Self {
        Self {
            webhooks: Arc::new(RwLock::new(HashMap::new())),
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            fingerprint: Arc::new(RwLock::new(None)),
            network: Arc::new(RwLock::new(String::new())),
        }
    }
}

impl WebhookManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn set_fingerprint(&self, fingerprint: Option<u32>) {
        *self.fingerprint.write().await = fingerprint;
    }

    pub async fn set_network(&self, network: String) {
        *self.network.write().await = network;
    }

    pub async fn register_webhook(&self, url: String, events: Option<Vec<String>>) -> String {
        let id = Uuid::new_v4().to_string();
        let config = WebhookConfig {
            id: id.clone(),
            url,
            events,
            active: true,
        };

        let mut webhooks = self.webhooks.write().await;
        webhooks.insert(id.clone(), config);
        id
    }

    pub async fn unregister_webhook(&self, id: &str) -> bool {
        let mut webhooks = self.webhooks.write().await;
        webhooks.remove(id).is_some()
    }

    pub async fn update_webhook(&self, id: &str, active: bool) -> bool {
        let mut webhooks = self.webhooks.write().await;
        if let Some(webhook) = webhooks.get_mut(id) {
            webhook.active = active;
            true
        } else {
            false
        }
    }

    pub async fn send_event(&self, event_type: String, data: serde_json::Value) {
        let fingerprint = *self.fingerprint.read().await;
        let network = self.network.read().await.clone();
        let event = WebhookEventPayload {
            id: Uuid::new_v4().to_string(),
            fingerprint,
            network,
            event_type: event_type.clone(),
            timestamp: chrono::Utc::now().timestamp(),
            data,
        };

        let webhooks = self.webhooks.read().await;
        let interested_webhooks: Vec<WebhookConfig> = webhooks
            .values()
            .filter(|w| {
                w.active
                    && match &w.events {
                        None => true, // None means all events
                        Some(events) => events.contains(&event_type),
                    }
            })
            .cloned()
            .collect();

        for webhook in interested_webhooks {
            let client = self.client.clone();
            let event = event.clone();
            tokio::spawn(async move {
                Self::deliver_webhook(client, webhook, event).await;
            });
        }
    }

    async fn deliver_webhook(client: Client, webhook: WebhookConfig, event: WebhookEventPayload) {
        const MAX_RETRIES: u32 = 3;

        for attempt in 0..MAX_RETRIES {
            match Self::send_webhook_request(&client, &webhook, &event).await {
                Ok(()) => {
                    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC");
                    debug!("[{}] Webhook delivered to {}", timestamp, webhook.url);
                    return;
                }
                Err(e) => {
                    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC");
                    error!(
                        "[{}] Webhook delivery failed (attempt {}/{}): {} - {}",
                        timestamp,
                        attempt + 1,
                        MAX_RETRIES,
                        webhook.url,
                        e
                    );
                    if attempt < MAX_RETRIES - 1 {
                        // Exponential backoff
                        let delay = std::time::Duration::from_secs(2u64.pow(attempt));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }
    }

    async fn send_webhook_request(
        client: &Client,
        webhook: &WebhookConfig,
        event: &WebhookEventPayload,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = client.post(&webhook.url).json(event);
        let response = request.send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("HTTP {}", response.status()).into())
        }
    }

    pub async fn list_webhooks(&self) -> Vec<WebhookConfig> {
        let webhooks = self.webhooks.read().await;
        webhooks.values().cloned().collect()
    }

    pub async fn load_webhooks(&self, entries: Vec<(String, String, Option<Vec<String>>, bool)>) {
        let mut webhooks = self.webhooks.write().await;
        for (id, url, events, enabled) in entries {
            let config = WebhookConfig {
                id: id.clone(),
                url,
                events,
                active: enabled,
            };
            webhooks.insert(id, config);
        }
    }

    // Get webhooks in a format suitable for saving to config
    pub async fn get_webhook_entries(&self) -> Vec<(String, String, Option<Vec<String>>, bool)> {
        let webhooks = self.webhooks.read().await;
        webhooks
            .values()
            .map(|w| (w.id.clone(), w.url.clone(), w.events.clone(), w.active))
            .collect()
    }
}

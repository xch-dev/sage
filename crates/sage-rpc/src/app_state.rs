use std::sync::Arc;

use sage::Sage;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct AppState {
    pub sage: Arc<Mutex<Sage>>,
}

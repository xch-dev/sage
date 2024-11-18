use axum::Router;

use crate::app_state::AppState;

pub fn api_router() -> Router<AppState> {
    Router::new()
}

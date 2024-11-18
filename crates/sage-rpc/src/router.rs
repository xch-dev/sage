use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use sage::Result;
use sage_api::{
    DeleteKey, ErrorKind, GenerateMnemonic, GetKey, GetKeys, GetSecretKey, ImportKey, Login,
    Logout, Resync,
};
use serde::Serialize;

use crate::app_state::AppState;

pub async fn login(State(state): State<AppState>, Json(req): Json<Login>) -> Response {
    handle(state.sage.lock().await.login(req).await)
}

pub async fn logout(State(state): State<AppState>, Json(req): Json<Logout>) -> Response {
    handle(state.sage.lock().await.logout(req).await)
}

pub async fn resync(State(state): State<AppState>, Json(req): Json<Resync>) -> Response {
    handle(state.sage.lock().await.resync(req).await)
}

pub async fn generate_mnemonic(
    State(state): State<AppState>,
    Json(req): Json<GenerateMnemonic>,
) -> Response {
    handle(state.sage.lock().await.generate_mnemonic(req))
}

pub async fn import_key(State(state): State<AppState>, Json(req): Json<ImportKey>) -> Response {
    handle(state.sage.lock().await.import_key(req).await)
}

pub async fn delete_key(State(state): State<AppState>, Json(req): Json<DeleteKey>) -> Response {
    handle(state.sage.lock().await.delete_key(req))
}

pub async fn get_key(State(state): State<AppState>, Json(req): Json<GetKey>) -> Response {
    handle(state.sage.lock().await.get_key(req))
}

pub async fn get_secret_key(
    State(state): State<AppState>,
    Json(req): Json<GetSecretKey>,
) -> Response {
    handle(state.sage.lock().await.get_secret_key(req))
}

pub async fn get_keys(State(state): State<AppState>, Json(req): Json<GetKeys>) -> Response {
    handle(state.sage.lock().await.get_keys(req))
}

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/resync", post(resync))
        .route("/generate_mnemonic", post(generate_mnemonic))
        .route("/import_key", post(import_key))
        .route("/delete_key", post(delete_key))
        .route("/get_key", post(get_key))
        .route("/get_secret_key", post(get_secret_key))
        .route("/get_keys", post(get_keys))
}

fn handle<T>(value: Result<T>) -> Response
where
    T: Serialize,
{
    match value {
        Ok(data) => Json(data).into_response(),
        Err(error) => {
            let status = match error.kind() {
                ErrorKind::Api => StatusCode::BAD_REQUEST,
                ErrorKind::NotFound => StatusCode::NOT_FOUND,
                ErrorKind::Unauthorized => StatusCode::UNAUTHORIZED,
                ErrorKind::Wallet | ErrorKind::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, error.to_string()).into_response()
        }
    }
}

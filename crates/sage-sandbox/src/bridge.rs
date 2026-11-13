//! The HTTP server that stands in for Tauri's IPC.
//!
//! `POST /{wallet}/invoke/{command}` takes the same arguments object that the frontend passes to
//! Tauri's `invoke`, such as `{ "req": { ... } }`, and answers with the command's result. Errors
//! come back as `400` with the same body the desktop app rejects with.

use std::{collections::HashMap, convert::Infallible, path::Path as FsPath, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, State, rejection::JsonRejection},
    http::StatusCode,
    response::{
        IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
    routing::{any, get, post},
};
use chia_wallet_sdk::test::PeerSimulator;
use futures_util::{Stream, stream};
use sage::Sage;
use sage_api::SyncEvent as ApiEvent;
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::{
    net::TcpListener,
    sync::broadcast::{self, error::RecvError},
};

use crate::{BridgeError, SandboxWallet, WalletInfo, call_endpoint, start_wallets};

#[derive(Debug, Clone)]
struct BridgeState {
    wallets: Arc<HashMap<String, SandboxWallet>>,
    /// The wallets' details in startup order, for `GET /`.
    infos: Arc<Vec<WalletInfo>>,
}

/// Starts the wallets, pointed at the XCHandles stand-in that `listener` will serve, and returns
/// the bridge to serve on it along with the wallets' details.
pub async fn start_bridge(
    data_dir: &FsPath,
    sim: &PeerSimulator,
    listener: &TcpListener,
) -> anyhow::Result<(Router, Vec<WalletInfo>)> {
    let xchandles_url = format!("http://{}/xchandles", listener.local_addr()?);
    let wallets = start_wallets(data_dir, sim, &xchandles_url).await?;
    let infos = wallets.iter().map(|wallet| wallet.info.clone()).collect();

    Ok((bridge_router(wallets), infos))
}

fn bridge_router(wallets: Vec<SandboxWallet>) -> Router {
    let infos = wallets.iter().map(|wallet| wallet.info.clone()).collect();
    let wallets = wallets
        .into_iter()
        .map(|wallet| (wallet.info.name.clone(), wallet))
        .collect();

    Router::new()
        .route("/", get(status))
        .route("/:wallet/invoke/:command", post(invoke))
        .route("/:wallet/events", get(events))
        .route("/xchandles/*path", any(xchandles))
        .with_state(BridgeState {
            wallets: Arc::new(wallets),
            infos: Arc::new(infos),
        })
}

async fn status(State(state): State<BridgeState>) -> Json<Vec<WalletInfo>> {
    Json(state.infos.as_ref().clone())
}

async fn invoke(
    State(state): State<BridgeState>,
    Path((wallet, command)): Path<(String, String)>,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    let Some(wallet) = state.wallets.get(&wallet) else {
        return unknown_wallet(&wallet);
    };

    // Require a JSON body, as the UI always sends one. This also means a page on another origin
    // can't call the bridge without a CORS preflight, which it doesn't answer.
    let args = match body {
        Ok(Json(args)) => args,
        Err(rejection) => return bad_request(BridgeError::api(rejection.body_text())),
    };

    // Run the command on its own task, so that it finishes even if the page goes away mid-request,
    // as a Tauri command would.
    let sage = wallet.sage.clone();
    let result = tokio::spawn(async move {
        let mut sage = sage.lock().await;
        invoke_command(&mut sage, &command, args).await
    })
    .await;

    match result {
        Ok(Ok(value)) => Json(value).into_response(),
        Ok(Err(error)) => bad_request(error),
        Err(error) => bad_request(BridgeError::internal(format!(
            "The command panicked: {error}"
        ))),
    }
}

/// Handles the commands that `src-tauri/src/commands.rs` implements by hand, then falls back to
/// the endpoints that every command shares with the RPC.
async fn invoke_command(sage: &mut Sage, command: &str, args: Value) -> Result<Value, BridgeError> {
    match command {
        // The backend was initialized before the bridge started, and there's no window to close.
        "initialize" | "quit" => Ok(Value::Null),
        "validate_address" => {
            let AddressArgs { address } = parse_args(args)?;
            Ok(Value::Bool(sage.is_valid_address(&address)))
        }
        "network_config" => BridgeError::to_value(&sage.config.network),
        "wallet_config" => {
            let FingerprintArgs { fingerprint } = parse_args(args)?;
            BridgeError::to_value(sage.wallet_entry(fingerprint))
        }
        "default_wallet_config" => BridgeError::to_value(sage.wallet_config.defaults),
        "switch_wallet" => {
            sage.switch_wallet().await?;
            Ok(Value::Null)
        }
        "move_key" => {
            let MoveKeyArgs { fingerprint, index } = parse_args(args)?;
            sage.move_key(fingerprint, index)?;
            Ok(Value::Null)
        }
        // The RPC server would listen on the real RPC port, which could clash with a running Sage.
        "is_rpc_running" | "get_rpc_run_on_startup" => Ok(Value::Bool(false)),
        // Logs go to the sandbox's own log file instead.
        "get_logs" => Ok(json!([])),
        _ => {
            let req = match args {
                Value::Object(mut args) => args.remove("req").unwrap_or(Value::Null),
                _ => Value::Null,
            };

            call_endpoint(sage, command, req).await.unwrap_or_else(|| {
                Err(BridgeError::api(format!(
                    "`{command}` isn't available in the sandbox"
                )))
            })
        }
    }
}

#[derive(Deserialize)]
struct AddressArgs {
    address: String,
}

#[derive(Deserialize)]
struct FingerprintArgs {
    fingerprint: u32,
}

#[derive(Deserialize)]
struct MoveKeyArgs {
    fingerprint: u32,
    index: u32,
}

fn parse_args<T: for<'de> Deserialize<'de>>(args: Value) -> Result<T, BridgeError> {
    serde_json::from_value(args).map_err(|error| BridgeError::invalid_request(&error))
}

/// Streams the wallet's `sync-event` payloads as server-sent events.
async fn events(State(state): State<BridgeState>, Path(wallet): Path<String>) -> Response {
    let Some(wallet) = state.wallets.get(&wallet) else {
        return unknown_wallet(&wallet);
    };

    Sse::new(event_stream(wallet.events.subscribe()))
        .keep_alive(KeepAlive::default())
        .into_response()
}

fn event_stream(
    receiver: broadcast::Receiver<ApiEvent>,
) -> impl Stream<Item = Result<Event, Infallible>> {
    stream::unfold(receiver, |mut receiver| async move {
        loop {
            match receiver.recv().await {
                Ok(event) => {
                    let event = Event::default().json_data(&event).unwrap_or_default();
                    return Some((Ok(event), receiver));
                }
                // A slow page missed some events. Refreshes are idempotent, so carry on.
                Err(RecvError::Lagged(_)) => {}
                Err(RecvError::Closed) => return None,
            }
        }
    })
}

/// Stands in for the XCHandles API by reporting that its index is behind the chain, so handle
/// lookups fall back to what the wallet already knows instead of reaching the real service.
async fn xchandles() -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({
            "code": "index_stale",
            "message": "The sandbox has no XCHandles index",
        })),
    )
        .into_response()
}

fn bad_request(error: BridgeError) -> Response {
    (StatusCode::BAD_REQUEST, Json(error)).into_response()
}

fn unknown_wallet(wallet: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(BridgeError::api(format!("There is no `{wallet}` wallet"))),
    )
        .into_response()
}

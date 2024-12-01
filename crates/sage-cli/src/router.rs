use std::{
    env,
    fs::{self, File},
    io::Read,
    net::SocketAddr,
    path::PathBuf,
    sync::Arc,
};

use anyhow::{bail, Result};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use paste::paste;
use reqwest::{Client, Identity};
use sage::Sage;
use sage_api::ErrorKind;
use sage_config::Config;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::info;

use crate::{app_state::AppState, tls::load_rustls_config};

macro_rules! routes {
    ( $( $route:ident $( $kw:ident )?: $ty:ident = $url:literal ),* $(,)? ) => {
        $( pub async fn $route(State(state): State<AppState>, Json(req): Json<sage_api::$ty>) -> Response {
            handle(state.sage.lock().await.$route(req) $( .$kw )?)
        } )*

        pub fn api_router() -> Router<AppState> {
            Router::new()
                $( .route($url, post($route)) )*
        }

        #[derive(Debug, Parser)]
        #[clap(rename_all = "snake_case")]
        pub enum RpcCommand {
            Start,
            $( $ty { #[clap(value_parser = parse_with_serde::<sage_api::$ty>)] body: sage_api::$ty } , )*
        }

        paste! {
            impl RpcCommand {
                pub async fn handle(self, path: PathBuf) -> anyhow::Result<()> {
                    match self {
                        Self::Start => start_rpc(path).await,
                        $( Self::$ty { body } => call_rpc::<_, sage_api::[< $ty Response >]>(path, $url, body).await , )*
                    }
                }
            }
        }
    };
}

routes!(
    login await: Login = "/login",
    logout await: Logout = "/logout",
    resync await: Resync = "/resync",
    generate_mnemonic: GenerateMnemonic = "/generate_mnemonic",
    import_key await: ImportKey = "/import_key",
    delete_key: DeleteKey = "/delete_key",
    rename_key: RenameKey = "/rename_key",
    get_key: GetKey = "/get_key",
    get_secret_key: GetSecretKey = "/get_secret_key",
    get_keys: GetKeys = "/get_keys",

    get_sync_status await: GetSyncStatus = "/get_sync_status",
    get_derivations await: GetDerivations = "/get_derivations",
    get_xch_coins await: GetXchCoins = "/get_xch_coins",
    get_cat_coins await: GetCatCoins = "/get_cat_coins",
    get_cats await: GetCats = "/get_cats",
    get_cat await: GetCat = "/get_cat",
    get_dids await: GetDids = "/get_dids",
    get_pending_transactions await: GetPendingTransactions = "/get_pending_transactions",
    get_nft_status await: GetNftStatus = "/get_nft_status",
    get_nft_collections await: GetNftCollections = "/get_nft_collections",
    get_nft_collection await: GetNftCollection = "/get_nft_collection",
    get_nfts await: GetNfts = "/get_nfts",
    get_nft await: GetNft = "/get_nft",

    send_xch await: SendXch = "/send_xch",
    combine_xch await: CombineXch = "/combine_xch",
    split_xch await: SplitXch = "/split_xch",
    combine_cat await: CombineCat = "/combine_cat",
    split_cat await: SplitCat = "/split_cat",
    issue_cat await: IssueCat = "/issue_cat",
    send_cat await: SendCat = "/send_cat",
    create_did await: CreateDid = "/create_did",
    bulk_mint_nfts await: BulkMintNfts = "/bulk_mint_nfts",
    transfer_nfts await: TransferNfts = "/transfer_nfts",
    add_nft_uri await: AddNftUri = "/add_nft_uri",
    assign_nfts_to_did await: AssignNftsToDid = "/assign_nfts_to_did",
    transfer_dids await: TransferDids = "/transfer_dids",
    sign_coin_spends await: SignCoinSpends = "/sign_coin_spends",
    view_coin_spends await: ViewCoinSpends = "/view_coin_spends",
    submit_transaction await: SubmitTransaction = "/submit_transaction",

    make_offer await: MakeOffer = "/make_offer",
    take_offer await: TakeOffer = "/take_offer",
    view_offer await: ViewOffer = "/view_offer",
    import_offer await: ImportOffer = "/import_offer",
    get_offers await: GetOffers = "/get_offers",
    get_offer await: GetOffer = "/get_offer",
    delete_offer await: DeleteOffer = "/delete_offer",

    get_peers await: GetPeers = "/get_peers",
    remove_peer await: RemovePeer = "/remove_peer",
    add_peer await: AddPeer = "/add_peer",
    set_discover_peers await: SetDiscoverPeers = "/set_discover_peers",
    set_target_peers await: SetTargetPeers = "/set_target_peers",
    set_network_id await: SetNetworkId = "/set_network_id",
    set_derive_automatically: SetDeriveAutomatically = "/set_derive_automatically",
    set_derivation_batch_size: SetDerivationBatchSize = "/set_derivation_batch_size",
    get_networks: GetNetworks = "/get_networks",

    remove_cat await: RemoveCat = "/remove_cat",
    update_cat await: UpdateCat = "/update_cat",
    update_did await: UpdateDid = "/update_did",
    update_nft await: UpdateNft = "/update_nft",
);

async fn start_rpc(path: PathBuf) -> Result<()> {
    let mut app = Sage::new(&path);
    let mut receiver = app.initialize().await?;

    tokio::spawn(async move {
        while let Some(message) = receiver.recv().await {
            println!("{message:?}");
        }
    });

    let addr: SocketAddr = ([127, 0, 0, 1], app.config.rpc.server_port).into();
    info!("RPC server is listening at {addr}");

    let app = api_router().with_state(AppState {
        sage: Arc::new(Mutex::new(app)),
    });

    let config = load_rustls_config(
        path.join("ssl")
            .join("wallet.crt")
            .to_str()
            .expect("could not convert path to string"),
        path.join("ssl")
            .join("wallet.key")
            .to_str()
            .expect("could not convert path to string"),
    )?;

    axum_server::bind_rustls(addr, RustlsConfig::from_config(Arc::new(config)))
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

pub async fn call_rpc<T: Serialize, R: Serialize + DeserializeOwned>(
    path: PathBuf,
    url: &str,
    body: T,
) -> Result<()> {
    let addr = if let Ok(addr) = env::var("SAGE_RPC_HOST") {
        addr.parse::<SocketAddr>()?
    } else {
        let config_path = path.join("config.toml");
        let config = if config_path.try_exists()? {
            let text = fs::read_to_string(&config_path)?;
            toml::from_str(&text)?
        } else {
            Config::default()
        };
        ([127, 0, 0, 1], config.rpc.server_port).into()
    };

    let cert_path = env::var("SAGE_RPC_CERT_PATH").unwrap_or_else(|_| {
        path.join("ssl")
            .join("wallet.crt")
            .to_str()
            .expect("could not convert path to string")
            .to_string()
    });

    let key_path = env::var("SAGE_RPC_KEY_PATH").unwrap_or_else(|_| {
        path.join("ssl")
            .join("wallet.key")
            .to_str()
            .expect("could not convert path to string")
            .to_string()
    });

    let mut buf = Vec::new();
    File::open(cert_path)?.read_to_end(&mut buf)?;
    File::open(key_path)?.read_to_end(&mut buf)?;
    let identity = Identity::from_pem(&buf)?;

    let response = Client::builder()
        .danger_accept_invalid_certs(true)
        .use_rustls_tls()
        .identity(identity)
        .build()?
        .post(format!("https://{addr}{url}"))
        .json(&body)
        .send()
        .await?;

    if response.status() != StatusCode::OK {
        bail!(response.text().await?);
    }

    let json = response.json::<R>().await?;

    println!("{}", serde_json::to_string_pretty(&json)?);

    Ok(())
}

fn parse_with_serde<T: for<'de> Deserialize<'de>>(s: &str) -> Result<T, String> {
    serde_json::from_str(s).map_err(|error| error.to_string())
}

fn handle<T>(value: sage::Result<T>) -> Response
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

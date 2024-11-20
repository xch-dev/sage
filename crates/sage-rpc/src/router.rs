use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use sage::Result;
use sage_api::ErrorKind;
use serde::Serialize;

use crate::app_state::AppState;

macro_rules! routes {
    ( $( $route:ident $( $kw:ident )?: $ty:ident = $url:literal ),* $(,)? ) => {
        $( pub async fn $route(State(state): State<AppState>, Json(req): Json<sage_api::$ty>) -> Response {
            handle(state.sage.lock().await.$route(req) $( .$kw )?)
        } )*

        pub fn api_router() -> Router<AppState> {
            Router::new()
                $( .route($url, post($route)) )*
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
    get_addresses await: GetAddresses = "/get_addresses",
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
    submit_transaction await: SubmitTransaction = "/submit_transaction",

    make_offer await: MakeOffer = "/make_offer",
    take_offer await: TakeOffer = "/take_offer",
    view_offer await: ViewOffer = "/view_offer",

    get_peers await: GetPeers = "/get_peers",
    remove_peer await: RemovePeer = "/remove_peer",
    add_peer await: AddPeer = "/add_peer",
    set_discover_peers await: SetDiscoverPeers = "/set_discover_peers",
    set_target_peers await: SetTargetPeers = "/set_target_peers",
    set_network_id await: SetNetworkId = "/set_network_id",
    set_derive_automatically: SetDeriveAutomatically = "/set_derive_automatically",
    set_derivation_batch_size: SetDerivationBatchSize = "/set_derivation_batch_size",

    remove_cat await: RemoveCat = "/remove_cat",
    update_cat await: UpdateCat = "/update_cat",
    update_did await: UpdateDid = "/update_did",
    update_nft await: UpdateNft = "/update_nft",
);

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

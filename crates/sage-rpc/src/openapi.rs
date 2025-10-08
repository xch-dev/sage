use indexmap::IndexMap;
use sage_api::OpenApiMetadata;
use utoipa::openapi::{
    path::{HttpMethod, OperationBuilder, PathItemBuilder},
    request_body::RequestBodyBuilder,
    response::ResponseBuilder,
    schema::{ObjectBuilder, Schema, SchemaType, Type},
    ComponentsBuilder, ContentBuilder, InfoBuilder, OpenApi, PathsBuilder, RefOr, ResponsesBuilder,
};

/// Generates the `OpenAPI` specification for all RPC endpoints
/// Dynamically reads from endpoints.json at compile time
pub fn generate_openapi() -> OpenApi {
    // Read endpoints at compile time from the same JSON file used by the macro
    let endpoints: IndexMap<String, bool> =
        serde_json::from_str(include_str!("../../sage-api/endpoints.json"))
            .expect("Failed to parse endpoints.json");

    let mut paths_builder = PathsBuilder::new();

    for endpoint in endpoints.keys() {
        let path_item = create_endpoint_path(endpoint);
        paths_builder = paths_builder.path(format!("/{endpoint}"), path_item);
    }

    let mut openapi = OpenApi::new(
        InfoBuilder::new()
            .title("Sage Wallet RPC API")
            .version(env!("CARGO_PKG_VERSION"))
            .description(Some(
                "RPC API for Sage wallet. All endpoints accept JSON request bodies and return JSON responses.\n\n\
                 **Authentication**: All endpoints (except those in the Authentication & Keys category) require a logged-in wallet session.",
            ))
            .build(),
        paths_builder.build(),
    );

    // Add schemas for documented types
    // TO ADD A NEW ENDPOINT: Add TypeName and TypeNameResponse here
    let mut components = ComponentsBuilder::new();
    components = components
        // Common types
        .schema_from::<sage_api::Amount>()
        .schema_from::<sage_api::Unit>()
        // Supporting types and enums
        .schema_from::<sage_api::CoinRecord>()
        .schema_from::<sage_api::TokenRecord>()
        .schema_from::<sage_api::DidRecord>()
        .schema_from::<sage_api::NftRecord>()
        .schema_from::<sage_api::NftCollectionRecord>()
        .schema_from::<sage_api::OptionRecord>()
        .schema_from::<sage_api::TransactionRecord>()
        .schema_from::<sage_api::PendingTransactionRecord>()
        .schema_from::<sage_api::DerivationRecord>()
        .schema_from::<sage_api::PeerRecord>()
        .schema_from::<sage_api::KeyInfo>()
        .schema_from::<sage_api::SecretKeyInfo>()
        .schema_from::<sage_api::KeyKind>()
        .schema_from::<sage_api::NftData>()
        .schema_from::<sage_api::NftSpecialUseType>()
        .schema_from::<sage_api::Asset>()
        .schema_from::<sage_api::AssetKind>()
        .schema_from::<sage_api::AddressKind>()
        .schema_from::<sage_api::CoinSortMode>()
        .schema_from::<sage_api::CoinFilterMode>()
        .schema_from::<sage_api::OptionSortMode>()
        .schema_from::<sage_api::NftSortMode>()
        .schema_from::<sage_api::NftUriKind>()
        .schema_from::<sage_api::TransactionSummary>()
        .schema_from::<sage_api::TransactionInput>()
        .schema_from::<sage_api::TransactionOutput>()
        .schema_from::<sage_api::TransactionCoinRecord>()
        .schema_from::<sage_api::CoinSpendJson>()
        .schema_from::<sage_api::SpendBundleJson>()
        .schema_from::<sage_api::CoinJson>()
        .schema_from::<sage_api::OfferRecord>()
        .schema_from::<sage_api::OfferRecordStatus>()
        .schema_from::<sage_api::OfferSummary>()
        .schema_from::<sage_api::OfferAsset>()
        .schema_from::<sage_api::NftRoyalty>()
        .schema_from::<sage_api::OptionAssets>()
        .schema_from::<sage_api::Payment>()
        .schema_from::<sage_api::NftMint>()
        .schema_from::<sage_api::OfferAmount>()
        .schema_from::<sage_api::OptionAsset>()
        .schema_from::<sage_api::NetworkKind>()
        // Endpoints
        .schema_from::<sage_api::Login>()
        .schema_from::<sage_api::LoginResponse>()
        .schema_from::<sage_api::Logout>()
        .schema_from::<sage_api::LogoutResponse>()
        .schema_from::<sage_api::GenerateMnemonic>()
        .schema_from::<sage_api::GenerateMnemonicResponse>()
        .schema_from::<sage_api::ImportKey>()
        .schema_from::<sage_api::ImportKeyResponse>()
        .schema_from::<sage_api::DeleteDatabase>()
        .schema_from::<sage_api::DeleteDatabaseResponse>()
        .schema_from::<sage_api::DeleteKey>()
        .schema_from::<sage_api::DeleteKeyResponse>()
        .schema_from::<sage_api::RenameKey>()
        .schema_from::<sage_api::RenameKeyResponse>()
        .schema_from::<sage_api::SetWalletEmoji>()
        .schema_from::<sage_api::SetWalletEmojiResponse>()
        .schema_from::<sage_api::GetKeys>()
        .schema_from::<sage_api::GetKeysResponse>()
        .schema_from::<sage_api::GetKey>()
        .schema_from::<sage_api::GetKeyResponse>()
        .schema_from::<sage_api::GetSecretKey>()
        .schema_from::<sage_api::GetSecretKeyResponse>()
        .schema_from::<sage_api::Resync>()
        .schema_from::<sage_api::ResyncResponse>()
        .schema_from::<sage_api::ResyncCat>()
        .schema_from::<sage_api::ResyncCatResponse>()
        .schema_from::<sage_api::UpdateCat>()
        .schema_from::<sage_api::UpdateCatResponse>()
        .schema_from::<sage_api::UpdateOption>()
        .schema_from::<sage_api::UpdateOptionResponse>()
        .schema_from::<sage_api::UpdateDid>()
        .schema_from::<sage_api::UpdateDidResponse>()
        .schema_from::<sage_api::UpdateNft>()
        .schema_from::<sage_api::UpdateNftResponse>()
        .schema_from::<sage_api::UpdateNftCollection>()
        .schema_from::<sage_api::UpdateNftCollectionResponse>()
        .schema_from::<sage_api::RedownloadNft>()
        .schema_from::<sage_api::RedownloadNftResponse>()
        .schema_from::<sage_api::IncreaseDerivationIndex>()
        .schema_from::<sage_api::IncreaseDerivationIndexResponse>()
        .schema_from::<sage_api::GetPeers>()
        .schema_from::<sage_api::GetPeersResponse>()
        .schema_from::<sage_api::RemovePeer>()
        .schema_from::<sage_api::EmptyResponse>()
        .schema_from::<sage_api::AddPeer>()
        .schema_from::<sage_api::SetDiscoverPeers>()
        .schema_from::<sage_api::SetTargetPeers>()
        .schema_from::<sage_api::SetNetwork>()
        .schema_from::<sage_api::SetNetworkOverride>()
        .schema_from::<sage_api::GetNetworks>()
        .schema_from::<sage_api::GetNetwork>()
        .schema_from::<sage_api::SetDeltaSync>()
        .schema_from::<sage_api::SetDeltaSyncOverride>()
        .schema_from::<sage_api::SetChangeAddress>()
        .schema_from::<sage_api::CheckAddress>()
        .schema_from::<sage_api::CheckAddressResponse>()
        .schema_from::<sage_api::GetDerivations>()
        .schema_from::<sage_api::GetDerivationsResponse>()
        .schema_from::<sage_api::PerformDatabaseMaintenance>()
        .schema_from::<sage_api::PerformDatabaseMaintenanceResponse>()
        .schema_from::<sage_api::GetDatabaseStats>()
        .schema_from::<sage_api::GetDatabaseStatsResponse>()
        .schema_from::<sage_api::GetSyncStatus>()
        .schema_from::<sage_api::GetSyncStatusResponse>()
        .schema_from::<sage_api::GetVersion>()
        .schema_from::<sage_api::GetVersionResponse>()
        .schema_from::<sage_api::GetAreCoinsSpendable>()
        .schema_from::<sage_api::GetAreCoinsSpendableResponse>()
        .schema_from::<sage_api::GetSpendableCoinCount>()
        .schema_from::<sage_api::GetSpendableCoinCountResponse>()
        .schema_from::<sage_api::GetCoins>()
        .schema_from::<sage_api::GetCoinsResponse>()
        .schema_from::<sage_api::GetCoinsByIds>()
        .schema_from::<sage_api::GetCoinsByIdsResponse>()
        .schema_from::<sage_api::GetAllCats>()
        .schema_from::<sage_api::GetAllCatsResponse>()
        .schema_from::<sage_api::GetCats>()
        .schema_from::<sage_api::GetCatsResponse>()
        .schema_from::<sage_api::GetToken>()
        .schema_from::<sage_api::GetTokenResponse>()
        .schema_from::<sage_api::GetDids>()
        .schema_from::<sage_api::GetDidsResponse>()
        .schema_from::<sage_api::GetMinterDidIds>()
        .schema_from::<sage_api::GetMinterDidIdsResponse>()
        .schema_from::<sage_api::GetOptions>()
        .schema_from::<sage_api::GetOptionsResponse>()
        .schema_from::<sage_api::GetOption>()
        .schema_from::<sage_api::GetOptionResponse>()
        .schema_from::<sage_api::GetPendingTransactions>()
        .schema_from::<sage_api::GetPendingTransactionsResponse>()
        .schema_from::<sage_api::GetTransaction>()
        .schema_from::<sage_api::GetTransactionResponse>()
        .schema_from::<sage_api::GetTransactions>()
        .schema_from::<sage_api::GetTransactionsResponse>()
        .schema_from::<sage_api::GetNftCollections>()
        .schema_from::<sage_api::GetNftCollectionsResponse>()
        .schema_from::<sage_api::GetNftCollection>()
        .schema_from::<sage_api::GetNftCollectionResponse>()
        .schema_from::<sage_api::GetNfts>()
        .schema_from::<sage_api::GetNftsResponse>()
        .schema_from::<sage_api::GetNft>()
        .schema_from::<sage_api::GetNftResponse>()
        .schema_from::<sage_api::GetNftIcon>()
        .schema_from::<sage_api::GetNftIconResponse>()
        .schema_from::<sage_api::GetNftThumbnail>()
        .schema_from::<sage_api::GetNftThumbnailResponse>()
        .schema_from::<sage_api::GetNftData>()
        .schema_from::<sage_api::GetNftDataResponse>()
        .schema_from::<sage_api::IsAssetOwned>()
        .schema_from::<sage_api::IsAssetOwnedResponse>()
        // Transactions
        .schema_from::<sage_api::SendXch>()
        .schema_from::<sage_api::BulkSendXch>()
        .schema_from::<sage_api::Combine>()
        .schema_from::<sage_api::Split>()
        .schema_from::<sage_api::AutoCombineXch>()
        .schema_from::<sage_api::AutoCombineXchResponse>()
        .schema_from::<sage_api::AutoCombineCat>()
        .schema_from::<sage_api::AutoCombineCatResponse>()
        .schema_from::<sage_api::IssueCat>()
        .schema_from::<sage_api::SendCat>()
        .schema_from::<sage_api::BulkSendCat>()
        .schema_from::<sage_api::MultiSend>()
        .schema_from::<sage_api::CreateDid>()
        .schema_from::<sage_api::BulkMintNfts>()
        .schema_from::<sage_api::BulkMintNftsResponse>()
        .schema_from::<sage_api::TransferNfts>()
        .schema_from::<sage_api::AddNftUri>()
        .schema_from::<sage_api::AssignNftsToDid>()
        .schema_from::<sage_api::TransferDids>()
        .schema_from::<sage_api::NormalizeDids>()
        .schema_from::<sage_api::MintOption>()
        .schema_from::<sage_api::MintOptionResponse>()
        .schema_from::<sage_api::ExerciseOptions>()
        .schema_from::<sage_api::TransferOptions>()
        .schema_from::<sage_api::SignCoinSpends>()
        .schema_from::<sage_api::SignCoinSpendsResponse>()
        .schema_from::<sage_api::ViewCoinSpends>()
        .schema_from::<sage_api::ViewCoinSpendsResponse>()
        .schema_from::<sage_api::SubmitTransaction>()
        .schema_from::<sage_api::SubmitTransactionResponse>()
        .schema_from::<sage_api::TransactionResponse>()
        // Offers
        .schema_from::<sage_api::MakeOffer>()
        .schema_from::<sage_api::MakeOfferResponse>()
        .schema_from::<sage_api::TakeOffer>()
        .schema_from::<sage_api::TakeOfferResponse>()
        .schema_from::<sage_api::CombineOffers>()
        .schema_from::<sage_api::CombineOffersResponse>()
        .schema_from::<sage_api::ViewOffer>()
        .schema_from::<sage_api::ViewOfferResponse>()
        .schema_from::<sage_api::ImportOffer>()
        .schema_from::<sage_api::ImportOfferResponse>()
        .schema_from::<sage_api::GetOffers>()
        .schema_from::<sage_api::GetOffersResponse>()
        .schema_from::<sage_api::GetOffersForAsset>()
        .schema_from::<sage_api::GetOffersForAssetResponse>()
        .schema_from::<sage_api::GetOffer>()
        .schema_from::<sage_api::GetOfferResponse>()
        .schema_from::<sage_api::DeleteOffer>()
        .schema_from::<sage_api::DeleteOfferResponse>()
        .schema_from::<sage_api::CancelOffer>()
        .schema_from::<sage_api::CancelOffers>();

    // Add common error schema
    components = components.schema(
        "Error",
        Schema::Object(
            ObjectBuilder::new()
                .schema_type(SchemaType::Type(Type::Object))
                .property(
                    "error",
                    ObjectBuilder::new()
                        .schema_type(SchemaType::Type(Type::String))
                        .description(Some("Error message describing what went wrong"))
                        .build(),
                )
                .required("error")
                .build(),
        ),
    );

    openapi.components = Some(components.build());
    openapi
}

fn create_endpoint_path(endpoint: &str) -> utoipa::openapi::path::PathItem {
    let (tag, description) = get_endpoint_metadata(endpoint);
    let request_schema = get_request_schema_ref(endpoint);
    let response_schema = get_response_schema_ref(endpoint);

    let operation = OperationBuilder::new()
        .tag(tag)
        .summary(Some(format_endpoint_name(endpoint)))
        .description(Some(description))
        .request_body(Some(
            RequestBodyBuilder::new()
                .description(Some(format!(
                    "Request parameters for the {endpoint} endpoint"
                )))
                .content(
                    "application/json",
                    ContentBuilder::new().schema(Some(request_schema)).build(),
                )
                .required(Some(utoipa::openapi::Required::True))
                .build(),
        ))
        .responses(create_endpoint_responses_with_schema(
            endpoint,
            response_schema,
        ))
        .build();

    PathItemBuilder::new()
        .operation(HttpMethod::Post, operation)
        .build()
}

// TO ADD A NEW ENDPOINT: Add 1 match arm here (4 lines)
fn get_endpoint_metadata(endpoint: &str) -> (&'static str, String) {
    match endpoint {
        "login" => (
            sage_api::Login::openapi_tag(),
            sage_api::Login::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "logout" => (
            sage_api::Logout::openapi_tag(),
            sage_api::Logout::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "generate_mnemonic" => (
            sage_api::GenerateMnemonic::openapi_tag(),
            sage_api::GenerateMnemonic::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "import_key" => (
            sage_api::ImportKey::openapi_tag(),
            sage_api::ImportKey::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "delete_database" => (
            sage_api::DeleteDatabase::openapi_tag(),
            sage_api::DeleteDatabase::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "delete_key" => (
            sage_api::DeleteKey::openapi_tag(),
            sage_api::DeleteKey::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "rename_key" => (
            sage_api::RenameKey::openapi_tag(),
            sage_api::RenameKey::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "set_wallet_emoji" => (
            sage_api::SetWalletEmoji::openapi_tag(),
            sage_api::SetWalletEmoji::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_keys" => (
            sage_api::GetKeys::openapi_tag(),
            sage_api::GetKeys::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_key" => (
            sage_api::GetKey::openapi_tag(),
            sage_api::GetKey::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_secret_key" => (
            sage_api::GetSecretKey::openapi_tag(),
            sage_api::GetSecretKey::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "resync" => (
            sage_api::Resync::openapi_tag(),
            sage_api::Resync::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "resync_cat" => (
            sage_api::ResyncCat::openapi_tag(),
            sage_api::ResyncCat::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "update_cat" => (
            sage_api::UpdateCat::openapi_tag(),
            sage_api::UpdateCat::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "update_option" => (
            sage_api::UpdateOption::openapi_tag(),
            sage_api::UpdateOption::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "update_did" => (
            sage_api::UpdateDid::openapi_tag(),
            sage_api::UpdateDid::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "update_nft" => (
            sage_api::UpdateNft::openapi_tag(),
            sage_api::UpdateNft::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "update_nft_collection" => (
            sage_api::UpdateNftCollection::openapi_tag(),
            sage_api::UpdateNftCollection::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "redownload_nft" => (
            sage_api::RedownloadNft::openapi_tag(),
            sage_api::RedownloadNft::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "increase_derivation_index" => (
            sage_api::IncreaseDerivationIndex::openapi_tag(),
            sage_api::IncreaseDerivationIndex::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_peers" => (
            sage_api::GetPeers::openapi_tag(),
            sage_api::GetPeers::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "remove_peer" => (
            sage_api::RemovePeer::openapi_tag(),
            sage_api::RemovePeer::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "add_peer" => (
            sage_api::AddPeer::openapi_tag(),
            sage_api::AddPeer::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "set_discover_peers" => (
            sage_api::SetDiscoverPeers::openapi_tag(),
            sage_api::SetDiscoverPeers::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "set_target_peers" => (
            sage_api::SetTargetPeers::openapi_tag(),
            sage_api::SetTargetPeers::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "set_network" => (
            sage_api::SetNetwork::openapi_tag(),
            sage_api::SetNetwork::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "set_network_override" => (
            sage_api::SetNetworkOverride::openapi_tag(),
            sage_api::SetNetworkOverride::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_networks" => (
            sage_api::GetNetworks::openapi_tag(),
            sage_api::GetNetworks::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_network" => (
            sage_api::GetNetwork::openapi_tag(),
            sage_api::GetNetwork::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "set_delta_sync" => (
            sage_api::SetDeltaSync::openapi_tag(),
            sage_api::SetDeltaSync::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "set_delta_sync_override" => (
            sage_api::SetDeltaSyncOverride::openapi_tag(),
            sage_api::SetDeltaSyncOverride::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "set_change_address" => (
            sage_api::SetChangeAddress::openapi_tag(),
            sage_api::SetChangeAddress::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "check_address" => (
            sage_api::CheckAddress::openapi_tag(),
            sage_api::CheckAddress::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_derivations" => (
            sage_api::GetDerivations::openapi_tag(),
            sage_api::GetDerivations::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "perform_database_maintenance" => (
            sage_api::PerformDatabaseMaintenance::openapi_tag(),
            sage_api::PerformDatabaseMaintenance::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_database_stats" => (
            sage_api::GetDatabaseStats::openapi_tag(),
            sage_api::GetDatabaseStats::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_sync_status" => (
            sage_api::GetSyncStatus::openapi_tag(),
            sage_api::GetSyncStatus::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_version" => (
            sage_api::GetVersion::openapi_tag(),
            sage_api::GetVersion::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_are_coins_spendable" => (
            sage_api::GetAreCoinsSpendable::openapi_tag(),
            sage_api::GetAreCoinsSpendable::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_spendable_coin_count" => (
            sage_api::GetSpendableCoinCount::openapi_tag(),
            sage_api::GetSpendableCoinCount::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_coins" => (
            sage_api::GetCoins::openapi_tag(),
            sage_api::GetCoins::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_coins_by_ids" => (
            sage_api::GetCoinsByIds::openapi_tag(),
            sage_api::GetCoinsByIds::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_all_cats" => (
            sage_api::GetAllCats::openapi_tag(),
            sage_api::GetAllCats::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_cats" => (
            sage_api::GetCats::openapi_tag(),
            sage_api::GetCats::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_token" => (
            sage_api::GetToken::openapi_tag(),
            sage_api::GetToken::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_dids" => (
            sage_api::GetDids::openapi_tag(),
            sage_api::GetDids::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_minter_did_ids" => (
            sage_api::GetMinterDidIds::openapi_tag(),
            sage_api::GetMinterDidIds::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_options" => (
            sage_api::GetOptions::openapi_tag(),
            sage_api::GetOptions::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_option" => (
            sage_api::GetOption::openapi_tag(),
            sage_api::GetOption::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_pending_transactions" => (
            sage_api::GetPendingTransactions::openapi_tag(),
            sage_api::GetPendingTransactions::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_transaction" => (
            sage_api::GetTransaction::openapi_tag(),
            sage_api::GetTransaction::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_transactions" => (
            sage_api::GetTransactions::openapi_tag(),
            sage_api::GetTransactions::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_nft_collections" => (
            sage_api::GetNftCollections::openapi_tag(),
            sage_api::GetNftCollections::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_nft_collection" => (
            sage_api::GetNftCollection::openapi_tag(),
            sage_api::GetNftCollection::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_nfts" => (
            sage_api::GetNfts::openapi_tag(),
            sage_api::GetNfts::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_nft" => (
            sage_api::GetNft::openapi_tag(),
            sage_api::GetNft::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_nft_icon" => (
            sage_api::GetNftIcon::openapi_tag(),
            sage_api::GetNftIcon::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_nft_thumbnail" => (
            sage_api::GetNftThumbnail::openapi_tag(),
            sage_api::GetNftThumbnail::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_nft_data" => (
            sage_api::GetNftData::openapi_tag(),
            sage_api::GetNftData::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "is_asset_owned" => (
            sage_api::IsAssetOwned::openapi_tag(),
            sage_api::IsAssetOwned::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "send_xch" => (
            sage_api::SendXch::openapi_tag(),
            sage_api::SendXch::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "bulk_send_xch" => (
            sage_api::BulkSendXch::openapi_tag(),
            sage_api::BulkSendXch::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "combine" => (
            sage_api::Combine::openapi_tag(),
            sage_api::Combine::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "split" => (
            sage_api::Split::openapi_tag(),
            sage_api::Split::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "auto_combine_xch" => (
            sage_api::AutoCombineXch::openapi_tag(),
            sage_api::AutoCombineXch::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "auto_combine_cat" => (
            sage_api::AutoCombineCat::openapi_tag(),
            sage_api::AutoCombineCat::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "issue_cat" => (
            sage_api::IssueCat::openapi_tag(),
            sage_api::IssueCat::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "send_cat" => (
            sage_api::SendCat::openapi_tag(),
            sage_api::SendCat::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "bulk_send_cat" => (
            sage_api::BulkSendCat::openapi_tag(),
            sage_api::BulkSendCat::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "multi_send" => (
            sage_api::MultiSend::openapi_tag(),
            sage_api::MultiSend::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "create_did" => (
            sage_api::CreateDid::openapi_tag(),
            sage_api::CreateDid::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "bulk_mint_nfts" => (
            sage_api::BulkMintNfts::openapi_tag(),
            sage_api::BulkMintNfts::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "transfer_nfts" => (
            sage_api::TransferNfts::openapi_tag(),
            sage_api::TransferNfts::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "add_nft_uri" => (
            sage_api::AddNftUri::openapi_tag(),
            sage_api::AddNftUri::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "assign_nfts_to_did" => (
            sage_api::AssignNftsToDid::openapi_tag(),
            sage_api::AssignNftsToDid::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "transfer_dids" => (
            sage_api::TransferDids::openapi_tag(),
            sage_api::TransferDids::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "normalize_dids" => (
            sage_api::NormalizeDids::openapi_tag(),
            sage_api::NormalizeDids::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "mint_option" => (
            sage_api::MintOption::openapi_tag(),
            sage_api::MintOption::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "exercise_options" => (
            sage_api::ExerciseOptions::openapi_tag(),
            sage_api::ExerciseOptions::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "transfer_options" => (
            sage_api::TransferOptions::openapi_tag(),
            sage_api::TransferOptions::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "sign_coin_spends" => (
            sage_api::SignCoinSpends::openapi_tag(),
            sage_api::SignCoinSpends::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "view_coin_spends" => (
            sage_api::ViewCoinSpends::openapi_tag(),
            sage_api::ViewCoinSpends::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "submit_transaction" => (
            sage_api::SubmitTransaction::openapi_tag(),
            sage_api::SubmitTransaction::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "make_offer" => (
            sage_api::MakeOffer::openapi_tag(),
            sage_api::MakeOffer::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "take_offer" => (
            sage_api::TakeOffer::openapi_tag(),
            sage_api::TakeOffer::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "combine_offers" => (
            sage_api::CombineOffers::openapi_tag(),
            sage_api::CombineOffers::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "view_offer" => (
            sage_api::ViewOffer::openapi_tag(),
            sage_api::ViewOffer::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "import_offer" => (
            sage_api::ImportOffer::openapi_tag(),
            sage_api::ImportOffer::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_offers" => (
            sage_api::GetOffers::openapi_tag(),
            sage_api::GetOffers::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_offers_for_asset" => (
            sage_api::GetOffersForAsset::openapi_tag(),
            sage_api::GetOffersForAsset::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "get_offer" => (
            sage_api::GetOffer::openapi_tag(),
            sage_api::GetOffer::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "delete_offer" => (
            sage_api::DeleteOffer::openapi_tag(),
            sage_api::DeleteOffer::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "cancel_offer" => (
            sage_api::CancelOffer::openapi_tag(),
            sage_api::CancelOffer::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        "cancel_offers" => (
            sage_api::CancelOffers::openapi_tag(),
            sage_api::CancelOffers::openapi_description()
                .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
        ),
        _ => {
            let tags = get_endpoint_tags(endpoint);
            (tags[0], get_endpoint_description(endpoint))
        }
    }
}

// TO ADD A NEW ENDPOINT: Add 1 match arm here (1 line)
fn get_request_schema_ref(endpoint: &str) -> RefOr<Schema> {
    match endpoint {
        "login" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/Login")),
        "logout" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/Logout")),
        "generate_mnemonic" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GenerateMnemonic",
        )),
        "import_key" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/ImportKey")),
        "delete_database" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/DeleteDatabase",
        )),
        "delete_key" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/DeleteKey")),
        "rename_key" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/RenameKey")),
        "set_wallet_emoji" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/SetWalletEmoji",
        )),
        "get_keys" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/GetKeys")),
        "get_key" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/GetKey")),
        "get_secret_key" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetSecretKey",
        )),
        "resync" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/Resync")),
        "resync_cat" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/ResyncCat")),
        "update_cat" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/UpdateCat")),
        "update_option" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/UpdateOption",
        )),
        "update_did" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/UpdateDid")),
        "update_nft" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/UpdateNft")),
        "update_nft_collection" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/UpdateNftCollection",
        )),
        "redownload_nft" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/RedownloadNft",
        )),
        "increase_derivation_index" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/IncreaseDerivationIndex",
        )),
        "get_peers" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/GetPeers")),
        "remove_peer" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/RemovePeer")),
        "add_peer" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/AddPeer")),
        "set_discover_peers" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/SetDiscoverPeers",
        )),
        "set_target_peers" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/SetTargetPeers",
        )),
        "set_network" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/SetNetwork")),
        "set_network_override" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/SetNetworkOverride",
        )),
        "get_networks" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetNetworks",
        )),
        "get_network" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/GetNetwork")),
        "set_delta_sync" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/SetDeltaSync",
        )),
        "set_delta_sync_override" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/SetDeltaSyncOverride",
        )),
        "set_change_address" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/SetChangeAddress",
        )),
        "check_address" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/CheckAddress",
        )),
        "get_derivations" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetDerivations",
        )),
        "perform_database_maintenance" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/PerformDatabaseMaintenance",
        )),
        "get_database_stats" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetDatabaseStats",
        )),
        "get_sync_status" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetSyncStatus",
        )),
        "get_version" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/GetVersion")),
        "get_are_coins_spendable" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetAreCoinsSpendable",
        )),
        "get_spendable_coin_count" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetSpendableCoinCount",
        )),
        "get_coins" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/GetCoins")),
        "get_coins_by_ids" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetCoinsByIds",
        )),
        "get_all_cats" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/GetAllCats")),
        "get_cats" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/GetCats")),
        "get_token" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/GetToken")),
        "get_dids" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/GetDids")),
        "get_minter_did_ids" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetMinterDidIds",
        )),
        "get_options" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/GetOptions")),
        "get_option" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/GetOption")),
        "get_pending_transactions" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetPendingTransactions",
        )),
        "get_transaction" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetTransaction",
        )),
        "get_transactions" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetTransactions",
        )),
        "get_nft_collections" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetNftCollections",
        )),
        "get_nft_collection" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetNftCollection",
        )),
        "get_nfts" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/GetNfts")),
        "get_nft" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/GetNft")),
        "get_nft_icon" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/GetNftIcon")),
        "get_nft_thumbnail" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetNftThumbnail",
        )),
        "get_nft_data" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/GetNftData")),
        "is_asset_owned" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/IsAssetOwned",
        )),
        "send_xch" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/SendXch")),
        "bulk_send_xch" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/BulkSendXch",
        )),
        "combine" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/Combine")),
        "split" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/Split")),
        "auto_combine_xch" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/AutoCombineXch",
        )),
        "auto_combine_cat" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/AutoCombineCat",
        )),
        "issue_cat" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/IssueCat")),
        "send_cat" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/SendCat")),
        "bulk_send_cat" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/BulkSendCat",
        )),
        "multi_send" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/MultiSend")),
        "create_did" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/CreateDid")),
        "bulk_mint_nfts" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/BulkMintNfts",
        )),
        "transfer_nfts" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/TransferNfts",
        )),
        "add_nft_uri" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/AddNftUri")),
        "assign_nfts_to_did" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/AssignNftsToDid",
        )),
        "transfer_dids" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/TransferDids",
        )),
        "normalize_dids" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/NormalizeDids",
        )),
        "mint_option" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/MintOption")),
        "exercise_options" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/ExerciseOptions",
        )),
        "transfer_options" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/TransferOptions",
        )),
        "sign_coin_spends" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/SignCoinSpends",
        )),
        "view_coin_spends" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/ViewCoinSpends",
        )),
        "submit_transaction" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/SubmitTransaction",
        )),
        "make_offer" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/MakeOffer")),
        "take_offer" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/TakeOffer")),
        "combine_offers" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/CombineOffers",
        )),
        "view_offer" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/ViewOffer")),
        "import_offer" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/ImportOffer",
        )),
        "get_offers" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/GetOffers")),
        "get_offers_for_asset" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetOffersForAsset",
        )),
        "get_offer" => RefOr::Ref(utoipa::openapi::Ref::new("#/components/schemas/GetOffer")),
        "delete_offer" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/DeleteOffer",
        )),
        "cancel_offer" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/CancelOffer",
        )),
        "cancel_offers" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/CancelOffers",
        )),
        _ => create_generic_schema(&format!("Request body for {endpoint} endpoint")),
    }
}

// TO ADD A NEW ENDPOINT: Add 1 match arm here (1 line)
fn get_response_schema_ref(endpoint: &str) -> RefOr<Schema> {
    match endpoint {
        "login" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/LoginResponse",
        )),
        "logout" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/LogoutResponse",
        )),
        "generate_mnemonic" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GenerateMnemonicResponse",
        )),
        "import_key" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/ImportKeyResponse",
        )),
        "delete_database" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/DeleteDatabaseResponse",
        )),
        "delete_key" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/DeleteKeyResponse",
        )),
        "rename_key" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/RenameKeyResponse",
        )),
        "set_wallet_emoji" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/SetWalletEmojiResponse",
        )),
        "get_keys" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetKeysResponse",
        )),
        "get_key" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetKeyResponse",
        )),
        "get_secret_key" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetSecretKeyResponse",
        )),
        "resync" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/ResyncResponse",
        )),
        "resync_cat" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/ResyncCatResponse",
        )),
        "update_cat" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/UpdateCatResponse",
        )),
        "update_option" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/UpdateOptionResponse",
        )),
        "update_did" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/UpdateDidResponse",
        )),
        "update_nft" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/UpdateNftResponse",
        )),
        "update_nft_collection" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/UpdateNftCollectionResponse",
        )),
        "redownload_nft" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/RedownloadNftResponse",
        )),
        "increase_derivation_index" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/IncreaseDerivationIndexResponse",
        )),
        "get_peers" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetPeersResponse",
        )),
        "remove_peer"
        | "add_peer"
        | "set_discover_peers"
        | "set_target_peers"
        | "set_network"
        | "set_network_override"
        | "set_delta_sync"
        | "set_delta_sync_override"
        | "set_change_address" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/EmptyResponse",
        )),
        "get_networks" => create_generic_schema("List of available networks"),
        "get_network" => create_generic_schema("Current network information"),
        "check_address" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/CheckAddressResponse",
        )),
        "get_derivations" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetDerivationsResponse",
        )),
        "perform_database_maintenance" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/PerformDatabaseMaintenanceResponse",
        )),
        "get_database_stats" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetDatabaseStatsResponse",
        )),
        "get_sync_status" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetSyncStatusResponse",
        )),
        "get_version" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetVersionResponse",
        )),
        "get_are_coins_spendable" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetAreCoinsSpendableResponse",
        )),
        "get_spendable_coin_count" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetSpendableCoinCountResponse",
        )),
        "get_coins" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetCoinsResponse",
        )),
        "get_coins_by_ids" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetCoinsByIdsResponse",
        )),
        "get_all_cats" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetAllCatsResponse",
        )),
        "get_cats" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetCatsResponse",
        )),
        "get_token" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetTokenResponse",
        )),
        "get_dids" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetDidsResponse",
        )),
        "get_minter_did_ids" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetMinterDidIdsResponse",
        )),
        "get_options" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetOptionsResponse",
        )),
        "get_option" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetOptionResponse",
        )),
        "get_pending_transactions" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetPendingTransactionsResponse",
        )),
        "get_transaction" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetTransactionResponse",
        )),
        "get_transactions" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetTransactionsResponse",
        )),
        "get_nft_collections" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetNftCollectionsResponse",
        )),
        "get_nft_collection" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetNftCollectionResponse",
        )),
        "get_nfts" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetNftsResponse",
        )),
        "get_nft" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetNftResponse",
        )),
        "get_nft_icon" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetNftIconResponse",
        )),
        "get_nft_thumbnail" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetNftThumbnailResponse",
        )),
        "get_nft_data" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetNftDataResponse",
        )),
        "is_asset_owned" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/IsAssetOwnedResponse",
        )),
        "send_xch" | "bulk_send_xch" | "combine" | "split" | "issue_cat" | "send_cat"
        | "bulk_send_cat" | "multi_send" | "create_did" | "transfer_nfts" | "add_nft_uri"
        | "assign_nfts_to_did" | "transfer_dids" | "normalize_dids" | "exercise_options"
        | "transfer_options" | "cancel_offer" | "cancel_offers" => RefOr::Ref(
            utoipa::openapi::Ref::new("#/components/schemas/TransactionResponse"),
        ),
        "auto_combine_xch" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/AutoCombineXchResponse",
        )),
        "auto_combine_cat" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/AutoCombineCatResponse",
        )),
        "bulk_mint_nfts" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/BulkMintNftsResponse",
        )),
        "mint_option" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/MintOptionResponse",
        )),
        "sign_coin_spends" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/SignCoinSpendsResponse",
        )),
        "view_coin_spends" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/ViewCoinSpendsResponse",
        )),
        "submit_transaction" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/SubmitTransactionResponse",
        )),
        "make_offer" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/MakeOfferResponse",
        )),
        "take_offer" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/TakeOfferResponse",
        )),
        "combine_offers" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/CombineOffersResponse",
        )),
        "view_offer" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/ViewOfferResponse",
        )),
        "import_offer" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/ImportOfferResponse",
        )),
        "get_offers" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetOffersResponse",
        )),
        "get_offers_for_asset" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetOffersForAssetResponse",
        )),
        "get_offer" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/GetOfferResponse",
        )),
        "delete_offer" => RefOr::Ref(utoipa::openapi::Ref::new(
            "#/components/schemas/DeleteOfferResponse",
        )),
        _ => create_generic_schema(&format!("Response data for {endpoint} endpoint")),
    }
}

fn create_generic_schema(description: &str) -> RefOr<Schema> {
    RefOr::T(Schema::Object(
        ObjectBuilder::new()
            .schema_type(SchemaType::Type(Type::Object))
            .description(Some(description))
            .build(),
    ))
}

fn create_endpoint_responses_with_schema(
    endpoint: &str,
    schema: RefOr<Schema>,
) -> utoipa::openapi::response::Responses {
    ResponsesBuilder::new()
        .response(
            "200",
            ResponseBuilder::new()
                .description(format!("Successful response from {endpoint}"))
                .content(
                    "application/json",
                    ContentBuilder::new()
                        .schema(Some(schema))
                        .build(),
                )
                .build(),
        )
        .response(
            "400",
            ResponseBuilder::new()
                .description("Bad request - invalid parameters or malformed JSON")
                .build(),
        )
        .response(
            "401",
            ResponseBuilder::new()
                .description("Unauthorized - no wallet is currently logged in, or authentication is required")
                .build(),
        )
        .response(
            "404",
            ResponseBuilder::new()
                .description("Not found - requested resource doesn't exist (e.g., NFT, offer, transaction)")
                .build(),
        )
        .response(
            "500",
            ResponseBuilder::new()
                .description("Internal server error - unexpected failure in wallet operations")
                .build(),
        )
        .build()
}

fn format_endpoint_name(endpoint: &str) -> String {
    endpoint
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn get_endpoint_tags(endpoint: &str) -> Vec<&'static str> {
    match endpoint {
        "get_user_themes" | "get_user_theme" | "save_user_theme" | "delete_user_theme" => {
            vec!["Themes"]
        }
        _ => vec!["General"],
    }
}

fn get_endpoint_description(endpoint: &str) -> String {
    format!("Endpoint for {endpoint}")
}

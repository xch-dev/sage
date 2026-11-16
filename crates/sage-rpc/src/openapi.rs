use indexmap::IndexMap;
use utoipa::openapi::{
    ComponentsBuilder, ContentBuilder, InfoBuilder, OpenApi, PathsBuilder, RefOr, ResponsesBuilder,
    path::{HttpMethod, OperationBuilder, PathItemBuilder},
    request_body::RequestBodyBuilder,
    response::ResponseBuilder,
    schema::{ObjectBuilder, Schema, SchemaType, Type},
    tag::TagBuilder,
};

/// The categories endpoints are grouped into, in the order they're shown, with their descriptions
const TAGS: &[(&str, &str)] = &[
    (
        "Wallets",
        "Create, import, log into and manage wallets, including watch-only wallets.",
    ),
    (
        "Keys",
        "Generate mnemonics and manage the keys in the keychain.",
    ),
    (
        "Addresses",
        "Check, derive and configure the active wallet's addresses.",
    ),
    (
        "Coins",
        "Look up the active wallet's coins, and combine or split them.",
    ),
    ("Payments", "Send XCH and CATs, and settle clawbacks."),
    (
        "Tokens",
        "Look up XCH and CAT balances and metadata, and issue new CATs.",
    ),
    (
        "NFTs",
        "Look up, mint, transfer and update NFTs and their collections.",
    ),
    ("DIDs", "Look up, create, transfer and update DIDs."),
    (
        "Options",
        "Look up, mint, transfer, exercise and cancel options.",
    ),
    ("Offers", "Make, view, take, track and cancel offers."),
    (
        "Transactions",
        "Build transactions from actions, sign and submit coin spends, and look up the \
         transaction history.",
    ),
    (
        "Address Book",
        "Manage the contacts shared by the wallets on a network.",
    ),
    ("XCHandles", "Resolve and verify XCHandles handles."),
    (
        "Network",
        "Choose the network and configure how wallets sync.",
    ),
    ("Peers", "Manage the peers Sage connects to."),
    (
        "Themes",
        "Save themes from theme NFTs and remember the selected theme.",
    ),
    (
        "System",
        "Check the version and sync status, resync wallets, maintain their databases, and get \
         the XCH price.",
    ),
];

/// Generates the `OpenAPI` specification for all RPC endpoints
/// Dynamically reads from endpoints.json at compile time
pub fn generate_openapi() -> OpenApi {
    // Read endpoints at compile time from the same JSON file used by the macro
    let endpoints: IndexMap<String, bool> =
        serde_json::from_str(include_str!("../../sage-api/endpoints.json"))
            .expect("Failed to parse endpoints.json");

    for endpoint in endpoints.keys() {
        let (tag, _) = get_endpoint_metadata(endpoint);
        assert!(
            TAGS.iter().any(|(name, _)| *name == tag),
            "The {endpoint} endpoint's tag \"{tag}\" isn't listed in TAGS"
        );
    }

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
                "RPC API for the Sage wallet. Every endpoint is a POST request with a JSON body, \
                 and returns a JSON response.\n\n\
                 **Authentication**: Sage RPC uses mutual TLS for authentication.\n\n\
                 **Active wallet**: Most endpoints act on the active wallet, which is chosen with \
                 `login`, and fail if no wallet is logged into.\n\n\
                 **Transactions**: Endpoints that create a transaction return a summary of it and \
                 its unsigned coin spends. With `auto_submit` set, it's also signed and submitted \
                 to the network. Otherwise, it can be signed with `sign_coin_spends` and submitted \
                 with `submit_transaction`. Coins aren't reserved until a transaction is submitted, so \
                 transactions built before then may select the same coins.",
            ))
            .build(),
        paths_builder.build(),
    );

    openapi.tags = Some(
        TAGS.iter()
            .map(|(name, description)| {
                TagBuilder::new()
                    .name(*name)
                    .description(Some(*description))
                    .build()
            })
            .collect(),
    );

    // Add schemas for documented types
    // Supporting types are registered manually, endpoints are auto-generated from endpoints.json
    let mut components = ComponentsBuilder::new();

    // Common types used across endpoints
    components = components
        .schema_from::<sage_api::Amount>()
        .schema_from::<sage_api::Unit>()
        .schema_from::<sage_api::CoinRecord>()
        .schema_from::<sage_api::TokenRecord>()
        .schema_from::<sage_api::DidRecord>()
        .schema_from::<sage_api::NftRecord>()
        .schema_from::<sage_api::NftCollectionRecord>()
        .schema_from::<sage_api::OptionRecord>()
        .schema_from::<sage_api::TransactionRecord>()
        .schema_from::<sage_api::PendingTransactionRecord>()
        .schema_from::<sage_api::DerivationRecord>()
        .schema_from::<sage_api::HandleRecord>()
        .schema_from::<sage_api::HandleStatus>()
        .schema_from::<sage_api::HandleVerification>()
        .schema_from::<sage_api::HandleSlotRecord>()
        .schema_from::<sage_api::PendingTransferStatus>()
        .schema_from::<sage_api::PendingTransferRecord>()
        .schema_from::<sage_api::HandleFilterMode>()
        .schema_from::<sage_api::AddressHandlesRecord>()
        .schema_from::<sage_api::HandleErrorCode>()
        .schema_from::<sage_api::ContactRecord>()
        .schema_from::<sage_api::ContactKind>()
        .schema_from::<sage_api::AddressContactsRecord>()
        .schema_from::<sage_api::ContactEntry>()
        .schema_from::<sage_api::SkippedContact>()
        .schema_from::<sage_api::ContactSkipReason>()
        .schema_from::<sage_api::HandleConfirmation>()
        .schema_from::<sage_api::RecipientError>()
        .schema_from::<sage_api::RecipientErrorKind>()
        .schema_from::<sage_api::OfferHandleNft>()
        .schema_from::<sage_api::OfferHandleStatus>()
        .schema_from::<sage_api::PeerRecord>()
        .schema_from::<sage_api::KeyInfo>()
        .schema_from::<sage_api::KeyKind>()
        .schema_from::<sage_api::SecretKeyInfo>()
        .schema_from::<sage_api::WalletInfo>()
        .schema_from::<sage_api::WalletKind>()
        .schema_from::<sage_api::NftData>()
        .schema_from::<sage_api::NftSpecialUseType>()
        .schema_from::<sage_api::NftUpdaterKind>()
        .schema_from::<sage_api::Asset>()
        .schema_from::<sage_api::AssetKind>()
        .schema_from::<sage_api::AddressKind>()
        .schema_from::<sage_api::CoinSortMode>()
        .schema_from::<sage_api::CoinFilterMode>()
        .schema_from::<sage_api::OptionSortMode>()
        .schema_from::<sage_api::OptionFilter>()
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
        .schema_from::<sage_api::Id>()
        .schema_from::<sage_api::Action>()
        .schema_from::<sage_api::SendAction>()
        .schema_from::<sage_api::MintNftAction>()
        .schema_from::<sage_api::UpdateNftAction>()
        .schema_from::<sage_api::FeeAction>()
        .schema_from::<sage_api::NewNftUri>()
        .schema_from::<sage_api::NftTransfer>();

    // Endpoints - automatically generated from endpoints.json
    components = sage_api_macro::register_openapi_types! {};

    // Special cases: Network endpoints use type aliases that don't implement ToSchema
    // Request types are registered here, responses are handled in get_response_schema_ref
    components = components
        .schema_from::<sage_api::GetNetworks>()
        .schema_from::<sage_api::GetNetwork>();

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

/// Returns (tag, description) for an endpoint
/// Metadata is automatically retrieved from `OpenApiMetadata` trait implementations
/// Match arms are auto-generated from endpoints.json at compile time
fn get_endpoint_metadata(endpoint: &str) -> (&'static str, String) {
    sage_api_macro::endpoint_metadata! {}
}

/// Returns the JSON schema reference for the request body of an endpoint
/// Match arms are auto-generated from endpoints.json at compile time
fn get_request_schema_ref(endpoint: &str) -> RefOr<Schema> {
    sage_api_macro::request_schemas! {}
}

/// Returns the JSON schema reference for the response body of an endpoint
/// Match arms are auto-generated from endpoints.json at compile time
fn get_response_schema_ref(endpoint: &str) -> RefOr<Schema> {
    sage_api_macro::response_schemas! {}
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

fn get_endpoint_tags(_endpoint: &str) -> Vec<&'static str> {
    vec!["General"]
}

fn get_endpoint_description(endpoint: &str) -> String {
    format!("Endpoint for {endpoint}")
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{TAGS, generate_openapi};

    #[test]
    fn every_tag_is_listed_and_used() {
        let openapi = generate_openapi();

        let used: HashSet<&str> = openapi
            .paths
            .paths
            .values()
            .filter_map(|item| item.post.as_ref())
            .flat_map(|operation| operation.tags.iter().flatten())
            .map(String::as_str)
            .collect();

        for (name, _) in TAGS {
            assert!(used.contains(name), "The \"{name}\" tag isn't used");
        }
    }
}

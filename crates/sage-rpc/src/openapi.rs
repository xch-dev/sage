use indexmap::IndexMap;
use std::collections::BTreeSet;
use utoipa::openapi::{
    path::{HttpMethod, OperationBuilder, PathItemBuilder},
    request_body::RequestBodyBuilder,
    response::ResponseBuilder,
    schema::{ObjectBuilder, Schema, SchemaType, Type},
    tag::TagBuilder,
    ComponentsBuilder, ContentBuilder, InfoBuilder, OpenApi, PathsBuilder, RefOr, ResponsesBuilder,
};

/// Generates the `OpenAPI` specification for all RPC endpoints
/// Dynamically reads from endpoints.json at compile time
pub fn generate_openapi() -> OpenApi {
    // Read endpoints at compile time from the same JSON file used by the macro
    let endpoints: IndexMap<String, bool> =
        serde_json::from_str(include_str!("../../sage-api/endpoints.json"))
            .expect("Failed to parse endpoints.json");

    // Collect unique tags from all endpoints (BTreeSet keeps them sorted)
    let mut tags = BTreeSet::new();
    for endpoint in endpoints.keys() {
        let (tag, _) = get_endpoint_metadata(endpoint);
        tags.insert(tag);
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
                "RPC API for Sage wallet. All endpoints accept JSON request bodies and return JSON responses.\n\n\
                 **Authentication**: Sage RPC uses Mutual TLS for authentication.",
            ))
            .build(),
        paths_builder.build(),
    );

    // Add alphabetically sorted tags to the OpenAPI spec
    openapi.tags = Some(
        tags.into_iter()
            .map(|tag| TagBuilder::new().name(tag).build())
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
        .schema_from::<sage_api::NetworkKind>();

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

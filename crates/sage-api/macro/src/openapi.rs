use convert_case::{Case, Casing};
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Ident as Ident2, TokenStream};
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, LitStr, Token};

pub struct OpenApiArgs {
    pub tag: String,
    pub description: Option<String>,
}

impl Parse for OpenApiArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut tag = None;
        let mut description = None;

        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: LitStr = input.parse()?;

            match key.to_string().as_str() {
                "tag" => tag = Some(value.value()),
                "description" => description = Some(value.value()),
                _ => return Err(syn::Error::new(key.span(), "unknown attribute key")),
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(OpenApiArgs {
            tag: tag.ok_or_else(|| input.error("missing required `tag` attribute"))?,
            description,
        })
    }
}

pub fn impl_openapi_metadata(args: &OpenApiArgs, input: &syn::DeriveInput) -> TokenStream {
    let name = &input.ident;
    let tag = &args.tag;
    let description = args.description.as_ref().map(|desc| {
        quote! {
            fn openapi_description() -> Option<&'static str> {
                Some(#desc)
            }
        }
    });

    quote! {
        #input

        #[cfg(feature = "openapi")]
        impl crate::OpenApiMetadata for #name {
            fn openapi_tag() -> &'static str {
                #tag
            }

            #description
        }
    }
}

/// Generates `OpenAPI` schema registration code from endpoints.json
///
/// Takes advantage of the enforced pattern:
/// - Endpoint: `login` (from endpoints.json)
/// - Input type: `Login` (`PascalCase`)
/// - Response type: `LoginResponse` (`PascalCase` + "Response")
///
/// Reads endpoints.json at compile time and generates schema registrations
pub fn impl_openapi_registration(_input: TokenStream1) -> TokenStream1 {
    use indexmap::IndexMap;

    // Read endpoints.json at compile time
    let endpoints: IndexMap<String, bool> =
        serde_json::from_str(include_str!("../../endpoints.json"))
            .expect("Failed to parse endpoints.json");

    // Endpoints that use type aliases or don't implement ToSchema for their responses
    let skip_endpoints = ["get_networks", "get_network"];

    // Convert endpoint names to PascalCase type names
    let type_registrations = endpoints
        .keys()
        .filter_map(|endpoint_name| {
            // Skip endpoints that use type aliases or special response types
            if skip_endpoints.contains(&endpoint_name.as_str()) {
                return None;
            }

            let type_name = endpoint_name.to_case(Case::Pascal);
            let type_ident = Ident2::new(&type_name, proc_macro2::Span::call_site());
            let response_ident = Ident2::new(
                &format!("{type_name}Response"),
                proc_macro2::Span::call_site(),
            );

            Some(vec![
                quote! { .schema_from::<sage_api::#type_ident>() },
                quote! { .schema_from::<sage_api::#response_ident>() },
            ])
        })
        .flatten();

    quote! {
        components #(#type_registrations)*
    }
    .into()
}

/// Generates endpoint metadata match arms from endpoints.json
///
/// Generates match arms that call `OpenApiMetadata` trait methods for each endpoint
pub fn impl_endpoint_metadata(_input: TokenStream1) -> TokenStream1 {
    use indexmap::IndexMap;

    // Read endpoints.json at compile time
    let endpoints: IndexMap<String, bool> =
        serde_json::from_str(include_str!("../../endpoints.json"))
            .expect("Failed to parse endpoints.json");

    let match_arms = endpoints.keys().map(|endpoint_name| {
        let type_name = endpoint_name.to_case(Case::Pascal);
        let type_ident = Ident2::new(&type_name, proc_macro2::Span::call_site());

        quote! {
            #endpoint_name => (
                sage_api::#type_ident::openapi_tag(),
                sage_api::#type_ident::openapi_description()
                    .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
            )
        }
    });

    quote! {
        {
            use sage_api::OpenApiMetadata;
            match endpoint {
                #(#match_arms,)*
                _ => {
                    let tags = get_endpoint_tags(endpoint);
                    (tags[0], get_endpoint_description(endpoint))
                }
            }
        }
    }
    .into()
}

/// Generates request schema match arms from endpoints.json
pub fn impl_request_schemas(_input: TokenStream1) -> TokenStream1 {
    use indexmap::IndexMap;

    let endpoints: IndexMap<String, bool> =
        serde_json::from_str(include_str!("../../endpoints.json"))
            .expect("Failed to parse endpoints.json");

    let match_arms = endpoints.keys().map(|endpoint_name| {
        let type_name = endpoint_name.to_case(Case::Pascal);
        let schema_name = type_name.clone();

        quote! {
            #endpoint_name => RefOr::Ref(utoipa::openapi::Ref::new(
                concat!("#/components/schemas/", #schema_name)
            ))
        }
    });

    quote! {
        {
            use utoipa::openapi::{RefOr, schema::Schema};
            match endpoint {
                #(#match_arms,)*
                _ => create_generic_schema(&format!("Request body for {endpoint} endpoint")),
            }
        }
    }
    .into()
}

/// Generates response schema match arms from endpoints.json
pub fn impl_response_schemas(_input: TokenStream1) -> TokenStream1 {
    use indexmap::IndexMap;

    let endpoints: IndexMap<String, bool> =
        serde_json::from_str(include_str!("../../endpoints.json"))
            .expect("Failed to parse endpoints.json");

    // Special cases that need manual handling
    let special_responses = [
        (
            "get_networks",
            "create_generic_schema(\"List of available networks\")",
        ),
        (
            "get_network",
            "create_generic_schema(\"Current network information\")",
        ),
    ];

    // Shared transaction response endpoints
    let transaction_response_endpoints = [
        "send_xch",
        "bulk_send_xch",
        "combine",
        "split",
        "issue_cat",
        "send_cat",
        "bulk_send_cat",
        "multi_send",
        "create_did",
        "transfer_nfts",
        "add_nft_uri",
        "assign_nfts_to_did",
        "transfer_dids",
        "normalize_dids",
        "exercise_options",
        "transfer_options",
        "cancel_offer",
        "cancel_offers",
    ];

    // Shared empty response endpoints
    let empty_response_endpoints = [
        "remove_peer",
        "add_peer",
        "set_discover_peers",
        "set_target_peers",
        "set_network",
        "set_network_override",
        "set_delta_sync",
        "set_delta_sync_override",
        "set_change_address",
    ];

    let match_arms = endpoints.keys().map(|endpoint_name| {
        // Check if this is a special case
        if let Some((_, handler)) = special_responses
            .iter()
            .find(|(name, _)| name == endpoint_name)
        {
            let handler_tokens: TokenStream = handler
                .parse()
                .expect("Failed to parse special response handler");
            return quote! {
                #endpoint_name => #handler_tokens
            };
        }

        // Check if this uses TransactionResponse
        if transaction_response_endpoints.contains(&endpoint_name.as_str()) {
            return quote! {
                #endpoint_name => RefOr::Ref(utoipa::openapi::Ref::new(
                    "#/components/schemas/TransactionResponse"
                ))
            };
        }

        // Check if this uses EmptyResponse
        if empty_response_endpoints.contains(&endpoint_name.as_str()) {
            return quote! {
                #endpoint_name => RefOr::Ref(utoipa::openapi::Ref::new(
                    "#/components/schemas/EmptyResponse"
                ))
            };
        }

        // Standard case: endpoint + "Response"
        let type_name = endpoint_name.to_case(Case::Pascal);
        let response_name = format!("{type_name}Response");

        quote! {
            #endpoint_name => RefOr::Ref(utoipa::openapi::Ref::new(
                concat!("#/components/schemas/", #response_name)
            ))
        }
    });

    quote! {
        {
            use utoipa::openapi::{RefOr, schema::Schema};
            match endpoint {
                #(#match_arms,)*
                _ => create_generic_schema(&format!("Response data for {endpoint} endpoint")),
            }
        }
    }
    .into()
}

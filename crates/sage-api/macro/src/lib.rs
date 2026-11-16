mod openapi;

use std::collections::HashSet;

use convert_case::{Case, Casing};
use indexmap::IndexMap;
use proc_macro::{Delimiter, Group, TokenStream, TokenTree};
use quote::quote;
use syn::parse_macro_input;

#[proc_macro]
pub fn impl_endpoints(input: TokenStream) -> TokenStream {
    generate(&input, false)
}

#[proc_macro]
pub fn impl_endpoints_tauri(input: TokenStream) -> TokenStream {
    generate(&input, true)
}

/// Attribute macro for `OpenAPI` metadata
///
/// Usage:
/// ```ignore
/// #[openapi(
///     tag = "Wallets",
///     description = "Make the wallet with this fingerprint the active wallet..."
/// )]
/// pub struct Login { ... }
/// ```
#[proc_macro_attribute]
pub fn openapi(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as openapi::OpenApiArgs);
    let input = parse_macro_input!(input as syn::DeriveInput);

    openapi::impl_openapi_metadata(&args, &input).into()
}

/// Generates schema registrations from endpoints.json
///
/// Automatically reads endpoints.json and generates `.schema_from::<Type>()`
/// calls for all endpoint request/response types.
#[proc_macro]
pub fn register_openapi_types(input: TokenStream) -> TokenStream {
    openapi::impl_openapi_registration(input)
}

/// Generates endpoint metadata match arms from endpoints.json
///
/// Automatically generates match arms that retrieve tag and description
/// from `OpenApiMetadata` trait implementations.
#[proc_macro]
pub fn endpoint_metadata(input: TokenStream) -> TokenStream {
    openapi::impl_endpoint_metadata(input)
}

/// Generates request schema match arms from endpoints.json
///
/// Automatically generates match arms mapping endpoints to their request schemas.
#[proc_macro]
pub fn request_schemas(input: TokenStream) -> TokenStream {
    openapi::impl_request_schemas(input)
}

/// Generates response schema match arms from endpoints.json
///
/// Automatically generates match arms mapping endpoints to their response schemas.
#[proc_macro]
pub fn response_schemas(input: TokenStream) -> TokenStream {
    openapi::impl_response_schemas(input)
}

/// The endpoints which need exclusive access to `Sage`, since they take `&mut self`. Every
/// other endpoint runs under a shared lock, at the same time as others.
fn exclusive_endpoints() -> HashSet<String> {
    let exclusive: HashSet<String> =
        serde_json::from_str(include_str!("../../endpoints-exclusive.json"))
            .expect("Invalid exclusive endpoint file");

    let rpc: IndexMap<String, bool> =
        serde_json::from_str(include_str!("../../endpoints.json")).expect("Invalid endpoint file");
    let tauri: IndexMap<String, bool> =
        serde_json::from_str(include_str!("../../endpoints-tauri.json"))
            .expect("Invalid endpoint file");

    for endpoint in &exclusive {
        assert!(
            rpc.contains_key(endpoint) || tauri.contains_key(endpoint),
            "Unknown exclusive endpoint {endpoint}"
        );
    }

    exclusive
}

fn generate(input: &TokenStream, tauri: bool) -> TokenStream {
    let mut endpoints: IndexMap<String, bool> =
        serde_json::from_str(include_str!("../../endpoints.json")).expect("Invalid endpoint file");

    if tauri {
        let tauri_endpoints: IndexMap<String, bool> =
            serde_json::from_str(include_str!("../../endpoints-tauri.json"))
                .expect("Invalid endpoint file");

        endpoints.extend(tauri_endpoints);
    }

    let exclusive = exclusive_endpoints();

    let mut output = proc_macro2::TokenStream::new();

    for token in input.clone() {
        convert(token, &endpoints, &exclusive, None, &mut output);
    }

    output.into()
}

fn convert(
    tree: TokenTree,
    endpoints: &IndexMap<String, bool>,
    exclusive: &HashSet<String>,
    endpoint: Option<&str>,
    output: &mut proc_macro2::TokenStream,
) {
    match &tree {
        TokenTree::Ident(old) => {
            let Some(endpoint) = endpoint else {
                output.extend(proc_macro2::TokenStream::from(TokenStream::from(tree)));
                return;
            };
            let is_async = endpoints[endpoint];

            let ident = old.to_string();

            if ident == "endpoint_string" {
                output.extend(quote!(#endpoint));
            } else if ident == "maybe_async" {
                if is_async {
                    output.extend(quote!(async));
                }
            } else if ident == "maybe_await" {
                if is_async {
                    output.extend(quote!(.await));
                }
            } else if ident == "read_or_write" {
                if exclusive.contains(endpoint) {
                    output.extend(quote!(write));
                } else {
                    output.extend(quote!(read));
                }
            } else if ident.is_case(Case::Snake) {
                let ident = proc_macro2::Ident::new(
                    &ident.replace("endpoint", &endpoint.to_case(Case::Snake)),
                    old.span().into(),
                );
                output.extend(quote!(#ident));
            } else if ident.is_case(Case::Pascal) {
                let ident = proc_macro2::Ident::new(
                    &ident.replace("Endpoint", &endpoint.to_case(Case::Pascal)),
                    old.span().into(),
                );
                output.extend(quote!(#ident));
            } else {
                output.extend(proc_macro2::TokenStream::from(TokenStream::from(tree)));
            }
        }
        TokenTree::Literal(..) | TokenTree::Punct(..) => {
            output.extend(proc_macro2::TokenStream::from(TokenStream::from(tree)));
        }
        TokenTree::Group(group) => {
            let mut stream = group.stream().into_iter().peekable();

            let repeat = stream.peek().is_some_and(|token| {
                if let TokenTree::Ident(ident) = &token {
                    ident.to_string() == "repeat" && group.delimiter() == Delimiter::Parenthesis
                } else {
                    false
                }
            });

            if repeat {
                stream.next();
            }

            if repeat {
                for endpoint in endpoints.keys() {
                    for tree in stream.clone() {
                        convert(tree, endpoints, exclusive, Some(endpoint), output);
                    }
                }
            } else {
                let mut inner = proc_macro2::TokenStream::new();

                for tree in stream {
                    convert(tree, endpoints, exclusive, endpoint, &mut inner);
                }

                output.extend(proc_macro2::TokenStream::from(TokenStream::from(
                    TokenTree::Group(Group::new(group.delimiter(), inner.into())),
                )));
            }
        }
    }
}

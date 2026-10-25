mod openapi;

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
///     tag = "Authentication & Keys",
///     description = "Authenticate and log into a wallet..."
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

/// The endpoints to generate code for.
struct Endpoints {
    /// Each endpoint, and whether it is async.
    kinds: IndexMap<String, bool>,
    /// Endpoints which manage the Sage lock themselves, so that they can release it while
    /// waiting on the network. They take `&Mutex<Sage>` rather than `&self`.
    unlocked: Vec<String>,
}

fn generate(input: &TokenStream, tauri: bool) -> TokenStream {
    let mut kinds: IndexMap<String, bool> =
        serde_json::from_str(include_str!("../../endpoints.json")).expect("Invalid endpoint file");

    if tauri {
        let tauri_endpoints: IndexMap<String, bool> =
            serde_json::from_str(include_str!("../../endpoints-tauri.json"))
                .expect("Invalid endpoint file");

        kinds.extend(tauri_endpoints);
    }

    let unlocked: Vec<String> =
        serde_json::from_str(include_str!("../../endpoints-unlocked.json"))
            .expect("Invalid endpoint file");

    for endpoint in &unlocked {
        assert!(
            kinds.get(endpoint).copied().unwrap_or(false),
            "Unlocked endpoint {endpoint} must be an async endpoint"
        );
    }

    let endpoints = Endpoints { kinds, unlocked };

    let mut output = proc_macro2::TokenStream::new();

    for token in input.clone() {
        convert(token, &endpoints, None, &mut output);
    }

    output.into()
}

fn convert(
    tree: TokenTree,
    endpoints: &Endpoints,
    endpoint: Option<&str>,
    output: &mut proc_macro2::TokenStream,
) {
    match &tree {
        TokenTree::Ident(old) => {
            let Some(endpoint) = endpoint else {
                output.extend(proc_macro2::TokenStream::from(TokenStream::from(tree)));
                return;
            };
            let is_async = endpoints.kinds[endpoint];

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

            let keyword = stream.peek().and_then(|token| {
                if let TokenTree::Ident(ident) = &token
                    && group.delimiter() == Delimiter::Parenthesis
                {
                    Some(ident.to_string())
                } else {
                    None
                }
            });

            match keyword.as_deref() {
                Some("repeat") => {
                    stream.next();

                    for endpoint in endpoints.kinds.keys() {
                        for tree in stream.clone() {
                            convert(tree, endpoints, Some(endpoint), output);
                        }
                    }
                }
                // `(if_unlocked { ... } { ... })` expands to the first block for endpoints
                // which manage the Sage lock themselves, and to the second otherwise.
                Some("if_unlocked") if endpoint.is_some() => {
                    stream.next();

                    let blocks: Vec<TokenTree> = stream.collect();
                    let [TokenTree::Group(unlocked), TokenTree::Group(locked)] = blocks.as_slice()
                    else {
                        panic!("Expected `(if_unlocked {{ ... }} {{ ... }})`");
                    };

                    let is_unlocked = endpoint
                        .is_some_and(|endpoint| endpoints.unlocked.iter().any(|e| e == endpoint));
                    let block = if is_unlocked { unlocked } else { locked };

                    for tree in block.stream() {
                        convert(tree, endpoints, endpoint, output);
                    }
                }
                _ => {
                    let mut inner = proc_macro2::TokenStream::new();

                    for tree in stream {
                        convert(tree, endpoints, endpoint, &mut inner);
                    }

                    output.extend(proc_macro2::TokenStream::from(TokenStream::from(
                        TokenTree::Group(Group::new(group.delimiter(), inner.into())),
                    )));
                }
            }
        }
    }
}

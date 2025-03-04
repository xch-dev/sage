use convert_case::{Case, Casing};
use indexmap::IndexMap;
use proc_macro::{Delimiter, Group, TokenStream, TokenTree};
use proc_macro2::Ident;
use quote::quote;

#[proc_macro]
pub fn impl_endpoints(input: TokenStream) -> TokenStream {
    generate(&input, false)
}

#[proc_macro]
pub fn impl_endpoints_tauri(input: TokenStream) -> TokenStream {
    generate(&input, true)
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

    let mut output = proc_macro2::TokenStream::new();

    for token in input.clone() {
        convert(token, &endpoints, None, &mut output);
    }

    output.into()
}

fn convert(
    tree: TokenTree,
    endpoints: &IndexMap<String, bool>,
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
            } else if ident.is_case(Case::Snake) {
                let ident = Ident::new(
                    &ident.replace("endpoint", &endpoint.to_case(Case::Snake)),
                    old.span().into(),
                );
                output.extend(quote!(#ident));
            } else if ident.is_case(Case::Pascal) {
                let ident = Ident::new(
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
                        convert(tree, endpoints, Some(endpoint), output);
                    }
                }
            } else {
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

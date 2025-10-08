use convert_case::{Case, Casing};
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, Ident, LitStr, Token};

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

/// List of input types for `OpenAPI` registration
/// Automatically derives endpoint names and response types from the pattern
struct OpenApiTypes {
    types: syn::punctuated::Punctuated<Ident, Token![,]>,
}

impl Parse for OpenApiTypes {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(OpenApiTypes {
            types: syn::punctuated::Punctuated::parse_terminated(input)?,
        })
    }
}

/// Generates `OpenAPI` registration code from just the input type names
///
/// Takes advantage of the enforced pattern:
/// - Input type: `Login` (`PascalCase`)
/// - Endpoint: `login` (`snake_case`)
/// - Response type: `LoginResponse` (`PascalCase` + "Response")
pub fn impl_openapi_registration(input: TokenStream1) -> TokenStream1 {
    let parsed = match syn::parse::<OpenApiTypes>(input) {
        Ok(p) => p,
        Err(e) => return e.to_compile_error().into(),
    };

    let types = parsed.types;

    // Generate component registrations
    let component_registrations = types.iter().flat_map(|type_name| {
        let response_name = Ident::new(&format!("{type_name}Response"), type_name.span());
        vec![
            quote! { .schema_from::<sage_api::#type_name>() },
            quote! { .schema_from::<sage_api::#response_name>() },
        ]
    });

    // Generate metadata match arms
    let metadata_arms = types.iter().map(|type_name| {
        let endpoint_name = type_name.to_string().to_case(Case::Snake);
        quote! {
            #endpoint_name => (
                sage_api::#type_name::openapi_tag(),
                sage_api::#type_name::openapi_description()
                    .map_or_else(|| get_endpoint_description(endpoint), ToString::to_string),
            )
        }
    });

    // Generate request schema match arms
    let request_schema_arms = types.iter().map(|type_name| {
        let endpoint_name = type_name.to_string().to_case(Case::Snake);
        let schema_name = type_name.to_string();
        quote! {
            #endpoint_name => RefOr::Ref(utoipa::openapi::Ref::new(
                concat!("#/components/schemas/", #schema_name)
            ))
        }
    });

    // Generate response schema match arms
    let response_schema_arms = types.iter().map(|type_name| {
        let endpoint_name = type_name.to_string().to_case(Case::Snake);
        let response_name = format!("{type_name}Response");
        quote! {
            #endpoint_name => RefOr::Ref(utoipa::openapi::Ref::new(
                concat!("#/components/schemas/", #response_name)
            ))
        }
    });

    quote! {
        fn get_generated_metadata(endpoint: &str) -> Option<(&'static str, String)> {
            match endpoint {
                #(#metadata_arms,)*
                _ => None,
            }
        }

        fn get_generated_request_schema(endpoint: &str) -> Option<utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>> {
            use utoipa::openapi::{RefOr, schema::Schema};
            match endpoint {
                #(#request_schema_arms,)*
                _ => None,
            }
        }

        fn get_generated_response_schema(endpoint: &str) -> Option<utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>> {
            use utoipa::openapi::{RefOr, schema::Schema};
            match endpoint {
                #(#response_schema_arms,)*
                _ => None,
            }
        }

        fn register_generated_components(mut components: utoipa::openapi::ComponentsBuilder) -> utoipa::openapi::ComponentsBuilder {
            components #(#component_registrations)*
        }
    }
    .into()
}

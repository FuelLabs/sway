use crate::errors::Error;
use crate::json_abi::parse_param;
use crate::types::{expand_type, ParamType, Property};
use crate::utils::ident;
use inflector::Inflector;
use proc_macro2::TokenStream;
use quote::quote;

/// Functions used by the Abigen to expand custom types defined in an ABI spec.

// TODO: Right now we have an "end-to-end" test suite for the Abigen!
// under `fuels-abigen/tests/harness.rs`. But it would be nice to have
// tests at the level of this component.

/// Transforms a custom type defined in [`Property`] into a [`TokenStream`]
/// that represents that same type as a Rust-native struct.
pub fn expand_internal_struct(name: &str, prop: &Property) -> Result<TokenStream, Error> {
    let components = prop.components.as_ref().unwrap();
    let mut fields = Vec::with_capacity(components.len());

    // Holds a TokenStream representing the process of
    // creating a [`Token`] and pushing it a vector of Tokens.
    let mut struct_fields_tokens = Vec::new();

    // For each component, we create two TokenStreams:
    // 1. A struct field declaration like `pub #field_name: #component_name`
    // 2. The creation of a token and its insertion into a vector of Tokens.
    for component in components {
        let component_name = ident(&component.name.to_class_case());
        let field_name = ident(&component.name.to_snake_case());
        let param_type = parse_param(&component)?;

        match param_type {
            // Case where a struct takes another struct
            ParamType::Struct(_params) => {
                fields.push(quote! {pub #field_name: #component_name});
                struct_fields_tokens.push(quote! { tokens.push(self.#field_name.into_token()) });
            }
            // Elementary type
            _ => {
                let ty = expand_type(&param_type)?;
                let param_type_string = ident(&param_type.to_string());

                // Field declaration
                fields.push(quote! { pub #field_name: #ty});

                // Token creation and insertion
                struct_fields_tokens
                    .push(quote! {tokens.push(Token::#param_type_string(self.#field_name))})
            }
        }
    }

    let name = ident(name);

    // Actual creation of the struct, using the inner TokenStreams from above
    // to produce the TokenStream that represents the whole struct + methods
    // declaration.
    Ok(quote! {
        #[derive(Clone, Debug, Default, Eq, PartialEq)]
        pub struct #name {
            #( #fields ),*
        }

        impl #name {
            pub fn into_token(self) -> Token {
                let mut tokens = Vec::new();
                #( #struct_fields_tokens; )*

                Token::Struct(tokens)
            }
        }
    })
}

/// Transforms a custom enum defined in [`Property`] into a [`TokenStream`]
/// that represents that same type as a Rust-native enum.
pub fn expand_internal_enum(name: &str, prop: &Property) -> Result<TokenStream, Error> {
    let components = prop.components.as_ref().unwrap();
    let mut fields = Vec::with_capacity(components.len());

    // Holds a TokenStream representing the process of
    // creating an enum [`Token`].
    let mut enum_selector_builder = Vec::new();

    let name = ident(name);

    for (discriminant, component) in components.iter().enumerate() {
        let field_name = ident(&component.name.to_class_case());

        let param_type = parse_param(&component)?;
        match param_type {
            // Case where an enum takes another enum
            ParamType::Enum(_params) => {
                // TODO: Support nested enums
                unimplemented!()
            }
            // Elementary type
            _ => {
                let ty = expand_type(&param_type)?;
                let param_type_string = ident(&param_type.to_string());

                // Enum variant declaration
                fields.push(quote! { #field_name(#ty)});

                // Token creation
                enum_selector_builder.push(quote! {
                    #name::#field_name(value) => (#discriminant as u8, Token::#param_type_string(value))
                })
            }
        }
    }

    // Actual creation of the enum, using the inner TokenStreams from above
    // to produce the TokenStream that represents the whole enum + methods
    // declaration.
    Ok(quote! {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub enum #name {
            #( #fields ),*
        }

        impl #name {
            pub fn into_token(self) -> Token {

                let (dis, tok) = match self {
                    #( #enum_selector_builder, )*
                };

                let selector = (dis, tok);
                Token::Enum(Box::new(selector))
            }
        }
    })
}

use crate::abi_encoder::ABIEncoder;
use crate::code_gen::docs_gen::expand_doc;
use crate::errors::Error;
use crate::json_abi::{parse_param, ABIParser};
use crate::types::{expand_type, Function, ParamType, Property, Selector};
use crate::utils::{ident, safe_ident};
use inflector::Inflector;

use proc_macro2::{Literal, TokenStream};
use quote::quote;
use std::collections::HashMap;

/// Functions used by the Abigen to expand functions defined in an ABI spec.

// TODO: Right now we have an "end-to-end" test suite for the Abigen!
// under `fuels-abigen/tests/harness.rs`. But it would be nice to have
// tests at the level of this component.

/// Transforms a function defined in [`Function`] into a [`TokenStream`]
/// that represents that same function signature as a Rust-native function
/// declaration.
/// The actual logic inside the function is the function `method_hash` under
/// [`Contract`], which is responsible for encoding the function selector
/// and the function parameters that will be used in the actual contract call.
pub fn expand_function(
    function: &Function,
    abi_parser: &ABIParser,
    custom_enums: &HashMap<String, Property>,
    custom_structs: &HashMap<String, Property>,
) -> Result<TokenStream, Error> {
    let name = safe_ident(&function.name);
    let fn_signature = abi_parser.build_fn_selector(&function.name, &function.inputs);

    let encoded = ABIEncoder::encode_function_selector(fn_signature.as_bytes());

    let tokenized_signature = expand_selector(encoded);
    let tokenized_output = expand_fn_outputs(&function.outputs)?;
    let result = quote! { ContractCall<#tokenized_output> };

    let (input, arg) = expand_function_arguments(function, custom_enums, custom_structs)?;

    let doc = expand_doc(&format!(
        "Calls the contract's `{}` (0x{}) function",
        function.name,
        hex::encode(encoded)
    ));

    Ok(quote! {
        #doc
        pub fn #name(&self #input) -> #result {
            Contract::method_hash(#tokenized_signature, #arg).expect("method not found (this should never happen)")
        }
    })
}

fn expand_selector(selector: Selector) -> TokenStream {
    let bytes = selector.iter().copied().map(Literal::u8_unsuffixed);
    quote! { [#( #bytes ),*] }
}

/// Expands the output of a function, i.e. what comes after `->` in a function
/// signature.
fn expand_fn_outputs(outputs: &[Property]) -> Result<TokenStream, Error> {
    // TODO: future, support struct outputs
    match outputs.len() {
        0 => Ok(quote! { () }),
        1 => expand_type(&parse_param(&outputs[0])?),
        _ => {
            let types = outputs
                .iter()
                .map(|param| expand_type(&parse_param(param)?))
                .collect::<Result<Vec<_>, Error>>()?;
            Ok(quote! { (#( #types ),*) })
        }
    }
}

/// Expands the arguments in a function declaration and the same arguments as input
/// to a function call. For instance:
/// 1. The `my_arg: u32` in `pub fn my_func(my_arg: u32) -> ()`
/// 2. The `my_arg.into_token()` in `another_fn_call(my_arg.into_token())`
fn expand_function_arguments(
    fun: &Function,
    custom_enums: &HashMap<String, Property>,
    custom_structs: &HashMap<String, Property>,
) -> Result<(TokenStream, TokenStream), Error> {
    let mut args = Vec::with_capacity(fun.inputs.len());
    let mut call_args = Vec::with_capacity(fun.inputs.len());

    // For each [`Property`] in a function input we expand:
    // 1. The name of the argument;
    // 2. The type of the argument;
    for (i, param) in fun.inputs.iter().enumerate() {
        // TokenStream representing the name of the argument
        let name = expand_input_name(i, &param.name);

        let rust_enum_name = custom_enums.get(&param.name);
        let rust_struct_name = custom_structs.get(&param.name);

        // TokenStream representing the type of the argument
        let ty = expand_input_param(
            fun,
            &param.name,
            &parse_param(param)?,
            &rust_enum_name,
            &rust_struct_name,
        )?;

        // Add the TokenStream to argument declarations
        args.push(quote! { #name: #ty });

        // This `name` TokenStream is also added to the call arguments
        call_args.push(name);
    }

    // The final TokenStream of the argument declaration in a function declaration
    let args = quote! { #( , #args )* };

    // The final TokenStream of the arguments being passed in a function call
    // It'll look like `&[my_arg.into_token(), another_arg.into_token()]`
    // as the [`Contract`] `method_hash` function expects a slice of Tokens
    // in order to encode the call.
    let call_args = match call_args.len() {
        0 => quote! { () },
        _ => quote! { &[ #(#call_args.into_token(), )* ] },
    };

    Ok((args, call_args))
}

/// Expands a positional identifier string that may be empty.
///
/// Note that this expands the parameter name with `safe_ident`, meaning that
/// identifiers that are reserved keywords get `_` appended to them.
pub fn expand_input_name(index: usize, name: &str) -> TokenStream {
    let name_str = match name {
        "" => format!("p{}", index),
        n => n.to_snake_case(),
    };
    let name = safe_ident(&name_str);

    quote! { #name }
}

// Expands the type of an argument being passed in a function declaration.
// I.e.: `pub fn my_func(my_arg: u32) -> ()`, in this case, `u32` is the
// type, coming in as a `ParamType::U32`.
fn expand_input_param(
    fun: &Function,
    param: &str,
    kind: &ParamType,
    rust_enum_name: &Option<&Property>,
    rust_struct_name: &Option<&Property>,
) -> Result<TokenStream, Error> {
    match kind {
        ParamType::Array(ty, _) => {
            let ty = expand_input_param(fun, param, ty, rust_enum_name, rust_struct_name)?;
            Ok(quote! {
                ::std::vec::Vec<#ty>
            })
        }
        ParamType::Enum(_) => {
            let ident = ident(&rust_enum_name.unwrap().name);
            Ok(quote! { #ident })
        }
        ParamType::Struct(_) => {
            let ident = ident(&rust_struct_name.unwrap().name);
            Ok(quote! { #ident })
        }
        // Primitive type
        _ => expand_type(kind),
    }
}

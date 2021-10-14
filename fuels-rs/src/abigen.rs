use crate::abi_encoder::ABIEncoder;
use crate::errors::Error;

use crate::json_abi::{parse_param, ABI};

use crate::bindings::ContractBindings;
use crate::tokens::{Detokenize, Tokenize};
use crate::types::{expand_type, ByteArray, Function, JsonABI, ParamType, Property, Selector};
use inflector::Inflector;
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::{collections::HashMap, fs::File, io::Write, path::Path};
use syn::{Ident as SynIdent, Path as SynPath};
pub struct Abigen {
    /// The parsed ABI.
    abi: JsonABI,

    /// The parser used for human readable format
    abi_parser: ABI,

    /// Contains all the solidity structs extracted from the JSON ABI.
    // internal_structs: InternalStructs, unclear if needed

    /// Was the ABI in human readable format?
    //human_readable: bool,

    /// The contract name as an identifier.
    contract_name: Ident, // TODO: option for now

    /// Format the code using a locally installed copy of `rustfmt`.
    rustfmt: bool,
}

/// Expands a identifier string into an token.
pub fn ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}

impl Abigen {
    /// Creates a new contract with the given ABI JSON source.
    pub fn new(contract_name: &str, abi_source: &str) -> Result<Self, Error> {
        let parsed_abi: JsonABI = serde_json::from_str(abi_source)?;
        Ok(Self {
            contract_name: ident(contract_name),
            abi: parsed_abi,
            abi_parser: ABI::new(),
            rustfmt: true,
        })
    }

    /// Generates the contract bindings.
    pub fn generate(self) -> Result<ContractBindings, Error> {
        let rustfmt = self.rustfmt;
        let tokens = self.expand()?;

        Ok(ContractBindings { tokens, rustfmt })
    }

    pub fn functions(&self) -> Result<TokenStream, Error> {
        // The goal here is to turn the parsed abi into TokenStream
        let mut tokenized_functions = Vec::new();

        for function in &self.abi {
            let tokenized_fn = self.expand_function(function, None)?;
            tokenized_functions.push(tokenized_fn);
        }

        Ok(quote! { #( #tokenized_functions )* })
    }

    fn expand_function(
        &self,
        function: &Function,
        _alias: Option<Ident>,
    ) -> Result<TokenStream, Error> {
        let name = Abigen::safe_ident(&function.name);

        let fn_signature = self
            .abi_parser
            .build_fn_selector(&function.name, &function.inputs);

        let encoded = ABIEncoder::encode_function_selector(fn_signature.as_bytes());
        let tokenized_signature = Abigen::expand_selector(encoded);

        let tokenized_output = Abigen::expand_fn_outputs(&function.outputs)?;

        let result = quote! { Call<#tokenized_output> };

        let (input, arg) = self.expand_inputs_call_arg_with_structs(function)?;

        let doc = Abigen::expand_doc(&format!(
            "Calls the contract's `{}` (0x{}) function",
            function.name,
            hex::encode(encoded)
        ));

        Ok(quote! {
            #doc
            pub fn #name(&self #input) -> #result {
                ContractCall::method_hash(#tokenized_signature, #arg).expect("method not found (this should never happen)")
            }
        })
    }

    /// Expands a doc string into an attribute token stream.
    pub fn expand_doc(s: &str) -> TokenStream {
        let doc = Literal::string(s);
        quote! {
            #[doc = #doc]
        }
    }

    fn expand_selector(selector: Selector) -> TokenStream {
        let bytes = selector.iter().copied().map(Literal::u8_unsuffixed);
        quote! { [#( #bytes ),*] }
    }

    fn expand_fn_outputs(outputs: &[Property]) -> Result<TokenStream, Error> {
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

    fn expand_inputs_call_arg_with_structs(
        &self,
        fun: &Function,
    ) -> Result<(TokenStream, TokenStream), Error> {
        let mut args = Vec::with_capacity(fun.inputs.len());
        let mut call_args = Vec::with_capacity(fun.inputs.len());

        for (i, param) in fun.inputs.iter().enumerate() {
            let name = Abigen::expand_input_name(i, &param.name);

            let ty = self.expand_input_param(fun, &param.name, &parse_param(param)?)?;
            println!("name: {:?}\n", name);
            println!("ty: {:?}\n", ty);
            args.push(quote! { #name: #ty });
            let call_arg = match parse_param(param)? {
                // this is awkward edge case where the function inputs are a single struct
                // we need to force this argument into a tuple so it gets expanded to `((#name,))`
                // this is currently necessary because internally `flatten_tokens` is called which removes the outermost `tuple` level
                // and since `((#name))` is not a rust tuple it doesn't get wrapped into another tuple that will be peeled off by `flatten_tokens`
                ParamType::Struct(_) if fun.inputs.len() == 1 => {
                    // make sure the tuple gets converted to `Token::Tuple`
                    // quote! {(#name,)}
                    unimplemented!()
                }
                _ => name,
            };
            call_args.push(call_arg);
        }
        let args = quote! { #( , #args )* };
        let call_args = match call_args.len() {
            0 => quote! { () },
            1 => quote! { #( #call_args )* },
            _ => quote! { ( #(#call_args, )* ) },
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
        let name = Abigen::safe_ident(&name_str);

        quote! { #name }
    }

    fn expand_input_param(
        &self,
        fun: &Function,
        param: &str,
        kind: &ParamType,
    ) -> Result<TokenStream, Error> {
        match kind {
            ParamType::Array(ty, _) => {
                let ty = self.expand_input_param(fun, param, ty)?;
                Ok(quote! {
                    ::std::vec::Vec<#ty>
                })
            }

            ParamType::Struct(_) => {
                // TODO: structs
                unimplemented!()
                // let ty = if let Some(rust_struct_name) = self
                //     .internal_structs
                //     .get_function_input_struct_type(&fun.name, param)
                // {
                //     let ident = util::ident(rust_struct_name);
                //     quote! {#ident}
                // } else {
                //     types::expand(kind)?
                // };
                // Ok(ty)
            }
            _ => expand_type(kind),
        }
    }

    // This is where the magic happens
    pub fn expand(&self) -> Result<TokenStream, Error> {
        let name = &self.contract_name;
        let name_mod = ident(&format!(
            "{}_mod",
            self.contract_name.to_string().to_lowercase()
        ));

        let contract_functions = self.functions()?; // This is the part we care the most for now

        Ok(quote! {
            pub use #name_mod::*;

            #[allow(clippy::too_many_arguments)]
            mod #name_mod {
                #![allow(clippy::enum_variant_names)]
                #![allow(dead_code)]
                #![allow(unused_imports)]

                // #imports
                use fuels_rs::contract::{ContractCall, Call};
                // #struct_decl

                pub struct #name;

                impl #name {
                    pub fn new() -> Self {
                        Self{}
                    }

                    #contract_functions
                }
            }
        })

        // unimplemented!()
    }

    // Expands an identifier string into a token and appending `_` if the
    /// identifier is for a reserved keyword.
    ///
    /// Parsing keywords like `self` can fail, in this case we add an underscore.
    pub fn safe_ident(name: &str) -> Ident {
        syn::parse_str::<SynIdent>(name).unwrap_or_else(|_| ident(&format!("{}_", name)))
    }
}

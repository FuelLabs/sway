use std::collections::HashMap;

use crate::abi_encoder::ABIEncoder;
use crate::bindings::ContractBindings;
use crate::errors::Error;
use crate::json_abi::{parse_param, ABI};
use crate::types::{expand_type, Function, JsonABI, ParamType, Property, Selector};
use inflector::Inflector;
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;

use syn::Ident as SynIdent;
pub struct Abigen {
    /// The parsed ABI.
    abi: JsonABI,

    /// The parser used for human readable format
    abi_parser: ABI,

    /// Contains all the solidity structs extracted from the JSON ABI.
    // internal_structs: InternalStructs, unclear if needed

    /// The contract name as an identifier.
    contract_name: Ident,

    custom_structs: HashMap<String, Property>,
    //custom_enums: Option<HashMap<String, Property>>,
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
            custom_structs: Abigen::get_custom_structs(&parsed_abi),
            contract_name: ident(contract_name),
            abi: parsed_abi,
            abi_parser: ABI::new(),
            rustfmt: true,
        })
    }

    fn get_custom_structs(abi: &JsonABI) -> HashMap<String, Property> {
        let mut structs = HashMap::new();
        for function in abi {
            for prop in &function.inputs {
                if prop.type_field.eq_ignore_ascii_case("struct") {
                    if !structs.contains_key(&prop.name) {
                        structs.insert(prop.name.clone(), prop.clone());
                    }
                }
            }
        }

        println!("structs: {:?}\n", structs);

        structs
    }

    /// Generates the contract bindings.
    pub fn generate(self) -> Result<ContractBindings, Error> {
        let rustfmt = self.rustfmt;
        let tokens = self.expand()?;

        Ok(ContractBindings { tokens, rustfmt })
    }

    pub fn expand(&self) -> Result<TokenStream, Error> {
        let name = &self.contract_name;
        let name_mod = ident(&format!(
            "{}_mod",
            self.contract_name.to_string().to_lowercase()
        ));

        // TODO: create structs used in the ABI
        // 5. Declare the structs parsed from the human readable abi

        let contract_functions = self.functions()?; // This is the part we care the most for now

        Ok(quote! {
            pub use #name_mod::*;

            #[allow(clippy::too_many_arguments)]
            mod #name_mod {
                #![allow(clippy::enum_variant_names)]
                #![allow(dead_code)]
                #![allow(unused_imports)]

                use fuels_rs::contract::{Contract, ContractCall};
                use fuels_rs::tokens::Tokenizable;

                pub struct #name;

                impl #name {
                    pub fn new() -> Self {
                        Self{}
                    }

                    #contract_functions
                }
            }
        })
    }

    /// Expand all structs parsed from the internal types of the JSON ABI
    // fn expand_internal_struct(
    //     &self,
    //     name: &str,
    //     sol_struct: &SolStruct,
    //     tuple: ParamType,
    // ) -> Result<TokenStream> {
    //     let mut fields = Vec::with_capacity(sol_struct.fields().len());
    //     for field in sol_struct.fields() {
    //         let field_name = util::ident(&field.name().to_snake_case());
    //         match field.r#type() {
    //             FieldType::Elementary(ty) => {
    //                 let ty = types::expand(ty)?;
    //                 fields.push(quote! { pub #field_name: #ty });
    //             }
    //             FieldType::Struct(struct_ty) => {
    //                 let ty = expand_struct_type(struct_ty);
    //                 fields.push(quote! { pub #field_name: #ty });
    //             }
    //             FieldType::Mapping(_) => {
    //                 return Err(anyhow::anyhow!(
    //                     "Mapping types in struct `{}` are not supported {:?}",
    //                     name,
    //                     field
    //                 ));
    //             }
    //         }
    //     }

    //     let sig = if let ParamType::Tuple(ref tokens) = tuple {
    //         tokens
    //             .iter()
    //             .map(|kind| kind.to_string())
    //             .collect::<Vec<_>>()
    //             .join(",")
    //     } else {
    //         "".to_string()
    //     };

    //     let abi_signature = format!("{}({})", name, sig,);

    //     let abi_signature_doc = util::expand_doc(&format!("`{}`", abi_signature));

    //     let name = util::ident(name);

    //     // use the same derives as for events
    //     let derives = &self.event_derives;
    //     let derives = quote! {#(#derives),*};

    //     Ok(quote! {
    //         #abi_signature_doc
    //         #[derive(Clone, Debug, Default, Eq, PartialEq, ethers::contract::EthAbiType, #derives)]
    //         pub struct #name {
    //             #( #fields ),*
    //         }
    //     })
    // }

    pub fn functions(&self) -> Result<TokenStream, Error> {
        // The goal here is to turn the parsed abi into TokenStream
        let mut tokenized_functions = Vec::new();

        for function in &self.abi {
            let tokenized_fn = self.expand_function(function, None)?;
            tokenized_functions.push(tokenized_fn);
        }

        Ok(quote! { #( #tokenized_functions )* })
    }

    // TODO: struct inputs don't work _at all_.
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

        let result = quote! { ContractCall<#tokenized_output> };

        let (input, arg) = self.expand_inputs_call_arg_with_structs(function)?;

        let doc = Abigen::expand_doc(&format!(
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
            args.push(quote! { #name: #ty });
            let call_arg = match parse_param(param)? {
                // this is awkward edge case where the function inputs are a single struct
                // we need to force this argument into a tuple so it gets expanded to `((#name,))`
                // this is currently necessary because internally `flatten_tokens` is called which removes the outermost `tuple` level
                // and since `((#name))` is not a rust tuple it doesn't get wrapped into another tuple that will be peeled off by `flatten_tokens`
                ParamType::Struct(_) if fun.inputs.len() == 1 => {
                    // make sure the tuple gets converted to `Token::Tuple`
                    quote! {(#name,)}
                }
                _ => name,
            };
            call_args.push(call_arg);
        }
        let args = quote! { #( , #args )* };
        let call_args = match call_args.len() {
            0 => quote! { () },
            //1 => quote! { #( #call_args.into_token() )* },
            _ => quote! { &[ #(#call_args.into_token(), )* ] },
        };

        // Can we turn call_args into Tokens?
        //

        println!("args: {:?}\n", args);
        println!("call_args: {:?}\n", call_args);

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
                let rust_struct_name = self.custom_structs.get(param).unwrap();
                let ident = ident(&rust_struct_name.name);
                Ok(quote! { #ident })
                // TODO: structs
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

    // Expands an identifier string into a token and appending `_` if the
    /// identifier is for a reserved keyword.
    ///
    /// Parsing keywords like `self` can fail, in this case we add an underscore.
    pub fn safe_ident(name: &str) -> Ident {
        syn::parse_str::<SynIdent>(name).unwrap_or_else(|_| ident(&format!("{}_", name)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: move a lot of the tests from ethers (e.g methods.rs file) here
    #[test]
    fn generates_bindings() {
        let contract = r#"
        [
            {
                "type":"contract",
                "inputs":[
                    {
                        "name":"arg",
                        "type":"u32"
                    }
                ],
                "name":"takes_u32_returns_bool",
                "outputs":[
                    {
                        "name":"",
                        "type":"bool"
                    }
                ]
            }
        ]
        "#;

        let bindings = Abigen::new("test", contract).unwrap().generate().unwrap();
        bindings.write(std::io::stdout()).unwrap();
    }

    #[test]
    fn generates_bindings_two_args() {
        let contract = r#"
        [
            {
                "type":"contract",
                "inputs":[
                    {
                        "name":"arg",
                        "type":"u32"
                    },
                    {
                        "name":"second_arg",
                        "type":"u16"
                    }
                ],
                "name":"takes_ints_returns_bool",
                "outputs":[
                    {
                        "name":"",
                        "type":"bool"
                    }
                ]
            }
        ]
        "#;

        let bindings = Abigen::new("test", contract).unwrap().generate().unwrap();
        bindings.write(std::io::stdout()).unwrap();
    }

    #[test]
    fn custom_types() {
        let contract = r#"
        [
            {
                "type":"contract",
                "inputs":[
                    {
                        "name":"MyStruct",
                        "type":"struct",
                        "components": [
                            {
                                "name": "foo",
                                "type": "u8"
                            },
                            {
                                "name": "bar",
                                "type": "bool"
                            }
                        ]
                    }
                ],
                "name":"takes_struct",
                "outputs":[]
            }
        ]
        "#;

        let bindings = Abigen::new("custom", contract).unwrap().generate().unwrap();
        bindings.write(std::io::stdout()).unwrap();
    }
}

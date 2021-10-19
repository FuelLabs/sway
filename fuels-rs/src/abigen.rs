use std::collections::HashMap;

use crate::abi_encoder::ABIEncoder;
use crate::bindings::ContractBindings;
use crate::errors::Error;
use crate::json_abi::{self, parse_param, ABI};
use crate::source::Source;
use crate::types::{expand_type, Function, JsonABI, ParamType, Property, Selector};
use inflector::Inflector;
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;

use syn::token::Struct;
use syn::Ident as SynIdent;

// TODO: continue from here, this needs a MAJOR clean-up. We might need to break it down into smaller files.

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

    custom_enums: HashMap<String, Property>,

    /// Format the code using a locally installed copy of `rustfmt`.
    rustfmt: bool,
}

/// Expands a identifier string into an token.
pub fn ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}

impl Abigen {
    /// Creates a new contract with the given ABI JSON source.
    pub fn new<S: AsRef<str>>(contract_name: &str, abi_source: S) -> Result<Self, Error> {
        let source = Source::parse(abi_source).unwrap();
        let parsed_abi: JsonABI = serde_json::from_str(&source.get().unwrap())?;

        Ok(Self {
            custom_structs: Abigen::get_custom_structs(&parsed_abi),
            custom_enums: Abigen::get_custom_enums(&parsed_abi),
            abi: parsed_abi,
            contract_name: ident(contract_name),
            abi_parser: ABI::new(),
            rustfmt: true,
        })
    }

    // TODO: improve this function
    fn get_custom_structs(abi: &JsonABI) -> HashMap<String, Property> {
        let mut structs = HashMap::new();
        let mut inner_structs: Vec<Property> = Vec::new();
        for function in abi {
            for prop in &function.inputs {
                if prop.type_field.eq_ignore_ascii_case("struct") {
                    if !structs.contains_key(&prop.name) {
                        structs.insert(prop.name.clone().to_class_case(), prop.clone());
                    }

                    for inner_component in prop.components.as_ref().unwrap() {
                        inner_structs.extend(Abigen::get_inner_structs(inner_component));
                    }
                }
            }
        }

        for inner_struct in inner_structs {
            if !structs.contains_key(&inner_struct.name) {
                let struct_name = inner_struct.name.to_class_case();
                structs.insert(struct_name, inner_struct);
            }
        }

        structs
    }

    // TODO: improve this function
    fn get_inner_structs(prop: &Property) -> Vec<Property> {
        let mut props = Vec::new();
        if prop.type_field.eq_ignore_ascii_case("struct") {
            props.push(prop.clone());

            for inner_prop in prop.components.as_ref().unwrap() {
                let inner = Abigen::get_inner_structs(inner_prop);
                props.extend(inner);
            }
        }

        props
    }

    fn get_custom_enums(abi: &JsonABI) -> HashMap<String, Property> {
        let mut enums = HashMap::new();
        let mut inner_enums: Vec<Property> = Vec::new();
        for function in abi {
            for prop in &function.inputs {
                if prop.type_field.eq_ignore_ascii_case("enum") {
                    if !enums.contains_key(&prop.name) {
                        enums.insert(prop.name.clone().to_class_case(), prop.clone());
                    }

                    for inner_component in prop.components.as_ref().unwrap() {
                        inner_enums.extend(Abigen::get_inner_enums(inner_component));
                    }
                }
            }
        }

        for inner_enum in inner_enums {
            if !enums.contains_key(&inner_enum.name) {
                let struct_name = inner_enum.name.to_class_case();
                enums.insert(struct_name, inner_enum);
            }
        }

        enums
    }

    // TODO: improve this function
    fn get_inner_enums(prop: &Property) -> Vec<Property> {
        let mut props = Vec::new();
        if prop.type_field.eq_ignore_ascii_case("enum") {
            props.push(prop.clone());

            for inner_prop in prop.components.as_ref().unwrap() {
                let inner = Abigen::get_inner_enums(inner_prop);
                props.extend(inner);
            }
        }

        props
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

        let abi_structs = self.abi_structs()?;
        let abi_enums = self.abi_enums()?;

        Ok(quote! {
            pub use #name_mod::*;

            #[allow(clippy::too_many_arguments)]
            mod #name_mod {
                #![allow(clippy::enum_variant_names)]
                #![allow(dead_code)]
                #![allow(unused_imports)]

                use fuels_rs::contract::{Contract, ContractCall};
                use fuels_rs::tokens::{Tokenizable, Token};
                use fuels_rs::types::EnumSelector;

                pub struct #name;

                impl #name {
                    pub fn new() -> Self {
                        Self{}
                    }

                    #contract_functions
                }

                #abi_structs
                #abi_enums
            }
        })
    }

    fn abi_structs(&self) -> Result<TokenStream, Error> {
        let mut structs = TokenStream::new();

        for (name, prop) in &self.custom_structs {
            structs.extend(self.expand_internal_struct(name, prop)?);
        }

        Ok(structs)
    }

    fn abi_enums(&self) -> Result<TokenStream, Error> {
        let mut enums = TokenStream::new();

        for (name, prop) in &self.custom_enums {
            enums.extend(self.expand_internal_enum(name, prop)?);
        }

        Ok(enums)
    }

    fn expand_internal_enum(&self, name: &str, prop: &Property) -> Result<TokenStream, Error> {
        let components = prop.components.as_ref().unwrap();
        let mut fields = Vec::with_capacity(components.len());

        // TODO: find better naming
        let mut enum_selector_builder = Vec::new();

        let name = ident(name);

        for (discriminant, component) in components.iter().enumerate() {
            let component_name = ident(&component.name.to_class_case());
            let field_name = ident(&component.name.to_class_case());

            let param_type = json_abi::parse_param(&component)?;
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
                    fields.push(quote! { #field_name(#ty)});

                    enum_selector_builder.push(quote! {
                        #name::#field_name(value) => (#discriminant as u8, Token::#param_type_string(value))
                    })
                }
            }
        }

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

    fn expand_internal_struct(&self, name: &str, prop: &Property) -> Result<TokenStream, Error> {
        let components = prop.components.as_ref().unwrap();
        let mut fields = Vec::with_capacity(components.len());

        // TODO: find better naming
        let mut inner_tokens_vector = Vec::new();

        for component in components {
            let component_name = ident(&component.name.to_class_case());
            let field_name = ident(&component.name.to_snake_case());

            let param_type = json_abi::parse_param(&component)?;
            match param_type {
                // Case where a struct takes another struct
                ParamType::Struct(_params) => {
                    fields.push(quote! {pub #field_name: #component_name});
                    inner_tokens_vector.push(quote! { tokens.push(self.#field_name.into_token()) });
                }
                // Elementary type
                _ => {
                    let ty = expand_type(&param_type)?;
                    let param_type_string = ident(&param_type.to_string());
                    fields.push(quote! { pub #field_name: #ty});
                    inner_tokens_vector
                        .push(quote! {tokens.push(Token::#param_type_string(self.#field_name))})
                }
            }
        }

        let name = ident(name);

        Ok(quote! {
            #[derive(Clone, Debug, Default, Eq, PartialEq)]
            pub struct #name {
                #( #fields ),*
            }

            impl #name {
                pub fn into_token(self) -> Token {
                    let mut tokens = Vec::new();
                    #( #inner_tokens_vector; )*

                    Token::Struct(tokens)
                }
            }



        })
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
                // ParamType::Struct(_) if fun.inputs.len() == 1 => {
                //     // make sure the tuple gets converted to `Token::Tuple`
                //     quote! {(#name,)}
                // }
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
            ParamType::Array(ty, size) => {
                let ty = self.expand_input_param(fun, param, ty)?;
                Ok(quote! {
                    ::std::vec::Vec<#ty>
                })
            }
            ParamType::Enum(v) => {
                let rust_enum_name = self.custom_enums.get(param).unwrap();
                let ident = ident(&rust_enum_name.name);
                Ok(quote! { #ident })
            }
            ParamType::Struct(_) => {
                let rust_struct_name = self.custom_structs.get(param).unwrap();
                let ident = ident(&rust_struct_name.name);
                Ok(quote! { #ident })
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
    fn generates_bindings_from_contract_file() {
        let bindings = Abigen::new(
            "test",
            "../fuels-rs-examples/examples/takes_ints_returns_bool.json",
        )
        .unwrap()
        .generate()
        .unwrap();
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
    fn custom_struct() {
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

        let contract = Abigen::new("custom", contract).unwrap();

        assert_eq!(1, contract.custom_structs.len());

        assert_eq!(true, contract.custom_structs.contains_key("MyStruct"));

        let bindings = contract.generate().unwrap();
        bindings.write(std::io::stdout()).unwrap();
    }

    #[test]
    fn multiple_custom_types() {
        let contract = r#"
        [
            {
                "type":"contract",
                "inputs":[
                {
                    "name":"MyNestedStruct",
                    "type":"struct",
                    "components":[
                    {
                        "name":"x",
                        "type":"u16"
                    },
                    {
                        "name":"inner_struct",
                        "type":"struct",
                        "components":[
                        {
                            "name":"a",
                            "type":"bool"
                        },
                        {
                            "name":"b",
                            "type":"u8[2]"
                        }
                        ]
                    }
                    ]
                },
                {
                    "name":"MySecondNestedStruct",
                    "type":"struct",
                    "components":[
                    {
                        "name":"x",
                        "type":"u16"
                    },
                    {
                        "name":"second_inner_struct",
                        "type":"struct",
                        "components":[
                        {
                            "name":"third_inner_struct",
                            "type":"struct",
                            "components":[
                            {
                                "name":"foo",
                                "type":"u8"
                            }
                            ]
                        }
                        ]
                    }
                    ]
                }
                ],
                "name":"takes_nested_struct",
                "outputs":[
                
                ]
            }
        ]
        "#;

        // TODO: Continue from here, make this work

        let contract = Abigen::new("custom", contract).unwrap();

        assert_eq!(5, contract.custom_structs.len());

        let expected_custom_struct_names = vec![
            "MyNestedStruct",
            "InnerStruct",
            "ThirdInnerStruct",
            "SecondInnerStruct",
            "MySecondNestedStruct",
        ];

        for name in expected_custom_struct_names {
            assert_eq!(true, contract.custom_structs.contains_key(name));
        }

        let bindings = contract.generate().unwrap();
        bindings.write(std::io::stdout()).unwrap();
    }

    #[test]
    fn custom_enum() {
        let contract = r#"
        [
            {
                "type":"contract",
                "inputs":[
                    {
                        "name":"MyEnum",
                        "type":"enum",
                        "components": [
                            {
                                "name": "x",
                                "type": "u32"
                            },
                            {
                                "name": "y",
                                "type": "bool"
                            }
                        ]
                    }
                ],
                "name":"takes_enum",
                "outputs":[]
            }
        ]
        "#;

        let contract = Abigen::new("custom", contract).unwrap();

        assert_eq!(1, contract.custom_enums.len());
        assert_eq!(0, contract.custom_structs.len());

        assert_eq!(true, contract.custom_enums.contains_key("MyEnum"));

        let bindings = contract.generate().unwrap();
        bindings.write(std::io::stdout()).unwrap();
    }
}

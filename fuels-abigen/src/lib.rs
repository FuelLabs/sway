use fuels_rs::abigen::Abigen;
use fuels_rs::types::ParamType;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::ops::Deref;
use syn::parse::{Parse, ParseStream, Result as ParseResult};
use syn::spanned::Spanned as _;
use syn::{braced, parenthesized, Ident, LitStr, Path, Token};
use syn::{
    parse::Error, parse_macro_input, AttrStyle, Data, DeriveInput, Expr, Field, Fields,
    GenericArgument, Lit, Meta, NestedMeta, PathArguments, Type,
};

#[proc_macro]
pub fn abigen(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as Spanned<ContractArgs>);

    let c = Abigen::new(&args.name, &args.abi).unwrap();

    c.expand().unwrap().into()
}

/// Trait that abstracts functionality for inner data that can be parsed and
/// wrapped with a specific `Span`.
trait ParseInner: Sized {
    fn spanned_parse(input: ParseStream) -> ParseResult<(Span, Self)>;
}

impl<T: Parse> ParseInner for T {
    fn spanned_parse(input: ParseStream) -> ParseResult<(Span, Self)> {
        Ok((input.span(), T::parse(input)?))
    }
}

impl<T: ParseInner> Parse for Spanned<T> {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let (span, value) = T::spanned_parse(input)?;
        Ok(Spanned(span, value))
    }
}

/// A struct that captures `Span` information for inner parsable data.
#[cfg_attr(test, derive(Clone, Debug))]
struct Spanned<T>(Span, T);

impl<T> Spanned<T> {
    /// Retrieves the captured `Span` information for the parsed data.
    pub fn span(&self) -> Span {
        self.0
    }

    /// Retrieves the inner data.
    pub fn into_inner(self) -> T {
        self.1
    }
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

/// Contract procedural macro arguments.
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub(crate) struct ContractArgs {
    name: String,
    abi: String,
    parameters: Option<Vec<ParamType>>,
}

impl ContractArgs {
    fn into_builder(self) -> Result<Abigen, Error> {
        let mut builder = Abigen::new(&self.name, &self.abi).unwrap();

        // for parameter in self.parameters.into_iter() {
        //     builder = match parameter {
        //         Parameter::Methods(methods) => methods.into_iter().fold(builder, |builder, m| {
        //             builder.add_method_alias(m.signature, m.alias)
        //         }),
        //         Parameter::EventDerives(derives) => derives
        //             .into_iter()
        //             .fold(builder, |builder, derive| builder.add_event_derive(derive)),
        //     };
        // }

        Ok(builder)
    }
}

impl ParseInner for ContractArgs {
    fn spanned_parse(input: ParseStream) -> ParseResult<(Span, Self)> {
        // read the contract name
        let name = input.parse::<Ident>()?.to_string();

        // skip the comma
        input.parse::<Token![,]>()?;

        let (span, abi) = {
            let literal = input.parse::<LitStr>()?;
            (literal.span(), literal.value())
        };

        if !input.is_empty() {
            input.parse::<Token![,]>()?;
        }

        Ok((
            span,
            ContractArgs {
                name,
                abi,
                parameters: None,
            },
        ))
    }
}

/// A single procedural macro parameter.
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
enum Parameter {
    Methods(Vec<Method>),
    EventDerives(Vec<String>),
}

// impl Parse for Parameter {
//     fn parse(input: ParseStream) -> ParseResult<Self> {
//         let name = input.call(Ident::parse_any)?;
//         let param = match name.to_string().as_str() {
//             "methods" => {
//                 let content;
//                 braced!(content in input);
//                 let methods = {
//                     let parsed =
//                         content.parse_terminated::<_, Token![;]>(Spanned::<Method>::parse)?;

//                     let mut methods = Vec::with_capacity(parsed.len());
//                     let mut signatures = HashSet::new();
//                     let mut aliases = HashSet::new();
//                     for method in parsed {
//                         if !signatures.insert(method.signature.clone()) {
//                             return Err(ParseError::new(
//                                 method.span(),
//                                 "duplicate method signature in `abigen!` macro invocation",
//                             ));
//                         }
//                         if !aliases.insert(method.alias.clone()) {
//                             return Err(ParseError::new(
//                                 method.span(),
//                                 "duplicate method alias in `abigen!` macro invocation",
//                             ));
//                         }
//                         methods.push(method.into_inner())
//                     }

//                     methods
//                 };

//                 Parameter::Methods(methods)
//             }
//             "event_derives" => {
//                 let content;
//                 parenthesized!(content in input);
//                 let derives = content
//                     .parse_terminated::<_, Token![,]>(Path::parse)?
//                     .into_iter()
//                     .map(|path| path.to_token_stream().to_string())
//                     .collect();
//                 Parameter::EventDerives(derives)
//             }
//             _ => {
//                 return Err(ParseError::new(
//                     name.span(),
//                     format!("unexpected named parameter `{}`", name),
//                 ))
//             }
//         };

//         Ok(param)
//     }
// }

/// An explicitely named contract method.
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
struct Method {
    signature: String,
    alias: String,
}

// impl Parse for Method {
//     fn parse(input: ParseStream) -> ParseResult<Self> {
//         let function = {
//             let name = input.parse::<Ident>()?.to_string();

//             let content;
//             parenthesized!(content in input);
//             let inputs = content
//                 .parse_terminated::<_, Token![,]>(Ident::parse)?
//                 .iter()
//                 .map(|ident| {
//                     let kind = serde_json::from_value(serde_json::json!(&ident.to_string()))
//                         .map_err(|err| ParseError::new(ident.span(), err))?;
//                     Ok(Param {
//                         name: "".into(),
//                         kind,
//                     })
//                 })
//                 .collect::<ParseResult<Vec<_>>>()?;

//             #[allow(deprecated)]
//             Function {
//                 name,
//                 inputs,

//                 // NOTE: The output types and const-ness of the function do not
//                 //   affect its signature.
//                 outputs: vec![],
//                 state_mutability: StateMutability::NonPayable,
//                 constant: false,
//             }
//         };
//         let signature = function.abi_signature();
//         input.parse::<Token![as]>()?;
//         let alias = {
//             let ident = input.parse::<Ident>()?;
//             ident.to_string()
//         };

//         Ok(Method { signature, alias })
//     }
// }

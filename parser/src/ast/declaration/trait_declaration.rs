use super::{FunctionDeclaration, FunctionParameter, TypeInfo};
use crate::error::CompileError;
use crate::parser::{HllParser, Rule};
use either::*;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub(crate) struct TraitDeclaration<'sc> {
    name: &'sc str,
    interface_surface: Vec<TraitFn<'sc>>,
    methods: Vec<FunctionDeclaration<'sc>>,
}

impl<'sc> TraitDeclaration<'sc> {
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> Result<Self, CompileError<'sc>> {
        let mut trait_parts = pair.into_inner();
        let _trait_keyword = trait_parts.next();
        let name = trait_parts.next().unwrap().as_str();
        let methods_and_interface = trait_parts
            .next()
            .map(|if_some: Pair<'sc, Rule>| -> Result<_, CompileError> {
                if_some
                    .into_inner()
                    .map(
                        |fn_sig_or_decl| -> Result<
                            Either<TraitFn<'sc>, FunctionDeclaration<'sc>>,
                            CompileError,
                        > {
                            Ok(match fn_sig_or_decl.as_rule() {
                                Rule::fn_signature => {
                                    Left(TraitFn::parse_from_pair(fn_sig_or_decl)?)
                                }
                                Rule::fn_decl => {
                                    Right(FunctionDeclaration::parse_from_pair(fn_sig_or_decl)?)
                                }
                                _ => unreachable!(),
                            })
                        },
                    )
                    .collect::<Result<Vec<_>, CompileError>>()
            })
            .unwrap_or_else(|| Ok(Vec::new()))?;

        let mut interface_surface = Vec::new();
        let mut methods = Vec::new();
        methods_and_interface.into_iter().for_each(|x| match x {
            Left(x) => interface_surface.push(x),
            Right(x) => methods.push(x),
        });

        Ok(TraitDeclaration {
            name,
            interface_surface,
            methods,
        })
    }
}

#[derive(Debug, Clone)]
struct TraitFn<'sc> {
    pub(crate) name: &'sc str,
    pub(crate) parameters: Vec<FunctionParameter<'sc>>,
    pub(crate) return_type: TypeInfo<'sc>,
}

impl<'sc> TraitFn<'sc> {
    fn parse_from_pair(pair: Pair<'sc, Rule>) -> Result<Self, CompileError<'sc>> {
        let mut signature = pair.clone().into_inner();
        let _fn_keyword = signature.next().unwrap();
        let name = signature.next().unwrap().as_str();
        let parameters = signature.next().unwrap();
        let parameters = FunctionParameter::list_from_pairs(parameters.into_inner())?;
        let return_type_signal = signature.next();
        let return_type = match return_type_signal {
            Some(_) => TypeInfo::parse_from_pair(signature.next().unwrap())?,
            None => TypeInfo::Unit,
        };

        Ok(TraitFn {
            name,
            parameters,
            return_type,
        })
    }
}

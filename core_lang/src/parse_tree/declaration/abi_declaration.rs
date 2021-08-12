use super::FunctionDeclaration;
use super::TraitFn;
use crate::parser::Rule;
use crate::{error::*, Ident};
use pest::iterators::Pair;
use pest::Span;

/// An `abi` declaration, which declares an interface for a contract
/// to implement or for a caller to use to call a contract.
#[derive(Debug, Clone)]
pub struct AbiDeclaration<'sc> {
    /// The name of the abi trait (also known as a "contract trait")
    pub(crate) name: Ident<'sc>,
    /// The methods a contract is required to implement in order opt in to this interface
    pub(crate) interface_surface: Vec<TraitFn<'sc>>,
    /// The methods provided to a contract "for free" upon opting in to this interface
    pub(crate) methods: Vec<FunctionDeclaration<'sc>>,
    pub(crate) span: Span<'sc>,
}

impl<'sc> AbiDeclaration<'sc> {
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let span = pair.as_span();
        let mut iter = pair.into_inner();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let _abi_keyword = iter.next().expect("guaranteed by grammar");
        let name = eval!(
            Ident::parse_from_pair,
            warnings,
            errors,
            iter.next().expect("guaranteed by grammar"),
            return err(warnings, errors)
        );
        let mut interface_surface = vec![];
        let mut methods = vec![];
        let trait_methods = iter.next().expect("guaranteed by grammar");
        for func in trait_methods.into_inner() {
            match func.as_rule() {
                Rule::fn_signature => interface_surface.push(eval!(
                    TraitFn::parse_from_pair,
                    warnings,
                    errors,
                    func,
                    continue
                )),
                Rule::fn_decl => methods.push(eval!(
                    FunctionDeclaration::parse_from_pair,
                    warnings,
                    errors,
                    func,
                    continue
                )),
                _ => unreachable!("guaranteed by grammar"),
            }
        }
        ok(
            AbiDeclaration {
                methods,
                interface_surface,
                name,
                span,
            },
            warnings,
            errors,
        )
    }
}

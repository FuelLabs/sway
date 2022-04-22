use super::{FunctionDeclaration, TraitFn};
use crate::{build_config::BuildConfig, error::*, parse_tree::ident, parser::Rule};

use sway_types::{ident::Ident, span::Span};

use pest::iterators::Pair;

/// An `abi` declaration, which declares an interface for a contract
/// to implement or for a caller to use to call a contract.
#[derive(Debug, Clone)]
pub struct AbiDeclaration {
    /// The name of the abi trait (also known as a "contract trait")
    pub(crate) name: Ident,
    /// The methods a contract is required to implement in order opt in to this interface
    pub(crate) interface_surface: Vec<TraitFn>,
    /// The methods provided to a contract "for free" upon opting in to this interface
    pub(crate) methods: Vec<FunctionDeclaration>,
    pub(crate) span: Span,
}

impl AbiDeclaration {
    pub(crate) fn parse_from_pair(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Self> {
        let span = Span::from_pest(pair.as_span(), config.map(|c| c.path()));
        let mut iter = pair.into_inner();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let _abi_keyword = iter.next().expect("guaranteed by grammar");
        let name = check!(
            ident::parse_from_pair(iter.next().expect("guaranteed by grammar"), config),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut interface_surface = vec![];
        let mut methods = vec![];
        let trait_methods = iter.next().expect("guaranteed by grammar");
        for func in trait_methods.into_inner() {
            match func.as_rule() {
                Rule::fn_signature => {
                    let fn_sig = check!(
                        TraitFn::parse_from_pair(func, config),
                        continue,
                        warnings,
                        errors
                    );
                    interface_surface.push(fn_sig);
                }
                Rule::fn_decl => methods.push(check!(
                    FunctionDeclaration::parse_from_pair(func, config),
                    continue,
                    warnings,
                    errors
                )),
                x => unreachable!("guaranteed to not be here: {:?}", x),
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

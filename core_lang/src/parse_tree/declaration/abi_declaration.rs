use super::FunctionDeclaration;
use super::TraitFn;
use crate::build_config::BuildConfig;
use crate::parser::Rule;
use crate::span::Span;
use crate::{error::*, Ident};
use pest::iterators::Pair;

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
    pub(crate) fn parse_from_pair(
        pair: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<'sc, Self> {
        let span = Span {
            span: pair.as_span(),
            path: config.map(|c| c.file_path.clone()),
        };
        let mut iter = pair.into_inner();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let _abi_keyword = iter.next().expect("guaranteed by grammar");
        let name = check!(
            Ident::parse_from_pair(iter.next().expect("guaranteed by grammar"), config),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut interface_surface = vec![];
        let mut methods = vec![];
        let trait_methods = iter.next().expect("guaranteed by grammar");
        for func in trait_methods.into_inner() {
            match func.as_rule() {
                Rule::fn_signature => interface_surface.push(check!(
                    TraitFn::parse_from_pair(func, config),
                    continue,
                    warnings,
                    errors
                )),
                Rule::fn_decl => methods.push(check!(
                    FunctionDeclaration::parse_from_pair(func, config),
                    continue,
                    warnings,
                    errors
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

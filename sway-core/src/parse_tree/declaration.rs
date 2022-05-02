mod abi;
mod constant;
mod r#enum;
pub mod function;
mod impl_trait;
mod reassignment;
mod storage;
mod r#struct;
mod r#trait;
mod type_argument;
mod type_parameter;
mod variable;

pub(crate) use abi::*;
pub use constant::*;
pub use function::*;
pub(crate) use impl_trait::*;
pub use r#enum::*;
pub use r#struct::*;
pub use r#trait::*;
pub(crate) use reassignment::*;
pub use storage::*;
pub(crate) use type_argument::*;
pub(crate) use type_parameter::*;
pub use variable::*;

use crate::{build_config::BuildConfig, error::*, parser::Rule};

use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub enum Declaration {
    VariableDeclaration(VariableDeclaration),
    FunctionDeclaration(FunctionDeclaration),
    TraitDeclaration(TraitDeclaration),
    StructDeclaration(StructDeclaration),
    EnumDeclaration(EnumDeclaration),
    Reassignment(Reassignment),
    ImplTrait(ImplTrait),
    ImplSelf(ImplSelf),
    AbiDeclaration(AbiDeclaration),
    ConstantDeclaration(ConstantDeclaration),
    StorageDeclaration(StorageDeclaration),
}
impl Declaration {
    pub(crate) fn parse_non_var_from_pair(
        decl: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Vec<Self>> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut pair = decl.into_inner();
        let decl_inner = pair.next().unwrap();
        let parsed_declaration = match decl_inner.as_rule() {
            Rule::non_var_decl => check!(
                Self::parse_non_var_from_pair(decl_inner, config),
                return err(warnings, errors),
                warnings,
                errors
            ),
            Rule::fn_decl => {
                let fn_decl = check!(
                    FunctionDeclaration::parse_from_pair(decl_inner, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                vec![Declaration::FunctionDeclaration(fn_decl)]
            }
            Rule::trait_decl => vec![Declaration::TraitDeclaration(check!(
                TraitDeclaration::parse_from_pair(decl_inner, config,),
                return err(warnings, errors),
                warnings,
                errors
            ))],
            Rule::struct_decl => {
                let struct_decl = check!(
                    StructDeclaration::parse_from_pair(decl_inner, config,),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                vec![Declaration::StructDeclaration(struct_decl)]
            }
            Rule::enum_decl => {
                let enum_decl = check!(
                    EnumDeclaration::parse_from_pair(decl_inner, config,),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                vec![Declaration::EnumDeclaration(enum_decl)]
            }
            Rule::impl_trait => vec![Declaration::ImplTrait(check!(
                ImplTrait::parse_from_pair(decl_inner, config,),
                return err(warnings, errors),
                warnings,
                errors
            ))],
            Rule::impl_self => vec![Declaration::ImplSelf(check!(
                ImplSelf::parse_from_pair(decl_inner, config,),
                return err(warnings, errors),
                warnings,
                errors
            ))],
            Rule::abi_decl => {
                let abi_decl = check!(
                    AbiDeclaration::parse_from_pair(decl_inner, config,),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                vec![Declaration::AbiDeclaration(abi_decl)]
            }
            Rule::const_decl => {
                let res_result = check!(
                    ConstantDeclaration::parse_from_pair(decl_inner, config,),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let mut decls = res_result
                    .var_decls
                    .into_iter()
                    .map(Declaration::VariableDeclaration)
                    .collect::<Vec<_>>();
                decls.push(Declaration::ConstantDeclaration(res_result.value));
                decls
            }
            Rule::storage_decl => {
                let res_result = check!(
                    StorageDeclaration::parse_from_pair(decl_inner, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let mut decls = res_result
                    .var_decls
                    .into_iter()
                    .map(Declaration::VariableDeclaration)
                    .collect::<Vec<_>>();
                decls.push(Declaration::StorageDeclaration(res_result.value));
                decls
            }
            a => unreachable!("declarations don't have any other sub-types: {:?}", a),
        };
        ok(parsed_declaration, warnings, errors)
    }

    /// This function returns a `Vec<Self>` because of actions taken during
    /// desugaring. Given this variable declaration:
    ///
    /// ```ignore
    /// let x = (1, 2);
    /// let (a, b) = x;
    /// ```
    ///
    /// This gets desugared to:
    ///
    /// ```ignore
    /// let x = (1, 2);
    /// let a = x.0;
    /// let b = x.1;
    /// ```
    ///
    /// So, the `var_decl` rule has the possibility of returning more than
    /// one `VariableDeclaration`, thus we may need to return multiple
    /// `Declaration`s.
    pub(crate) fn parse_from_pair(
        decl: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Vec<Self>> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut pair = decl.into_inner();
        let decl_inner = pair.next().unwrap();
        let parsed_declarations = match decl_inner.as_rule() {
            Rule::fn_decl => vec![Declaration::FunctionDeclaration(check!(
                FunctionDeclaration::parse_from_pair(decl_inner, config),
                return err(warnings, errors),
                warnings,
                errors
            ))],
            Rule::var_decl => {
                let var_decls = check!(
                    VariableDeclaration::parse_from_pair(decl_inner, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                var_decls
                    .into_iter()
                    .map(Declaration::VariableDeclaration)
                    .collect::<Vec<_>>()
            }
            Rule::trait_decl => vec![Declaration::TraitDeclaration(check!(
                TraitDeclaration::parse_from_pair(decl_inner, config),
                return err(warnings, errors),
                warnings,
                errors
            ))],
            Rule::struct_decl => vec![Declaration::StructDeclaration(check!(
                StructDeclaration::parse_from_pair(decl_inner, config),
                return err(warnings, errors),
                warnings,
                errors
            ))],
            Rule::non_var_decl => check!(
                Self::parse_non_var_from_pair(decl_inner, config),
                return err(warnings, errors),
                warnings,
                errors
            ),
            Rule::enum_decl => vec![Declaration::EnumDeclaration(check!(
                EnumDeclaration::parse_from_pair(decl_inner, config),
                return err(warnings, errors),
                warnings,
                errors
            ))],
            Rule::reassignment => {
                let res_result = check!(
                    Reassignment::parse_from_pair(decl_inner, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let mut decls = res_result
                    .var_decls
                    .into_iter()
                    .map(Declaration::VariableDeclaration)
                    .collect::<Vec<_>>();
                decls.push(Declaration::Reassignment(res_result.value));
                decls
            }
            Rule::impl_trait => vec![Declaration::ImplTrait(check!(
                ImplTrait::parse_from_pair(decl_inner, config),
                return err(warnings, errors),
                warnings,
                errors
            ))],
            Rule::impl_self => vec![Declaration::ImplSelf(check!(
                ImplSelf::parse_from_pair(decl_inner, config),
                return err(warnings, errors),
                warnings,
                errors
            ))],
            Rule::const_decl => {
                let res_result = check!(
                    ConstantDeclaration::parse_from_pair(decl_inner, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let mut decls = res_result
                    .var_decls
                    .into_iter()
                    .map(Declaration::VariableDeclaration)
                    .collect::<Vec<_>>();
                decls.push(Declaration::ConstantDeclaration(res_result.value));
                decls
            }
            Rule::abi_decl => vec![Declaration::AbiDeclaration(check!(
                AbiDeclaration::parse_from_pair(decl_inner, config),
                return err(warnings, errors),
                warnings,
                errors
            ))],
            a => unreachable!("declarations don't have any other sub-types: {:?}", a),
        };
        ok(parsed_declarations, warnings, errors)
    }
}

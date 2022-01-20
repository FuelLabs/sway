

use crate::{
    error::*, type_engine::*, CallPath,
    CompileResult, Ident, TypeInfo, TypedDeclaration, TypedFunctionDeclaration,
};

use sway_types::span::{Span};

use std::collections::{BTreeMap, HashMap};

pub mod arena;
pub use arena::*;

type ModuleName = String;
type TraitName = CallPath;
/// A namespace represents all items that exist either via declaration or importing.
#[derive(Clone, Debug, Default)]
pub struct Namespace {
    // This is a BTreeMap because we rely on its ordering being consistent. See
    // [Namespace::get_all_declared_symbols] -- we need that iterator to have a deterministic
    // order.
    symbols: BTreeMap<Ident, TypedDeclaration>,
    implemented_traits: HashMap<(TraitName, TypeInfo), Vec<TypedFunctionDeclaration>>,
    // Any other modules within this scope, where a module is a namespace associated with an identifier.
    // This is a BTreeMap because we rely on its ordering being consistent. See
    // [Namespace::get_all_imported_modules] -- we need that iterator to have a deterministic
    // order.
    modules: BTreeMap<ModuleName, NamespaceRef>,
    use_synonyms: HashMap<Ident, Vec<Ident>>,
    // Represents an alternative name for a symbol.
    use_aliases: HashMap<String, Ident>,
}

impl Namespace {
    pub fn get_all_declared_symbols(&self) -> impl Iterator<Item = &TypedDeclaration> {
        self.symbols.values()
    }

    pub fn get_all_imported_modules(&self) -> impl Iterator<Item = &NamespaceRef> {
        self.modules.values()
    }

    pub(crate) fn insert(&mut self, name: Ident, item: TypedDeclaration) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        if self.symbols.get(&name).is_some() {
            match item {
                TypedDeclaration::EnumDeclaration { .. }
                | TypedDeclaration::StructDeclaration { .. } => {
                    errors.push(CompileError::ShadowsOtherSymbol {
                        span: name.span().clone(),
                        name: name.as_str().to_string(),
                    });
                    return err(warnings, errors);
                }
                _ => {
                    warnings.push(CompileWarning {
                        span: name.span().clone(),
                        warning_content: Warning::ShadowsOtherSymbol {
                            name: name.span().as_str().to_string(),
                        },
                    });
                }
            }
        }
        self.symbols.insert(name, item);
        ok((), warnings, errors)
    }

    pub(crate) fn insert_trait_implementation(
        &mut self,
        trait_name: CallPath,
        type_implementing_for: TypeInfo,
        functions_buf: Vec<TypedFunctionDeclaration>,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let errors = vec![];
        let new_prefixes = if trait_name.prefixes.is_empty() {
            self.use_synonyms
                .get(&trait_name.suffix)
                .unwrap_or(&trait_name.prefixes)
                .clone()
        } else {
            trait_name.prefixes
        };
        let trait_name = CallPath {
            suffix: trait_name.suffix,
            prefixes: new_prefixes,
        };
        if self
            .implemented_traits
            .insert((trait_name.clone(), type_implementing_for), functions_buf)
            .is_some()
        {
            warnings.push(CompileWarning {
                warning_content: Warning::OverridingTraitImplementation,
                span: trait_name.span(),
            })
        }
        ok((), warnings, errors)
    }

    pub fn insert_module(&mut self, module_name: String, ix: NamespaceRef) {
        self.modules.insert(module_name, ix);
    }

    pub fn insert_dependency_module(&mut self, module_name: String, ix: NamespaceRef) {
        self.insert_module(module_name, ix)
    }

    pub(crate) fn get_methods_for_type(&self, r#type: TypeId) -> Vec<TypedFunctionDeclaration> {
        let mut methods = vec![];
        let r#type = crate::type_engine::look_up_type_id(r#type);
        for ((_trait_name, type_info), l_methods) in &self.implemented_traits {
            if *type_info == r#type {
                methods.append(&mut l_methods.clone());
            }
        }
        methods
    }

    pub(crate) fn get_tuple_elems(
        &self,
        ty: TypeId,
        debug_string: impl Into<String>,
        debug_span: &Span,
    ) -> CompileResult<Vec<TypeId>> {
        let warnings = vec![];
        let errors = vec![];
        let ty = crate::type_engine::look_up_type_id(ty);
        match ty {
            TypeInfo::Tuple(elems) => ok(elems, warnings, errors),
            TypeInfo::ErrorRecovery => err(warnings, errors),
            a => err(
                vec![],
                vec![CompileError::NotATuple {
                    name: debug_string.into(),
                    span: debug_span.clone(),
                    actually: a.friendly_type_str(),
                }],
            ),
        }
    }
}

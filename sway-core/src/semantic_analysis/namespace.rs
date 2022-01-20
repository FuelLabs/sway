use super::ast_node::{
    OwnedTypedStructField, TypedEnumDeclaration, TypedEnumVariant, TypedStructDeclaration,
    TypedStructField,
};

use crate::{
    error::*, parse_tree::Visibility, semantic_analysis::TypedExpression, type_engine::*, CallPath,
    CompileResult, Ident, TypeInfo, TypedDeclaration, TypedFunctionDeclaration,
};

use sway_types::span::{join_spans, Span};

use std::collections::{BTreeMap, HashMap, VecDeque};

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

    /// Returns a tuple where the first element is the [ResolvedType] of the actual expression,
    /// and the second is the [ResolvedType] of its parent, for control-flow analysis.
    pub(crate) fn find_subfield_type(
        &self,
        subfield_exp: &[Ident],
    ) -> CompileResult<(TypeId, TypeId)> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut ident_iter = subfield_exp.iter().peekable();
        let first_ident = ident_iter.next().unwrap();
        let symbol = match self.symbols.get(first_ident) {
            Some(s) => s,
            None => {
                errors.push(CompileError::UnknownVariable {
                    var_name: first_ident.as_str().to_string(),
                    span: first_ident.span().clone(),
                });
                return err(warnings, errors);
            }
        };
        if ident_iter.peek().is_none() {
            let ty = check!(
                symbol.return_type(),
                return err(warnings, errors),
                warnings,
                errors
            );
            return ok((ty, ty), warnings, errors);
        }
        let mut symbol = check!(
            symbol.return_type(),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut type_fields =
            self.get_struct_type_fields(symbol, first_ident.as_str(), first_ident.span());
        warnings.append(&mut type_fields.warnings);
        errors.append(&mut type_fields.errors);
        let (mut fields, struct_name) = match type_fields.value {
            // if it is missing, the error message comes from within the above method
            // so we don't need to re-add it here
            None => return err(warnings, errors),
            Some(value) => value,
        };

        let mut parent_rover = symbol;

        for ident in ident_iter {
            // find the ident in the currently available fields
            let OwnedTypedStructField { r#type, .. } =
                match fields.iter().find(|x| x.name == ident.as_str()) {
                    Some(field) => field.clone(),
                    None => {
                        // gather available fields for the error message
                        let available_fields =
                            fields.iter().map(|x| x.name.as_str()).collect::<Vec<_>>();

                        errors.push(CompileError::FieldNotFound {
                            field_name: ident.clone(),
                            struct_name,
                            available_fields: available_fields.join(", "),
                            span: ident.span().clone(),
                        });
                        return err(warnings, errors);
                    }
                };

            match crate::type_engine::look_up_type_id(r#type) {
                TypeInfo::Struct {
                    fields: ref l_fields,
                    ..
                } => {
                    parent_rover = symbol;
                    fields = l_fields.clone();
                    symbol = r#type;
                }
                _ => {
                    fields = vec![];
                    parent_rover = symbol;
                    symbol = r#type;
                }
            }
        }
        ok((symbol, parent_rover), warnings, errors)
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

    /// given a declaration that may refer to a variable which contains a struct,
    /// find that struct's fields and name for use in determining if a subfield expression is valid
    /// e.g. foo.bar.baz
    /// is foo a struct? does it contain a field bar? is foo.bar a struct? does foo.bar contain a
    /// field baz? this is the problem this function addresses
    pub(crate) fn get_struct_type_fields(
        &self,
        ty: TypeId,
        debug_string: impl Into<String>,
        debug_span: &Span,
    ) -> CompileResult<(Vec<OwnedTypedStructField>, String)> {
        let ty = crate::type_engine::look_up_type_id(ty);
        match ty {
            TypeInfo::Struct { name, fields } => ok((fields.to_vec(), name), vec![], vec![]),
            // If we hit `ErrorRecovery` then the source of that type should have populated
            // the error buffer elsewhere
            TypeInfo::ErrorRecovery => err(vec![], vec![]),
            a => err(
                vec![],
                vec![CompileError::NotAStruct {
                    name: debug_string.into(),
                    span: debug_span.clone(),
                    actually: a.friendly_type_str(),
                }],
            ),
        }
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

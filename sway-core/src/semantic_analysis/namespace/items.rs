use crate::{
    decl_engine::*,
    engine_threading::Engines,
    error::*,
    language::{
        ty::{self, TyDecl, TyStorageDecl},
        CallPath,
    },
    namespace::*,
    type_system::*,
};

use super::TraitMap;

use sway_error::error::CompileError;
use sway_types::{span::Span, Spanned};

use std::sync::Arc;

/// Is this a glob (`use foo::*;`) import?
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum GlobImport {
    Yes,
    No,
}

pub(super) type SymbolMap = im::OrdMap<Ident, ty::TyDecl>;
pub(super) type UseSynonyms = im::HashMap<Ident, (Vec<Ident>, GlobImport, ty::TyDecl)>;
pub(super) type UseAliases = im::HashMap<String, Ident>;

/// The set of items that exist within some lexical scope via declaration or importing.
#[derive(Clone, Debug, Default)]
pub struct Items {
    /// An ordered map from `Ident`s to their associated typed declarations.
    pub(crate) symbols: SymbolMap,
    pub(crate) implemented_traits: TraitMap,
    /// Represents the absolute path from which a symbol was imported.
    ///
    /// For example, in `use ::foo::bar::Baz;`, we store a mapping from the symbol `Baz` to its
    /// path `foo::bar::Baz`.
    pub(crate) use_synonyms: UseSynonyms,
    /// Represents an alternative name for an imported symbol.
    ///
    /// Aliases are introduced with syntax like `use foo::bar as baz;` syntax, where `baz` is an
    /// alias for `bar`.
    pub(crate) use_aliases: UseAliases,
    /// If there is a storage declaration (which are only valid in contracts), store it here.
    pub(crate) declared_storage: Option<DeclRefStorage>,
}

impl Items {
    /// Immutable access to the inner symbol map.
    pub fn symbols(&self) -> &SymbolMap {
        &self.symbols
    }

    pub fn apply_storage_load(
        &self,
        engines: &Engines,
        fields: Vec<Ident>,
        storage_fields: &[ty::TyStorageField],
    ) -> CompileResult<(ty::TyStorageAccess, TypeId)> {
        let warnings = vec![];
        let mut errors = vec![];
        let type_engine = engines.te();
        let decl_engine = engines.de();
        match self.declared_storage {
            Some(ref decl_ref) => {
                let storage = decl_engine.get_storage(&decl_ref.id().clone());
                storage.apply_storage_load(type_engine, decl_engine, fields, storage_fields)
            }
            None => {
                errors.push(CompileError::NoDeclaredStorage {
                    span: fields[0].span(),
                });
                err(warnings, errors)
            }
        }
    }

    pub fn set_storage_declaration(&mut self, decl_ref: DeclRefStorage) -> CompileResult<()> {
        if self.declared_storage.is_some() {
            return err(
                vec![],
                vec![CompileError::MultipleStorageDeclarations {
                    span: decl_ref.span(),
                }],
            );
        }
        self.declared_storage = Some(decl_ref);
        ok((), vec![], vec![])
    }

    pub fn get_all_declared_symbols(&self) -> impl Iterator<Item = &Ident> {
        self.symbols().keys()
    }

    pub(crate) fn insert_symbol(&mut self, name: Ident, item: ty::TyDecl) -> CompileResult<()> {
        let mut errors = vec![];

        let append_shadowing_error =
            |decl: &ty::TyDecl, item: &ty::TyDecl, errors: &mut Vec<CompileError>| {
                use ty::TyDecl::*;
                match (decl, &item) {
                // variable shadowing a constant
                // constant shadowing a constant
                (
                    ConstantDecl { .. },
                    VariableDecl { .. } | ConstantDecl { .. },
                )
                // constant shadowing a variable
                | (VariableDecl { .. }, ConstantDecl { .. })
                // type or type alias shadowing another type or type alias
                // trait/abi shadowing another trait/abi
                // type or type alias shadowing a trait/abi, or vice versa
                | (
                    StructDecl { .. }
                    | EnumDecl { .. }
                    | TypeAliasDecl { .. }
                    | TraitDecl { .. }
                    | AbiDecl { .. },
                    StructDecl { .. }
                    | EnumDecl { .. }
                    | TypeAliasDecl { .. }
                    | TraitDecl { .. }
                    | AbiDecl { .. },
                ) => errors.push(CompileError::NameDefinedMultipleTimes { name: name.to_string(), span: name.span() }),
                // Generic parameter shadowing another generic parameter
                (GenericTypeForFunctionScope { .. }, GenericTypeForFunctionScope { .. }) => {
                    errors.push(CompileError::GenericShadowsGeneric { name: name.clone() });
                }
                _ => {}
            }
            };

        if let Some(decl) = self.symbols.get(&name) {
            append_shadowing_error(decl, &item, &mut errors);
        }

        if let Some((_, GlobImport::No, decl)) = self.use_synonyms.get(&name) {
            append_shadowing_error(decl, &item, &mut errors);
        }

        self.symbols.insert(name, item);
        ok((), vec![], errors)
    }

    pub(crate) fn check_symbol(&self, name: &Ident) -> Result<&ty::TyDecl, CompileError> {
        self.symbols
            .get(name)
            .ok_or_else(|| CompileError::SymbolNotFound {
                name: name.clone(),
                span: name.span(),
            })
    }

    pub(crate) fn insert_trait_implementation_for_type(
        &mut self,
        engines: &Engines,
        type_id: TypeId,
    ) {
        self.implemented_traits.insert_for_type(engines, type_id);
    }

    pub fn get_items_for_type(&self, engines: &Engines, type_id: TypeId) -> Vec<ty::TyTraitItem> {
        self.implemented_traits.get_items_for_type(engines, type_id)
    }

    pub fn get_impl_spans_for_decl(&self, engines: &Engines, ty_decl: &TyDecl) -> Vec<Span> {
        ty_decl
            .return_type(engines)
            .value
            .map(|type_id| {
                self.implemented_traits
                    .get_impl_spans_for_type(engines, &type_id)
            })
            .unwrap_or_default()
    }

    pub fn get_impl_spans_for_type(&self, engines: &Engines, type_id: &TypeId) -> Vec<Span> {
        self.implemented_traits
            .get_impl_spans_for_type(engines, type_id)
    }

    pub fn get_impl_spans_for_trait_name(&self, trait_name: &CallPath) -> Vec<Span> {
        self.implemented_traits
            .get_impl_spans_for_trait_name(trait_name)
    }

    pub fn get_methods_for_type(&self, engines: &Engines, type_id: TypeId) -> Vec<DeclRefFunction> {
        self.get_items_for_type(engines, type_id)
            .into_iter()
            .filter_map(|item| match item {
                ty::TyTraitItem::Fn(decl_ref) => Some(decl_ref),
                ty::TyTraitItem::Constant(_decl_ref) => None,
            })
            .collect::<Vec<_>>()
    }

    pub(crate) fn has_storage_declared(&self) -> bool {
        self.declared_storage.is_some()
    }

    pub fn get_declared_storage(&self, decl_engine: &DeclEngine) -> Option<TyStorageDecl> {
        self.declared_storage
            .as_ref()
            .map(|decl_ref| decl_engine.get_storage(decl_ref))
    }

    pub(crate) fn get_storage_field_descriptors(
        &self,
        decl_engine: &DeclEngine,
    ) -> CompileResult<Vec<ty::TyStorageField>> {
        let warnings = vec![];
        let mut errors = vec![];
        match self.get_declared_storage(decl_engine) {
            Some(storage) => ok(storage.fields, warnings, errors),
            None => {
                let msg = "unknown source location";
                let span = Span::new(Arc::from(msg), 0, msg.len(), None).unwrap();
                errors.push(CompileError::NoDeclaredStorage { span });
                err(warnings, errors)
            }
        }
    }

    /// Returns a tuple where the first element is the [ResolvedType] of the actual expression, and
    /// the second is the [ResolvedType] of its parent, for control-flow analysis.
    pub(crate) fn find_subfield_type(
        &self,
        engines: &Engines,
        base_name: &Ident,
        projections: &[ty::ProjectionKind],
    ) -> CompileResult<(TypeId, TypeId)> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let type_engine = engines.te();
        let decl_engine = engines.de();

        let symbol = match self.symbols.get(base_name).cloned() {
            Some(s) => s,
            None => {
                errors.push(CompileError::UnknownVariable {
                    var_name: base_name.clone(),
                    span: base_name.span(),
                });
                return err(warnings, errors);
            }
        };
        let mut symbol = check!(
            symbol.return_type(engines),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut symbol_span = base_name.span();
        let mut parent_rover = symbol;
        let mut full_name_for_error = base_name.to_string();
        let mut full_span_for_error = base_name.span();
        for projection in projections {
            let resolved_type = match type_engine.to_typeinfo(symbol, &symbol_span) {
                Ok(resolved_type) => resolved_type,
                Err(error) => {
                    errors.push(CompileError::TypeError(error));
                    return err(warnings, errors);
                }
            };
            match (resolved_type, projection) {
                (
                    TypeInfo::Struct(decl_ref),
                    ty::ProjectionKind::StructField { name: field_name },
                ) => {
                    let struct_decl = decl_engine.get_struct(&decl_ref);
                    let field_type_opt = {
                        struct_decl.fields.iter().find_map(
                            |ty::TyStructField {
                                 type_argument,
                                 name,
                                 ..
                             }| {
                                if name == field_name {
                                    Some(type_argument.type_id)
                                } else {
                                    None
                                }
                            },
                        )
                    };
                    let field_type = match field_type_opt {
                        Some(field_type) => field_type,
                        None => {
                            // gather available fields for the error message
                            let available_fields = struct_decl
                                .fields
                                .iter()
                                .map(|field| field.name.as_str())
                                .collect::<Vec<_>>();

                            errors.push(CompileError::FieldNotFound {
                                field_name: field_name.clone(),
                                struct_name: struct_decl.call_path.suffix,
                                available_fields: available_fields.join(", "),
                                span: field_name.span(),
                            });
                            return err(warnings, errors);
                        }
                    };
                    parent_rover = symbol;
                    symbol = field_type;
                    symbol_span = field_name.span().clone();
                    full_name_for_error.push_str(field_name.as_str());
                    full_span_for_error =
                        Span::join(full_span_for_error, field_name.span().clone());
                }
                (TypeInfo::Tuple(fields), ty::ProjectionKind::TupleField { index, index_span }) => {
                    let field_type_opt = {
                        fields
                            .get(*index)
                            .map(|TypeArgument { type_id, .. }| type_id)
                    };
                    let field_type = match field_type_opt {
                        Some(field_type) => field_type,
                        None => {
                            errors.push(CompileError::TupleIndexOutOfBounds {
                                index: *index,
                                count: fields.len(),
                                span: Span::join(full_span_for_error, index_span.clone()),
                            });
                            return err(warnings, errors);
                        }
                    };
                    parent_rover = symbol;
                    symbol = *field_type;
                    symbol_span = index_span.clone();
                    full_name_for_error.push_str(&index.to_string());
                    full_span_for_error = Span::join(full_span_for_error, index_span.clone());
                }
                (
                    TypeInfo::Array(elem_ty, _),
                    ty::ProjectionKind::ArrayIndex { index_span, .. },
                ) => {
                    parent_rover = symbol;
                    symbol = elem_ty.type_id;
                    symbol_span = index_span.clone();
                    full_span_for_error = index_span.clone();
                }
                (actually, ty::ProjectionKind::StructField { .. }) => {
                    errors.push(CompileError::FieldAccessOnNonStruct {
                        span: full_span_for_error,
                        actually: engines.help_out(actually).to_string(),
                    });
                    return err(warnings, errors);
                }
                (actually, ty::ProjectionKind::TupleField { .. }) => {
                    errors.push(CompileError::NotATuple {
                        name: full_name_for_error,
                        span: full_span_for_error,
                        actually: engines.help_out(actually).to_string(),
                    });
                    return err(warnings, errors);
                }
                (actually, ty::ProjectionKind::ArrayIndex { .. }) => {
                    errors.push(CompileError::NotIndexable {
                        name: full_name_for_error,
                        span: full_span_for_error,
                        actually: engines.help_out(actually).to_string(),
                    });
                    return err(warnings, errors);
                }
            }
        }
        ok((symbol, parent_rover), warnings, errors)
    }
}

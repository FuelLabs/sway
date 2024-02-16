use crate::{
    decl_engine::*,
    engine_threading::Engines,
    language::{
        ty::{self, StructAccessInfo, TyDecl, TyStorageDecl},
        CallPath,
    },
    namespace::*,
    semantic_analysis::{ast_node::ConstShadowingMode, GenericShadowingMode},
    type_system::*,
};

use super::TraitMap;

use sway_error::{
    error::{CompileError, StructFieldUsageContext},
    handler::{ErrorEmitted, Handler},
};
use sway_types::{span::Span, Spanned};

use std::sync::Arc;

/// Is this a glob (`use foo::*;`) import?
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum GlobImport {
    Yes,
    No,
}

pub(super) type SymbolMap = im::OrdMap<Ident, ty::TyDecl>;
// The final `bool` field of `UseSynonyms` is true if the `Vec<Ident>` path is absolute.
pub(super) type UseSynonyms = im::HashMap<Ident, (Vec<Ident>, GlobImport, ty::TyDecl, bool)>;
pub(super) type UseAliases = im::HashMap<String, Ident>;

/// Represents a lexical scope integer-based identifier, which can be used to reference
/// specific a lexical scope.
pub type LexicalScopeId = usize;

/// Represents a lexical scope path, a vector of lexical scope identifiers, which specifies
/// the path from root to a specific lexical scope in the hierarchy.
pub type LexicalScopePath = Vec<LexicalScopeId>;

/// A `LexicalScope` contains a set of all items that exist within the lexical scope via declaration or
/// importing, along with all its associated hierarchical scopes.
#[derive(Clone, Debug, Default)]
pub struct LexicalScope {
    /// The set of symbols, implementations, synonyms and aliases present within this scope.
    pub items: Items,
    /// The set of available scopes defined inside this scope's hierarchy.
    pub children: Vec<LexicalScopeId>,
    /// The parent scope associated with this scope. Will be None for a root scope.
    pub parent: Option<LexicalScopeId>,
}

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
        handler: &Handler,
        engines: &Engines,
        namespace: &Namespace,
        fields: Vec<Ident>,
        storage_fields: &[ty::TyStorageField],
        storage_keyword_span: Span,
    ) -> Result<(ty::TyStorageAccess, TypeId), ErrorEmitted> {
        match self.declared_storage {
            Some(ref decl_ref) => {
                let storage = engines.de().get_storage(&decl_ref.id().clone());
                storage.apply_storage_load(
                    handler,
                    engines,
                    namespace,
                    fields,
                    storage_fields,
                    storage_keyword_span,
                )
            }
            None => Err(handler.emit_err(CompileError::NoDeclaredStorage {
                span: fields[0].span(),
            })),
        }
    }

    pub fn set_storage_declaration(
        &mut self,
        handler: &Handler,
        decl_ref: DeclRefStorage,
    ) -> Result<(), ErrorEmitted> {
        if self.declared_storage.is_some() {
            return Err(handler.emit_err(CompileError::MultipleStorageDeclarations {
                span: decl_ref.span(),
            }));
        }
        self.declared_storage = Some(decl_ref);
        Ok(())
    }

    pub fn get_all_declared_symbols(&self) -> impl Iterator<Item = &Ident> {
        self.symbols().keys()
    }

    pub(crate) fn insert_symbol(
        &mut self,
        handler: &Handler,
        name: Ident,
        item: ty::TyDecl,
        const_shadowing_mode: ConstShadowingMode,
        generic_shadowing_mode: GenericShadowingMode,
    ) -> Result<(), ErrorEmitted> {
        let append_shadowing_error =
            |ident: &Ident,
             decl: &ty::TyDecl,
             is_use: bool,
             is_alias: bool,
             item: &ty::TyDecl,
             const_shadowing_mode: ConstShadowingMode| {
                use ty::TyDecl::*;
                match (
                    ident,
                    decl,
                    is_use,
                    is_alias,
                    &item,
                    const_shadowing_mode,
                    generic_shadowing_mode,
                ) {
                    // variable shadowing a constant
                    (
                        constant_ident,
                        ConstantDecl(constant_decl),
                        is_imported_constant,
                        is_alias,
                        VariableDecl { .. },
                        _,
                        _,
                    ) => {
                        handler.emit_err(CompileError::ConstantsCannotBeShadowed {
                            variable_or_constant: "Variable".to_string(),
                            name: (&name).into(),
                            constant_span: constant_ident.span(),
                            constant_decl: if is_imported_constant {
                                constant_decl.decl_span.clone()
                            } else {
                                Span::dummy()
                            },
                            is_alias,
                        });
                    }
                    // constant shadowing a constant sequentially
                    (
                        constant_ident,
                        ConstantDecl(constant_decl),
                        is_imported_constant,
                        is_alias,
                        ConstantDecl { .. },
                        ConstShadowingMode::Sequential,
                        _,
                    ) => {
                        handler.emit_err(CompileError::ConstantsCannotBeShadowed {
                            variable_or_constant: "Constant".to_string(),
                            name: (&name).into(),
                            constant_span: constant_ident.span(),
                            constant_decl: if is_imported_constant {
                                constant_decl.decl_span.clone()
                            } else {
                                Span::dummy()
                            },
                            is_alias,
                        });
                    }
                    // constant shadowing a variable
                    (_, VariableDecl(variable_decl), _, _, ConstantDecl { .. }, _, _) => {
                        handler.emit_err(CompileError::ConstantShadowsVariable {
                            name: (&name).into(),
                            variable_span: variable_decl.name.span(),
                        });
                    }
                    // constant shadowing a constant item-style (outside of a function body)
                    (
                        _,
                        ConstantDecl { .. },
                        _,
                        _,
                        ConstantDecl { .. },
                        ConstShadowingMode::ItemStyle,
                        _,
                    ) => {
                        handler.emit_err(CompileError::MultipleDefinitionsOfConstant {
                            name: name.clone(),
                            span: name.span(),
                        });
                    }
                    // type or type alias shadowing another type or type alias
                    // trait/abi shadowing another trait/abi
                    // type or type alias shadowing a trait/abi, or vice versa
                    (
                        _,
                        StructDecl { .. }
                        | EnumDecl { .. }
                        | TypeAliasDecl { .. }
                        | TraitDecl { .. }
                        | AbiDecl { .. },
                        _,
                        _,
                        StructDecl { .. }
                        | EnumDecl { .. }
                        | TypeAliasDecl { .. }
                        | TraitDecl { .. }
                        | AbiDecl { .. },
                        _,
                        _,
                    ) => {
                        handler.emit_err(CompileError::MultipleDefinitionsOfName {
                            name: name.clone(),
                            span: name.span(),
                        });
                    }
                    // generic parameter shadowing another generic parameter
                    (
                        _,
                        GenericTypeForFunctionScope { .. },
                        _,
                        _,
                        GenericTypeForFunctionScope { .. },
                        _,
                        GenericShadowingMode::Disallow,
                    ) => {
                        handler.emit_err(CompileError::GenericShadowsGeneric {
                            name: (&name).into(),
                        });
                    }
                    _ => {}
                }
            };

        if let Some((ident, decl)) = self.symbols.get_key_value(&name) {
            append_shadowing_error(ident, decl, false, false, &item, const_shadowing_mode);
        }

        if let Some((ident, (_, GlobImport::No, decl, _))) = self.use_synonyms.get_key_value(&name)
        {
            append_shadowing_error(
                ident,
                decl,
                true,
                self.use_aliases.get(&name.to_string()).is_some(),
                &item,
                const_shadowing_mode,
            );
        }

        self.symbols.insert(name, item);

        Ok(())
    }

    pub(crate) fn check_symbol(&self, name: &Ident) -> Result<&ty::TyDecl, CompileError> {
        self.symbols
            .get(name)
            .ok_or_else(|| CompileError::SymbolNotFound {
                name: name.clone(),
                span: name.span(),
            })
    }

    pub fn get_items_for_type(&self, engines: &Engines, type_id: TypeId) -> Vec<ty::TyTraitItem> {
        self.implemented_traits.get_items_for_type(engines, type_id)
    }

    pub fn get_impl_spans_for_decl(&self, engines: &Engines, ty_decl: &TyDecl) -> Vec<Span> {
        let handler = Handler::default();
        ty_decl
            .return_type(&handler, engines)
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
                ty::TyTraitItem::Type(_decl_ref) => None,
            })
            .collect::<Vec<_>>()
    }

    pub(crate) fn has_storage_declared(&self) -> bool {
        self.declared_storage.is_some()
    }

    pub fn get_declared_storage(&self, decl_engine: &DeclEngine) -> Option<TyStorageDecl> {
        self.declared_storage
            .as_ref()
            .map(|decl_ref| (*decl_engine.get_storage(decl_ref)).clone())
    }

    pub(crate) fn get_storage_field_descriptors(
        &self,
        handler: &Handler,
        decl_engine: &DeclEngine,
    ) -> Result<Vec<ty::TyStorageField>, ErrorEmitted> {
        match self.get_declared_storage(decl_engine) {
            Some(storage) => Ok(storage.fields.clone()),
            None => {
                let msg = "unknown source location";
                let span = Span::new(Arc::from(msg), 0, msg.len(), None).unwrap();
                Err(handler.emit_err(CompileError::NoDeclaredStorage { span }))
            }
        }
    }

    /// Returns a tuple where the first element is the [TypeId] of the actual expression, and
    /// the second is the [TypeId] of its parent.
    pub(crate) fn find_subfield_type(
        &self,
        handler: &Handler,
        engines: &Engines,
        namespace: &Namespace,
        base_name: &Ident,
        projections: &[ty::ProjectionKind],
    ) -> Result<(TypeId, TypeId), ErrorEmitted> {
        let type_engine = engines.te();
        let decl_engine = engines.de();

        let symbol = match self.symbols.get(base_name).cloned() {
            Some(s) => s,
            None => {
                return Err(handler.emit_err(CompileError::UnknownVariable {
                    var_name: base_name.clone(),
                    span: base_name.span(),
                }));
            }
        };
        let mut symbol = symbol.return_type(handler, engines)?;
        let mut symbol_span = base_name.span();
        let mut parent_rover = symbol;
        let mut full_span_for_error = base_name.span();
        for projection in projections {
            let resolved_type = match type_engine.to_typeinfo(symbol, &symbol_span) {
                Ok(resolved_type) => resolved_type,
                Err(error) => {
                    return Err(handler.emit_err(CompileError::TypeError(error)));
                }
            };
            match (resolved_type, projection) {
                (
                    TypeInfo::Struct(decl_ref),
                    ty::ProjectionKind::StructField { name: field_name },
                ) => {
                    let struct_decl = decl_engine.get_struct(&decl_ref);
                    let (struct_can_be_changed, is_public_struct_access) =
                        StructAccessInfo::get_info(&struct_decl, namespace).into();

                    let field_type_id = match struct_decl.find_field(field_name) {
                        Some(struct_field) => {
                            if is_public_struct_access && struct_field.is_private() {
                                return Err(handler.emit_err(CompileError::StructFieldIsPrivate {
                                    field_name: field_name.into(),
                                    struct_name: struct_decl.call_path.suffix.clone(),
                                    field_decl_span: struct_field.name.span(),
                                    struct_can_be_changed,
                                    usage_context: StructFieldUsageContext::StructFieldAccess,
                                }));
                            }

                            struct_field.type_argument.type_id
                        }
                        None => {
                            return Err(handler.emit_err(CompileError::StructFieldDoesNotExist {
                                field_name: field_name.into(),
                                available_fields: struct_decl
                                    .accessible_fields_names(is_public_struct_access),
                                is_public_struct_access,
                                struct_name: struct_decl.call_path.suffix.clone(),
                                struct_decl_span: struct_decl.span(),
                                struct_is_empty: struct_decl.is_empty(),
                                usage_context: StructFieldUsageContext::StructFieldAccess,
                            }));
                        }
                    };
                    parent_rover = symbol;
                    symbol = field_type_id;
                    symbol_span = field_name.span().clone();
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
                            return Err(handler.emit_err(CompileError::TupleIndexOutOfBounds {
                                index: *index,
                                count: fields.len(),
                                tuple_type: engines.help_out(symbol).to_string(),
                                span: index_span.clone(),
                                prefix_span: full_span_for_error.clone(),
                            }));
                        }
                    };
                    parent_rover = symbol;
                    symbol = *field_type;
                    symbol_span = index_span.clone();
                    full_span_for_error = Span::join(full_span_for_error, index_span.clone());
                }
                (
                    TypeInfo::Array(elem_ty, _),
                    ty::ProjectionKind::ArrayIndex { index_span, .. },
                ) => {
                    parent_rover = symbol;
                    symbol = elem_ty.type_id;
                    symbol_span = index_span.clone();
                    // `index_span` does not contain the enclosing square brackets.
                    // Which means, if this array index access is the last one before the
                    // erroneous expression, the `full_span_for_error` will be missing the
                    // closing `]`. We can live with this small glitch so far. To fix it,
                    // we would need to bring the full span of the index all the way from
                    // the parsing stage. An effort that doesn't pay off at the moment.
                    // TODO: Include the closing square bracket into the error span.
                    full_span_for_error = Span::join(full_span_for_error, index_span.clone());
                }
                (actually, ty::ProjectionKind::StructField { name }) => {
                    return Err(handler.emit_err(CompileError::FieldAccessOnNonStruct {
                        actually: engines.help_out(actually).to_string(),
                        storage_variable: None,
                        field_name: name.into(),
                        span: full_span_for_error,
                    }));
                }
                (
                    actually,
                    ty::ProjectionKind::TupleField {
                        index, index_span, ..
                    },
                ) => {
                    return Err(
                        handler.emit_err(CompileError::TupleElementAccessOnNonTuple {
                            actually: engines.help_out(actually).to_string(),
                            span: full_span_for_error,
                            index: *index,
                            index_span: index_span.clone(),
                        }),
                    );
                }
                (actually, ty::ProjectionKind::ArrayIndex { .. }) => {
                    return Err(handler.emit_err(CompileError::NotIndexable {
                        actually: engines.help_out(actually).to_string(),
                        span: full_span_for_error,
                    }));
                }
            }
        }
        Ok((symbol, parent_rover))
    }
}

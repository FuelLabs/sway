use crate::{error::*, namespace::*, parse_tree::*, semantic_analysis::*, type_engine::*};

use super::TraitMap;

use sway_types::{span::Span, Spanned};

use std::sync::Arc;

type SymbolMap = im::OrdMap<Ident, TypedDeclaration>;
type UseSynonyms = im::HashMap<Ident, Vec<Ident>>;
type UseAliases = im::HashMap<String, Ident>;

/// The set of items that exist within some lexical scope via declaration or importing.
#[derive(Clone, Debug, Default, PartialEq)]
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
    pub(crate) declared_storage: Option<TypedStorageDeclaration>,
}

impl Items {
    /// Immutable access to the inner symbol map.
    pub fn symbols(&self) -> &SymbolMap {
        &self.symbols
    }

    pub fn apply_storage_load(
        &self,
        fields: Vec<Ident>,
        storage_fields: &[TypedStorageField],
    ) -> CompileResult<(TypeCheckedStorageAccess, TypeId)> {
        match self.declared_storage {
            Some(ref storage) => storage.apply_storage_load(fields, storage_fields),
            None => err(
                vec![],
                vec![CompileError::NoDeclaredStorage {
                    span: fields[0].span(),
                }],
            ),
        }
    }

    pub fn set_storage_declaration(&mut self, decl: TypedStorageDeclaration) -> CompileResult<()> {
        if self.declared_storage.is_some() {
            return err(
                vec![],
                vec![CompileError::MultipleStorageDeclarations { span: decl.span() }],
            );
        }
        self.declared_storage = Some(decl);
        ok((), vec![], vec![])
    }

    pub fn get_all_declared_symbols(&self) -> impl Iterator<Item = &Ident> {
        self.symbols().keys()
    }

    pub(crate) fn insert_symbol(
        &mut self,
        name: Ident,
        item: TypedDeclaration,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        // purposefully do not preemptively return errors so that the
        // new definiton allows later usages to compile
        if self.symbols.get(&name).is_some() {
            match item {
                TypedDeclaration::EnumDeclaration { .. }
                | TypedDeclaration::StructDeclaration { .. } => {
                    errors.push(CompileError::ShadowsOtherSymbol { name: name.clone() });
                }
                TypedDeclaration::GenericTypeForFunctionScope { .. } => {
                    errors.push(CompileError::GenericShadowsGeneric { name: name.clone() });
                }
                _ => {
                    warnings.push(CompileWarning {
                        span: name.span(),
                        warning_content: Warning::ShadowsOtherSymbol { name: name.clone() },
                    });
                }
            }
        }
        self.symbols.insert(name, item);
        ok((), warnings, errors)
    }

    pub(crate) fn check_symbol(&self, name: &Ident) -> Result<&TypedDeclaration, CompileError> {
        self.symbols
            .get(name)
            .ok_or_else(|| CompileError::SymbolNotFound { name: name.clone() })
    }

    pub(crate) fn insert_trait_implementation(
        &mut self,
        trait_name: CallPath,
        implementing_for_type_id: TypeId,
        functions_buf: Vec<TypedFunctionDeclaration>,
    ) {
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
            is_absolute: trait_name.is_absolute,
        };
        self.implemented_traits
            .insert(trait_name, implementing_for_type_id, functions_buf);
    }

    pub(crate) fn get_methods_for_type(
        &self,
        implementing_for_type_id: TypeId,
    ) -> Vec<TypedFunctionDeclaration> {
        self.implemented_traits
            .get_methods_for_type(implementing_for_type_id)
    }

    pub(crate) fn get_canonical_path(&self, symbol: &Ident) -> &[Ident] {
        self.use_synonyms.get(symbol).map(|v| &v[..]).unwrap_or(&[])
    }

    pub(crate) fn has_storage_declared(&self) -> bool {
        self.declared_storage.is_some()
    }

    pub(crate) fn get_storage_field_descriptors(&self) -> CompileResult<Vec<TypedStorageField>> {
        if let Some(fields) = self.declared_storage.as_ref().map(|ds| ds.fields.clone()) {
            ok(fields, vec![], vec![])
        } else {
            let msg = "unknown source location";
            let span = Span::new(Arc::from(msg), 0, msg.len(), None).unwrap();
            err(vec![], vec![CompileError::NoDeclaredStorage { span }])
        }
    }

    /// Returns a tuple where the first element is the [ResolvedType] of the actual expression, and
    /// the second is the [ResolvedType] of its parent, for control-flow analysis.
    pub(crate) fn find_subfield_type(
        &self,
        base_name: &Ident,
        projections: &[ProjectionKind],
    ) -> CompileResult<(TypeId, TypeId)> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let symbol = match self.symbols.get(base_name).cloned() {
            Some(s) => s,
            None => {
                errors.push(CompileError::UnknownVariable {
                    var_name: base_name.clone(),
                });
                return err(warnings, errors);
            }
        };
        let mut symbol = check!(
            symbol.return_type(),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut symbol_span = base_name.span();
        let mut parent_rover = symbol;
        let mut full_name_for_error = base_name.to_string();
        let mut full_span_for_error = base_name.span();
        for projection in projections {
            let resolved_type = match resolve_type(symbol, &symbol_span) {
                Ok(resolved_type) => resolved_type,
                Err(error) => {
                    errors.push(CompileError::TypeError(error));
                    return err(warnings, errors);
                }
            };
            match (resolved_type, projection) {
                (
                    TypeInfo::Struct {
                        name: struct_name,
                        fields,
                        ..
                    },
                    ProjectionKind::StructField { name: field_name },
                ) => {
                    let field_type_opt = {
                        fields.iter().find_map(
                            |TypedStructField {
                                 type_id: r#type,
                                 name,
                                 ..
                             }| {
                                if name == field_name {
                                    Some(r#type)
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
                            let available_fields = fields
                                .iter()
                                .map(|field| field.name.as_str())
                                .collect::<Vec<_>>();

                            errors.push(CompileError::FieldNotFound {
                                field_name: field_name.clone(),
                                struct_name,
                                available_fields: available_fields.join(", "),
                            });
                            return err(warnings, errors);
                        }
                    };
                    parent_rover = symbol;
                    symbol = *field_type;
                    symbol_span = field_name.span().clone();
                    full_name_for_error.push_str(field_name.as_str());
                    full_span_for_error =
                        Span::join(full_span_for_error, field_name.span().clone());
                }
                (TypeInfo::Tuple(fields), ProjectionKind::TupleField { index, index_span }) => {
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
                (actually, ProjectionKind::StructField { .. }) => {
                    errors.push(CompileError::FieldAccessOnNonStruct {
                        span: full_span_for_error,
                        actually: actually.to_string(),
                    });
                    return err(warnings, errors);
                }
                (actually, ProjectionKind::TupleField { .. }) => {
                    errors.push(CompileError::NotATuple {
                        name: full_name_for_error,
                        span: full_span_for_error,
                        actually: actually.to_string(),
                    });
                    return err(warnings, errors);
                }
            }
        }
        ok((symbol, parent_rover), warnings, errors)
    }
}

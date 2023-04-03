use std::{
    fmt,
    hash::{Hash, Hasher},
};

use sway_error::error::CompileError;
use sway_types::{Ident, Span, Spanned};

use crate::{
    decl_engine::*,
    engine_threading::*,
    error::*,
    language::{ty::*, Visibility},
    type_system::*,
    types::*,
};

#[derive(Clone, Debug)]
pub enum TyDecl {
    VariableDecl(Box<TyVariableDecl>),
    ConstantDecl {
        name: Ident,
        decl_id: DeclId<TyConstantDecl>,
        decl_span: Span,
    },
    FunctionDecl {
        name: Ident,
        decl_id: DeclId<TyFunctionDecl>,
        subst_list: Template<SubstList>,
        decl_span: Span,
    },
    TraitDecl {
        name: Ident,
        decl_id: DeclId<TyTraitDecl>,
        subst_list: Template<SubstList>,
        decl_span: Span,
    },
    StructDecl {
        name: Ident,
        decl_id: DeclId<TyStructDecl>,
        subst_list: Template<SubstList>,
        decl_span: Span,
    },
    EnumDecl {
        name: Ident,
        decl_id: DeclId<TyEnumDecl>,
        subst_list: Template<SubstList>,
        decl_span: Span,
    },
    EnumVariantDecl {
        decl_id: DeclId<TyEnumDecl>,
        subst_list: Template<SubstList>,
        variant_name: Ident,
        variant_decl_span: Span,
    },
    ImplTrait {
        name: Ident,
        decl_id: DeclId<TyImplTrait>,
        subst_list: Template<SubstList>,
        decl_span: Span,
    },
    AbiDecl {
        name: Ident,
        decl_id: DeclId<TyAbiDecl>,
        decl_span: Span,
    },
    // If type parameters are defined for a function, they are put in the namespace just for
    // the body of that function.
    GenericTypeForFunctionScope {
        name: Ident,
        type_id: TypeId,
    },
    ErrorRecovery(Span),
    StorageDecl {
        decl_id: DeclId<TyStorageDecl>,
        decl_span: Span,
    },
    TypeAliasDecl {
        name: Ident,
        decl_id: DeclId<TyTypeAliasDecl>,
        decl_span: Span,
    },
}

impl EqWithEngines for TyDecl {}
impl PartialEqWithEngines for TyDecl {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let type_engine = engines.te();
        match (self, other) {
            (Self::VariableDecl(x), Self::VariableDecl(y)) => x.eq(y, engines),
            (
                Self::ConstantDecl {
                    name: ln,
                    decl_id: lid,
                    ..
                },
                Self::ConstantDecl {
                    name: rn,
                    decl_id: rid,
                    ..
                },
            ) => ln == rn && lid == rid,

            (
                Self::FunctionDecl {
                    name: ln,
                    decl_id: lid,
                    subst_list: lsl,
                    ..
                },
                Self::FunctionDecl {
                    name: rn,
                    decl_id: rid,
                    subst_list: rsl,
                    ..
                },
            ) => ln == rn && lid == rid && lsl.inner().eq(rsl.inner(), engines),

            (
                Self::TraitDecl {
                    name: ln,
                    decl_id: lid,
                    subst_list: lsl,
                    ..
                },
                Self::TraitDecl {
                    name: rn,
                    decl_id: rid,
                    subst_list: rsl,
                    ..
                },
            ) => ln == rn && lid == rid && lsl.inner().eq(rsl.inner(), engines),
            (
                Self::StructDecl {
                    name: ln,
                    decl_id: lid,
                    subst_list: lsl,
                    ..
                },
                Self::StructDecl {
                    name: rn,
                    decl_id: rid,
                    subst_list: rsl,
                    ..
                },
            ) => ln == rn && lid == rid && lsl.inner().eq(rsl.inner(), engines),
            (
                Self::EnumDecl {
                    name: ln,
                    decl_id: lid,
                    subst_list: lsl,
                    ..
                },
                Self::EnumDecl {
                    name: rn,
                    decl_id: rid,
                    subst_list: rsl,
                    ..
                },
            ) => ln == rn && lid == rid && lsl.inner().eq(rsl.inner(), engines),
            (
                Self::ImplTrait {
                    name: ln,
                    decl_id: lid,
                    subst_list: lsl,
                    ..
                },
                Self::ImplTrait {
                    name: rn,
                    decl_id: rid,
                    subst_list: rsl,
                    ..
                },
            ) => ln == rn && lid == rid && lsl.inner().eq(rsl.inner(), engines),

            (
                Self::AbiDecl {
                    name: ln,
                    decl_id: lid,
                    ..
                },
                Self::AbiDecl {
                    name: rn,
                    decl_id: rid,
                    ..
                },
            ) => ln == rn && lid == rid,
            (Self::StorageDecl { decl_id: lid, .. }, Self::StorageDecl { decl_id: rid, .. }) => {
                lid == rid
            }
            (
                Self::TypeAliasDecl { decl_id: lid, .. },
                Self::TypeAliasDecl { decl_id: rid, .. },
            ) => lid == rid,
            (
                Self::GenericTypeForFunctionScope {
                    name: xn,
                    type_id: xti,
                },
                Self::GenericTypeForFunctionScope {
                    name: yn,
                    type_id: yti,
                },
            ) => xn == yn && type_engine.get(*xti).eq(&type_engine.get(*yti), engines),
            (Self::ErrorRecovery(x), Self::ErrorRecovery(y)) => x == y,
            _ => false,
        }
    }
}

impl HashWithEngines for TyDecl {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        use TyDecl::*;
        let type_engine = engines.te();
        std::mem::discriminant(self).hash(state);
        match self {
            VariableDecl(decl) => {
                decl.hash(state, engines);
            }
            ConstantDecl { decl_id, .. } => {
                decl_id.hash(state);
            }
            FunctionDecl {
                decl_id,
                subst_list,
                ..
            } => {
                decl_id.hash(state);
                subst_list.inner().hash(state, engines);
            }
            TraitDecl {
                decl_id,
                subst_list,
                ..
            } => {
                decl_id.hash(state);
                subst_list.inner().hash(state, engines);
            }
            StructDecl {
                decl_id,
                subst_list,
                ..
            } => {
                decl_id.hash(state);
                subst_list.inner().hash(state, engines);
            }
            EnumDecl {
                decl_id,
                subst_list,
                ..
            } => {
                decl_id.hash(state);
                subst_list.inner().hash(state, engines);
            }
            EnumVariantDecl {
                decl_id,
                variant_name,
                subst_list,
                ..
            } => {
                decl_id.hash(state);
                subst_list.inner().hash(state, engines);
                variant_name.hash(state);
            }
            ImplTrait {
                decl_id,
                subst_list,
                ..
            } => {
                decl_id.hash(state);
                subst_list.inner().hash(state, engines);
            }
            AbiDecl { decl_id, .. } => {
                decl_id.hash(state);
            }
            TypeAliasDecl { decl_id, .. } => {
                decl_id.hash(state);
            }
            StorageDecl { decl_id, .. } => {
                decl_id.hash(state);
            }
            GenericTypeForFunctionScope { name, type_id } => {
                name.hash(state);
                type_engine.get(*type_id).hash(state, engines);
            }
            ErrorRecovery(_) => {}
        }
    }
}

impl Spanned for TyDecl {
    fn span(&self) -> Span {
        use TyDecl::*;
        match self {
            VariableDecl(decl) => decl.name.span(),
            FunctionDecl { decl_span, .. }
            | TraitDecl { decl_span, .. }
            | ImplTrait { decl_span, .. }
            | ConstantDecl { decl_span, .. }
            | StorageDecl { decl_span, .. }
            | TypeAliasDecl { decl_span, .. }
            | AbiDecl { decl_span, .. }
            | StructDecl { decl_span, .. }
            | EnumDecl { decl_span, .. } => decl_span.clone(),
            EnumVariantDecl {
                variant_decl_span, ..
            } => variant_decl_span.clone(),
            GenericTypeForFunctionScope { name, .. } => name.span(),
            ErrorRecovery(span) => span.clone(),
        }
    }
}

impl DisplayWithEngines for TyDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: Engines<'_>) -> std::fmt::Result {
        let type_engine = engines.te();
        write!(
            f,
            "{} declaration ({})",
            self.friendly_type_name(),
            match self {
                TyDecl::VariableDecl(decl) => {
                    let TyVariableDecl {
                        mutability,
                        name,
                        type_ascription,
                        body,
                        ..
                    } = &**decl;
                    let mut builder = String::new();
                    match mutability {
                        VariableMutability::Mutable => builder.push_str("mut"),
                        VariableMutability::RefMutable => builder.push_str("ref mut"),
                        VariableMutability::Immutable => {}
                    }
                    builder.push_str(name.as_str());
                    builder.push_str(": ");
                    builder.push_str(
                        &engines
                            .help_out(type_engine.get(type_ascription.type_id))
                            .to_string(),
                    );
                    builder.push_str(" = ");
                    builder.push_str(&engines.help_out(body).to_string());
                    builder
                }
                TyDecl::FunctionDecl { name, .. }
                | TyDecl::TraitDecl { name, .. }
                | TyDecl::StructDecl { name, .. }
                | TyDecl::EnumDecl { name, .. } => name.as_str().into(),
                _ => String::new(),
            }
        )
    }
}

impl DebugWithEngines for TyDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: Engines<'_>) -> std::fmt::Result {
        let type_engine = engines.te();
        write!(
            f,
            "{} declaration ({})",
            self.friendly_type_name(),
            match self {
                TyDecl::VariableDecl(decl) => {
                    let TyVariableDecl {
                        mutability,
                        name,
                        type_ascription,
                        body,
                        ..
                    } = &**decl;
                    let mut builder = String::new();
                    match mutability {
                        VariableMutability::Mutable => builder.push_str("mut"),
                        VariableMutability::RefMutable => builder.push_str("ref mut"),
                        VariableMutability::Immutable => {}
                    }
                    builder.push_str(name.as_str());
                    builder.push_str(": ");
                    builder.push_str(
                        format!(
                            "{:?}",
                            engines.help_out(type_engine.get(type_ascription.type_id))
                        )
                        .as_str(),
                    );
                    builder.push_str(" = ");
                    builder.push_str(format!("{:?}", engines.help_out(body)).as_str());
                    builder
                }
                TyDecl::FunctionDecl { name, .. }
                | TyDecl::TraitDecl { name, .. }
                | TyDecl::StructDecl { name, .. }
                | TyDecl::EnumDecl { name, .. } => name.as_str().into(),
                _ => String::new(),
            }
        )
    }
}

impl CollectTypesMetadata for TyDecl {
    // this is only run on entry nodes, which must have all well-formed types
    fn collect_types_metadata(
        &self,
        ctx: &mut CollectTypesMetadataContext,
    ) -> CompileResult<Vec<TypeMetadata>> {
        use TyDecl::*;
        let mut warnings = vec![];
        let mut errors = vec![];
        let decl_engine = ctx.decl_engine;
        let metadata = match self {
            VariableDecl(decl) => {
                let mut body = check!(
                    decl.body.collect_types_metadata(ctx),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                body.append(&mut check!(
                    decl.type_ascription.type_id.collect_types_metadata(ctx),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
                body
            }
            FunctionDecl { decl_id, .. } => {
                let decl = decl_engine.get_function(decl_id);
                check!(
                    decl.collect_types_metadata(ctx),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            ConstantDecl { decl_id, .. } => {
                let TyConstantDecl { value, .. } = decl_engine.get_constant(decl_id);
                if let Some(value) = value {
                    check!(
                        value.collect_types_metadata(ctx),
                        return err(warnings, errors),
                        warnings,
                        errors
                    )
                } else {
                    return ok(vec![], warnings, errors);
                }
            }
            ErrorRecovery(_)
            | StorageDecl { .. }
            | TraitDecl { .. }
            | StructDecl { .. }
            | EnumDecl { .. }
            | EnumVariantDecl { .. }
            | ImplTrait { .. }
            | AbiDecl { .. }
            | TypeAliasDecl { .. }
            | GenericTypeForFunctionScope { .. } => vec![],
        };
        if errors.is_empty() {
            ok(metadata, warnings, errors)
        } else {
            err(warnings, errors)
        }
    }
}

impl GetDeclIdent for TyDecl {
    fn get_decl_ident(&self) -> Option<Ident> {
        match self {
            TyDecl::VariableDecl(decl) => Some(decl.name.clone()),
            TyDecl::FunctionDecl { name, .. }
            | TyDecl::TraitDecl { name, .. }
            | TyDecl::ConstantDecl { name, .. }
            | TyDecl::ImplTrait { name, .. }
            | TyDecl::AbiDecl { name, .. }
            | TyDecl::TypeAliasDecl { name, .. }
            | TyDecl::GenericTypeForFunctionScope { name, .. }
            | TyDecl::StructDecl { name, .. }
            | TyDecl::EnumDecl { name, .. } => Some(name.clone()),
            TyDecl::EnumVariantDecl { variant_name, .. } => Some(variant_name.clone()),
            TyDecl::ErrorRecovery(_) => None,
            TyDecl::StorageDecl { .. } => None,
        }
    }
}

impl TyDecl {
    pub fn get_fun_decl_ref(&self) -> Option<DeclRefFunction> {
        if let TyDecl::FunctionDecl {
            name,
            decl_id,
            subst_list,
            decl_span,
        } = self
        {
            Some(DeclRef::new(
                name.clone(),
                *decl_id,
                subst_list.unscoped_copy(),
                decl_span.clone(),
            ))
        } else {
            None
        }
    }

    /// Retrieves the declaration as a `DeclRef<DeclId<TyEnumDecl>>`.
    ///
    /// Returns an error if `self` is not the [TyDecl][EnumDecl] variant.
    pub(crate) fn to_enum_ref(&self, engines: Engines) -> CompileResult<DeclRefEnum> {
        match self {
            TyDecl::EnumDecl {
                name,
                decl_id,
                subst_list,
                decl_span,
            } => ok(
                DeclRef::new(
                    name.clone(),
                    *decl_id,
                    subst_list.unscoped_copy(),
                    decl_span.clone(),
                ),
                vec![],
                vec![],
            ),
            TyDecl::TypeAliasDecl { decl_id, .. } => {
                let TyTypeAliasDecl { ty, span, .. } = engines.de().get_type_alias(decl_id);
                engines.te().get(ty.type_id).expect_enum(engines, "", &span)
            }
            TyDecl::ErrorRecovery(_) => err(vec![], vec![]),
            decl => err(
                vec![],
                vec![CompileError::DeclIsNotAnEnum {
                    actually: decl.friendly_type_name().to_string(),
                    span: decl.span(),
                }],
            ),
        }
    }

    /// Retrieves the declaration as a `DeclRef<DeclId<TyStructDecl>>`.
    ///
    /// Returns an error if `self` is not the [TyDecl][StructDecl] variant.
    pub(crate) fn to_struct_ref(&self, engines: Engines) -> CompileResult<DeclRefStruct> {
        match self {
            TyDecl::StructDecl {
                name,
                decl_id,
                subst_list,
                decl_span,
            } => ok(
                DeclRef::new(
                    name.clone(),
                    *decl_id,
                    subst_list.unscoped_copy(),
                    decl_span.clone(),
                ),
                vec![],
                vec![],
            ),
            TyDecl::TypeAliasDecl { decl_id, .. } => {
                let TyTypeAliasDecl { ty, span, .. } = engines.de().get_type_alias(decl_id);
                engines.te().get(ty.type_id).expect_struct(engines, &span)
            }
            TyDecl::ErrorRecovery(_) => err(vec![], vec![]),
            decl => err(
                vec![],
                vec![CompileError::DeclIsNotAStruct {
                    actually: decl.friendly_type_name().to_string(),
                    span: decl.span(),
                }],
            ),
        }
    }

    /// Retrieves the declaration as a `DeclRef<DeclId<TyFunctionDecl>>`.
    ///
    /// Returns an error if `self` is not the [TyDecl][FunctionDecl] variant.
    pub(crate) fn to_fn_ref(&self) -> CompileResult<DeclRef<DeclId<TyFunctionDecl>>> {
        match self {
            TyDecl::FunctionDecl {
                name,
                decl_id,
                subst_list,
                decl_span,
            } => ok(
                DeclRef::new(
                    name.clone(),
                    *decl_id,
                    subst_list.unscoped_copy(),
                    decl_span.clone(),
                ),
                vec![],
                vec![],
            ),
            TyDecl::ErrorRecovery(_) => err(vec![], vec![]),
            decl => err(
                vec![],
                vec![CompileError::DeclIsNotAFunction {
                    actually: decl.friendly_type_name().to_string(),
                    span: decl.span(),
                }],
            ),
        }
    }

    /// Retrieves the declaration as a variable declaration.
    ///
    /// Returns an error if `self` is not a [TyVariableDecl].
    pub(crate) fn expect_variable(&self) -> CompileResult<&TyVariableDecl> {
        let warnings = vec![];
        let mut errors = vec![];
        match self {
            TyDecl::VariableDecl(decl) => ok(decl, warnings, errors),
            TyDecl::ErrorRecovery(_) => err(vec![], vec![]),
            decl => {
                errors.push(CompileError::DeclIsNotAVariable {
                    actually: decl.friendly_type_name().to_string(),
                    span: decl.span(),
                });
                err(warnings, errors)
            }
        }
    }

    /// Retrieves the declaration as a `DeclRef<DeclId<TyAbiDecl>>`.
    ///
    /// Returns an error if `self` is not the [TyDecl][AbiDecl] variant.
    pub(crate) fn to_abi_ref(&self) -> CompileResult<DeclRef<DeclId<TyAbiDecl>>> {
        match self {
            TyDecl::AbiDecl {
                name,
                decl_id,
                decl_span,
            } => ok(
                DeclRef::new(name.clone(), *decl_id, SubstList::new(), decl_span.clone()),
                vec![],
                vec![],
            ),
            TyDecl::ErrorRecovery(_) => err(vec![], vec![]),
            decl => err(
                vec![],
                vec![CompileError::DeclIsNotAnAbi {
                    actually: decl.friendly_type_name().to_string(),
                    span: decl.span(),
                }],
            ),
        }
    }

    /// Retrieves the declaration as a `DeclRef<DeclId<TyConstantDecl>>`.
    ///
    /// Returns an error if `self` is not the [TyDecl][ConstantDecl] variant.
    pub(crate) fn to_const_ref(&self) -> CompileResult<DeclRef<DeclId<TyConstantDecl>>> {
        match self {
            TyDecl::ConstantDecl {
                name,
                decl_id,
                decl_span,
            } => ok(
                DeclRef::new(name.clone(), *decl_id, SubstList::new(), decl_span.clone()),
                vec![],
                vec![],
            ),
            TyDecl::ErrorRecovery(_) => err(vec![], vec![]),
            decl => err(
                vec![],
                vec![CompileError::DeclIsNotAConstant {
                    actually: decl.friendly_type_name().to_string(),
                    span: decl.span(),
                }],
            ),
        }
    }

    /// friendly name string used for error reporting,
    /// which consists of the the identifier for the declaration.
    pub fn friendly_name(&self, engines: &Engines) -> String {
        use TyDecl::*;
        let decl_engine = engines.de();
        let type_engine = engines.te();
        match self {
            ImplTrait { decl_id, .. } => {
                let decl = decl_engine.get_impl_trait(decl_id);
                let implementing_for_type_id = type_engine.get(decl.implementing_for.type_id);
                format!(
                    "{} for {:?}",
                    self.get_decl_ident()
                        .map_or(String::from(""), |f| f.as_str().to_string()),
                    engines.help_out(implementing_for_type_id)
                )
            }
            _ => self
                .get_decl_ident()
                .map_or(String::from(""), |f| f.as_str().to_string()),
        }
    }

    /// friendly type name string used for error reporting,
    /// which consists of the type name of the declaration AST node.
    pub fn friendly_type_name(&self) -> &'static str {
        use TyDecl::*;
        match self {
            VariableDecl(_) => "variable",
            ConstantDecl { .. } => "constant",
            FunctionDecl { .. } => "function",
            TraitDecl { .. } => "trait",
            StructDecl { .. } => "struct",
            EnumDecl { .. } => "enum",
            EnumVariantDecl { .. } => "enum variant",
            ImplTrait { .. } => "impl trait",
            AbiDecl { .. } => "abi",
            GenericTypeForFunctionScope { .. } => "generic type parameter",
            ErrorRecovery(_) => "error",
            StorageDecl { .. } => "contract storage declaration",
            TypeAliasDecl { .. } => "type alias declaration",
        }
    }

    /// name string used in `forc doc` file path generation that mirrors `cargo doc`.
    pub fn doc_name(&self) -> &'static str {
        use TyDecl::*;
        match self {
            StructDecl { .. } => "struct",
            EnumDecl { .. } => "enum",
            TraitDecl { .. } => "trait",
            AbiDecl { .. } => "abi",
            StorageDecl { .. } => "contract_storage",
            ImplTrait { .. } => "impl_trait",
            FunctionDecl { .. } => "fn",
            ConstantDecl { .. } => "constant",
            TypeAliasDecl { .. } => "type alias",
            _ => unreachable!("these items are non-documentable"),
        }
    }

    pub(crate) fn return_type(&self, engines: Engines<'_>) -> CompileResult<TypeId> {
        let warnings = vec![];
        let mut errors = vec![];
        let type_engine = engines.te();
        let decl_engine = engines.de();
        let type_id = match self {
            TyDecl::VariableDecl(decl) => decl.body.return_type,
            TyDecl::FunctionDecl { decl_id, .. } => {
                let decl = decl_engine.get_function(decl_id);
                decl.return_type.type_id
            }
            TyDecl::StructDecl {
                name,
                decl_id,
                subst_list,
                decl_span,
            } => type_engine.insert(
                decl_engine,
                TypeInfo::Struct(DeclRef::new(
                    name.clone(),
                    *decl_id,
                    subst_list.unscoped_copy(),
                    decl_span.clone(),
                )),
            ),
            TyDecl::EnumDecl {
                name,
                decl_id,
                subst_list,
                decl_span,
            } => type_engine.insert(
                decl_engine,
                TypeInfo::Enum(DeclRef::new(
                    name.clone(),
                    *decl_id,
                    subst_list.unscoped_copy(),
                    decl_span.clone(),
                )),
            ),
            TyDecl::StorageDecl { decl_id, .. } => {
                let storage_decl = decl_engine.get_storage(decl_id);
                type_engine.insert(
                    decl_engine,
                    TypeInfo::Storage {
                        fields: storage_decl.fields_as_typed_struct_fields(),
                    },
                )
            }
            TyDecl::TypeAliasDecl { decl_id, .. } => {
                let decl = decl_engine.get_type_alias(decl_id);
                decl.create_type_id(engines)
            }
            TyDecl::GenericTypeForFunctionScope { type_id, .. } => *type_id,
            decl => {
                errors.push(CompileError::NotAType {
                    span: decl.span(),
                    name: engines.help_out(decl).to_string(),
                    actually_is: decl.friendly_type_name(),
                });
                return err(warnings, errors);
            }
        };
        ok(type_id, warnings, errors)
    }

    pub(crate) fn visibility(&self, decl_engine: &DeclEngine) -> Visibility {
        use TyDecl::*;
        match self {
            TraitDecl { decl_id, .. } => {
                let TyTraitDecl { visibility, .. } = decl_engine.get_trait(decl_id);
                visibility
            }
            ConstantDecl { decl_id, .. } => {
                let TyConstantDecl { visibility, .. } = decl_engine.get_constant(decl_id);
                visibility
            }
            StructDecl { decl_id, .. } => {
                let TyStructDecl { visibility, .. } = decl_engine.get_struct(decl_id);
                visibility
            }
            EnumDecl { decl_id, .. } => {
                let TyEnumDecl { visibility, .. } = decl_engine.get_enum(decl_id);
                visibility
            }
            EnumVariantDecl { decl_id, .. } => {
                let TyEnumDecl { visibility, .. } = decl_engine.get_enum(decl_id);
                visibility
            }
            FunctionDecl { decl_id, .. } => {
                let TyFunctionDecl { visibility, .. } = decl_engine.get_function(decl_id);
                visibility
            }
            TypeAliasDecl { decl_id, .. } => {
                let TyTypeAliasDecl { visibility, .. } = decl_engine.get_type_alias(decl_id);
                visibility
            }
            GenericTypeForFunctionScope { .. }
            | ImplTrait { .. }
            | StorageDecl { .. }
            | AbiDecl { .. }
            | ErrorRecovery(_) => Visibility::Public,
            VariableDecl(decl) => decl.mutability.visibility(),
        }
    }
}

impl From<DeclRef<DeclId<TyConstantDecl>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyConstantDecl>>) -> Self {
        TyDecl::ConstantDecl {
            name: decl_ref.name().clone(),
            decl_id: *decl_ref.id(),
            decl_span: decl_ref.decl_span().clone(),
        }
    }
}

impl From<DeclRef<DeclId<TyEnumDecl>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyEnumDecl>>) -> Self {
        TyDecl::EnumDecl {
            name: decl_ref.name().clone(),
            decl_id: *decl_ref.id(),
            subst_list: Template::new(decl_ref.subst_list().clone()),
            decl_span: decl_ref.decl_span().clone(),
        }
    }
}

impl From<DeclRef<DeclId<TyFunctionDecl>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyFunctionDecl>>) -> Self {
        TyDecl::FunctionDecl {
            name: decl_ref.name().clone(),
            decl_id: *decl_ref.id(),
            subst_list: Template::new(decl_ref.subst_list().clone()),
            decl_span: decl_ref.decl_span().clone(),
        }
    }
}

impl From<DeclRef<DeclId<TyTraitDecl>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyTraitDecl>>) -> Self {
        TyDecl::TraitDecl {
            name: decl_ref.name().clone(),
            decl_id: *decl_ref.id(),
            subst_list: Template::new(decl_ref.subst_list().clone()),
            decl_span: decl_ref.decl_span().clone(),
        }
    }
}

impl From<DeclRef<DeclId<TyImplTrait>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyImplTrait>>) -> Self {
        TyDecl::ImplTrait {
            name: decl_ref.name().clone(),
            decl_id: *decl_ref.id(),
            subst_list: Template::new(decl_ref.subst_list().clone()),
            decl_span: decl_ref.decl_span().clone(),
        }
    }
}

impl From<DeclRef<DeclId<TyStructDecl>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyStructDecl>>) -> Self {
        TyDecl::StructDecl {
            name: decl_ref.name().clone(),
            decl_id: *decl_ref.id(),
            subst_list: Template::new(decl_ref.subst_list().clone()),
            decl_span: decl_ref.decl_span().clone(),
        }
    }
}

impl From<DeclRef<DeclId<TyAbiDecl>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyAbiDecl>>) -> Self {
        TyDecl::AbiDecl {
            name: decl_ref.name().clone(),
            decl_id: *decl_ref.id(),
            decl_span: decl_ref.decl_span().clone(),
        }
    }
}

impl From<DeclRef<DeclId<TyStorageDecl>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyStorageDecl>>) -> Self {
        TyDecl::StorageDecl {
            decl_id: *decl_ref.id(),
            decl_span: decl_ref.decl_span().clone(),
        }
    }
}
impl From<DeclRef<DeclId<TyTypeAliasDecl>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyTypeAliasDecl>>) -> Self {
        TyDecl::TypeAliasDecl {
            name: decl_ref.name().clone(),
            decl_id: *decl_ref.id(),
            decl_span: decl_ref.decl_span().clone(),
        }
    }
}

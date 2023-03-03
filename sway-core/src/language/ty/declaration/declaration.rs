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
};

#[derive(Clone, Debug)]
pub enum TyDeclaration {
    VariableDeclaration(Box<TyVariableDeclaration>),
    ConstantDeclaration {
        name: Ident,
        decl_id: DeclId<TyConstantDeclaration>,
        decl_span: Span,
    },
    FunctionDeclaration {
        name: Ident,
        decl_id: DeclId<TyFunctionDeclaration>,
        decl_span: Span,
    },
    TraitDeclaration {
        name: Ident,
        decl_id: DeclId<TyTraitDeclaration>,
        decl_span: Span,
    },
    StructDeclaration {
        name: Ident,
        decl_id: DeclId<TyStructDeclaration>,
        decl_span: Span,
    },
    EnumDeclaration {
        name: Ident,
        decl_id: DeclId<TyEnumDeclaration>,
        decl_span: Span,
    },
    ImplTrait {
        name: Ident,
        decl_id: DeclId<TyImplTrait>,
        decl_span: Span,
    },
    AbiDeclaration {
        name: Ident,
        decl_id: DeclId<TyAbiDeclaration>,
        decl_span: Span,
    },
    // If type parameters are defined for a function, they are put in the namespace just for
    // the body of that function.
    GenericTypeForFunctionScope {
        name: Ident,
        type_id: TypeId,
    },
    ErrorRecovery(Span),
    StorageDeclaration {
        decl_id: DeclId<TyStorageDeclaration>,
        decl_span: Span,
    },
}

impl EqWithEngines for TyDeclaration {}
impl PartialEqWithEngines for TyDeclaration {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let decl_engine = engines.de();
        let type_engine = engines.te();
        match (self, other) {
            (Self::VariableDeclaration(x), Self::VariableDeclaration(y)) => x.eq(y, engines),
            (
                Self::ConstantDeclaration {
                    name: ln,
                    decl_id: lid,
                    ..
                },
                Self::ConstantDeclaration {
                    name: rn,
                    decl_id: rid,
                    ..
                },
            ) => ln == rn && decl_engine.get(*lid).eq(&decl_engine.get(*rid), engines),

            (
                Self::FunctionDeclaration {
                    name: ln,
                    decl_id: lid,
                    ..
                },
                Self::FunctionDeclaration {
                    name: rn,
                    decl_id: rid,
                    ..
                },
            ) => ln == rn && decl_engine.get(*lid).eq(&decl_engine.get(*rid), engines),

            (
                Self::TraitDeclaration {
                    name: ln,
                    decl_id: lid,
                    ..
                },
                Self::TraitDeclaration {
                    name: rn,
                    decl_id: rid,
                    ..
                },
            ) => ln == rn && decl_engine.get(*lid).eq(&decl_engine.get(*rid), engines),

            (
                Self::StructDeclaration {
                    name: ln,
                    decl_id: lid,
                    ..
                },
                Self::StructDeclaration {
                    name: rn,
                    decl_id: rid,
                    ..
                },
            ) => ln == rn && decl_engine.get(*lid).eq(&decl_engine.get(*rid), engines),

            (
                Self::EnumDeclaration {
                    name: ln,
                    decl_id: lid,
                    ..
                },
                Self::EnumDeclaration {
                    name: rn,
                    decl_id: rid,
                    ..
                },
            ) => ln == rn && decl_engine.get(*lid).eq(&decl_engine.get(*rid), engines),

            (
                Self::ImplTrait {
                    name: ln,
                    decl_id: lid,
                    ..
                },
                Self::ImplTrait {
                    name: rn,
                    decl_id: rid,
                    ..
                },
            ) => ln == rn && decl_engine.get(*lid).eq(&decl_engine.get(*rid), engines),

            (
                Self::AbiDeclaration {
                    name: ln,
                    decl_id: lid,
                    ..
                },
                Self::AbiDeclaration {
                    name: rn,
                    decl_id: rid,
                    ..
                },
            ) => ln == rn && decl_engine.get(*lid).eq(&decl_engine.get(*rid), engines),
            (
                Self::StorageDeclaration { decl_id: lid, .. },
                Self::StorageDeclaration { decl_id: rid, .. },
            ) => decl_engine.get(*lid).eq(&decl_engine.get(*rid), engines),
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

impl HashWithEngines for TyDeclaration {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        use TyDeclaration::*;
        let decl_engine = engines.de();
        let type_engine = engines.te();
        std::mem::discriminant(self).hash(state);
        match self {
            VariableDeclaration(decl) => {
                decl.hash(state, engines);
            }
            ConstantDeclaration { decl_id, .. } => {
                decl_engine.get(*decl_id).hash(state, engines);
            }
            FunctionDeclaration { decl_id, .. } => {
                decl_engine.get(*decl_id).hash(state, engines);
            }
            TraitDeclaration { decl_id, .. } => {
                decl_engine.get(*decl_id).hash(state, engines);
            }
            StructDeclaration { decl_id, .. } => {
                decl_engine.get(*decl_id).hash(state, engines);
            }
            EnumDeclaration { decl_id, .. } => {
                decl_engine.get(*decl_id).hash(state, engines);
            }
            ImplTrait { decl_id, .. } => {
                decl_engine.get(*decl_id).hash(state, engines);
            }
            AbiDeclaration { decl_id, .. } => {
                decl_engine.get(*decl_id).hash(state, engines);
            }
            StorageDeclaration { decl_id, .. } => {
                decl_engine.get(*decl_id).hash(state, engines);
            }
            GenericTypeForFunctionScope { name, type_id } => {
                name.hash(state);
                type_engine.get(*type_id).hash(state, engines);
            }
            ErrorRecovery(_) => {}
        }
    }
}

impl SubstTypes for TyDeclaration {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        use TyDeclaration::*;
        match self {
            VariableDeclaration(ref mut var_decl) => var_decl.subst(type_mapping, engines),
            FunctionDeclaration {
                ref mut decl_id, ..
            } => decl_id.subst(type_mapping, engines),
            TraitDeclaration {
                ref mut decl_id, ..
            } => decl_id.subst(type_mapping, engines),
            StructDeclaration {
                ref mut decl_id, ..
            } => decl_id.subst(type_mapping, engines),
            EnumDeclaration {
                ref mut decl_id, ..
            } => decl_id.subst(type_mapping, engines),
            ImplTrait {
                ref mut decl_id, ..
            } => decl_id.subst(type_mapping, engines),
            // generics in an ABI is unsupported by design
            AbiDeclaration { .. }
            | ConstantDeclaration { .. }
            | StorageDeclaration { .. }
            | GenericTypeForFunctionScope { .. }
            | ErrorRecovery(_) => (),
        }
    }
}

impl ReplaceSelfType for TyDeclaration {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        use TyDeclaration::*;
        match self {
            VariableDeclaration(ref mut var_decl) => var_decl.replace_self_type(engines, self_type),
            FunctionDeclaration {
                ref mut decl_id, ..
            } => decl_id.replace_self_type(engines, self_type),
            TraitDeclaration {
                ref mut decl_id, ..
            } => decl_id.replace_self_type(engines, self_type),
            StructDeclaration {
                ref mut decl_id, ..
            } => decl_id.replace_self_type(engines, self_type),
            EnumDeclaration {
                ref mut decl_id, ..
            } => decl_id.replace_self_type(engines, self_type),
            ImplTrait {
                ref mut decl_id, ..
            } => decl_id.replace_self_type(engines, self_type),
            // generics in an ABI is unsupported by design
            AbiDeclaration { .. }
            | ConstantDeclaration { .. }
            | StorageDeclaration { .. }
            | GenericTypeForFunctionScope { .. }
            | ErrorRecovery(_) => (),
        }
    }
}

impl TyDeclaration {
    pub fn get_fun_decl_ref(&self) -> Option<DeclRefFunction> {
        if let TyDeclaration::FunctionDeclaration {
            name,
            decl_id,
            decl_span,
        } = self
        {
            Some(DeclRef {
                name: name.clone(),
                id: *decl_id,
                decl_span: decl_span.clone(),
            })
        } else {
            None
        }
    }
}

impl Spanned for TyDeclaration {
    fn span(&self) -> Span {
        use TyDeclaration::*;
        match self {
            VariableDeclaration(decl) => decl.name.span(),
            FunctionDeclaration { decl_span, .. }
            | TraitDeclaration { decl_span, .. }
            | StructDeclaration { decl_span, .. }
            | EnumDeclaration { decl_span, .. }
            | ImplTrait { decl_span, .. }
            | ConstantDeclaration { decl_span, .. }
            | StorageDeclaration { decl_span, .. }
            | AbiDeclaration { decl_span, .. } => decl_span.clone(),
            GenericTypeForFunctionScope { name, .. } => name.span(),
            ErrorRecovery(span) => span.clone(),
        }
    }
}

impl DisplayWithEngines for TyDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: Engines<'_>) -> std::fmt::Result {
        let type_engine = engines.te();
        write!(
            f,
            "{} declaration ({})",
            self.friendly_type_name(),
            match self {
                TyDeclaration::VariableDeclaration(decl) => {
                    let TyVariableDeclaration {
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
                TyDeclaration::FunctionDeclaration { name, .. }
                | TyDeclaration::TraitDeclaration { name, .. }
                | TyDeclaration::StructDeclaration { name, .. }
                | TyDeclaration::EnumDeclaration { name, .. } => name.as_str().into(),
                _ => String::new(),
            }
        )
    }
}

impl CollectTypesMetadata for TyDeclaration {
    // this is only run on entry nodes, which must have all well-formed types
    fn collect_types_metadata(
        &self,
        ctx: &mut CollectTypesMetadataContext,
    ) -> CompileResult<Vec<TypeMetadata>> {
        use TyDeclaration::*;
        let mut warnings = vec![];
        let mut errors = vec![];
        let decl_engine = ctx.decl_engine;
        let metadata = match self {
            VariableDeclaration(decl) => {
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
            FunctionDeclaration { decl_id, .. } => {
                let decl = decl_engine.get_function(decl_id);
                check!(
                    decl.collect_types_metadata(ctx),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            ConstantDeclaration { decl_id, .. } => {
                let TyConstantDeclaration { value, .. } = decl_engine.get_constant(decl_id);
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
            | StorageDeclaration { .. }
            | TraitDeclaration { .. }
            | StructDeclaration { .. }
            | EnumDeclaration { .. }
            | ImplTrait { .. }
            | AbiDeclaration { .. }
            | GenericTypeForFunctionScope { .. } => vec![],
        };
        if errors.is_empty() {
            ok(metadata, warnings, errors)
        } else {
            err(warnings, errors)
        }
    }
}

impl GetDeclIdent for TyDeclaration {
    fn get_decl_ident(&self) -> Option<Ident> {
        match self {
            TyDeclaration::VariableDeclaration(decl) => Some(decl.name.clone()),
            TyDeclaration::FunctionDeclaration { name, .. }
            | TyDeclaration::TraitDeclaration { name, .. }
            | TyDeclaration::StructDeclaration { name, .. }
            | TyDeclaration::EnumDeclaration { name, .. }
            | TyDeclaration::ConstantDeclaration { name, .. }
            | TyDeclaration::ImplTrait { name, .. }
            | TyDeclaration::AbiDeclaration { name, .. }
            | TyDeclaration::GenericTypeForFunctionScope { name, .. } => Some(name.clone()),
            TyDeclaration::ErrorRecovery(_) => None,
            TyDeclaration::StorageDeclaration { .. } => None,
        }
    }
}

impl TyDeclaration {
    /// Retrieves the declaration as an enum declaration.
    ///
    /// Returns an error if `self` is not a [TyEnumDeclaration].
    pub(crate) fn expect_enum(&self, decl_engine: &DeclEngine) -> CompileResult<TyEnumDeclaration> {
        match self {
            TyDeclaration::EnumDeclaration { decl_id, .. } => {
                ok(decl_engine.get_enum(decl_id), vec![], vec![])
            }
            TyDeclaration::ErrorRecovery(_) => err(vec![], vec![]),
            decl => err(
                vec![],
                vec![CompileError::DeclIsNotAnEnum {
                    actually: decl.friendly_type_name().to_string(),
                    span: decl.span(),
                }],
            ),
        }
    }

    /// Retrieves the declaration as a struct declaration.
    ///
    /// Returns an error if `self` is not a [TyStructDeclaration].
    pub(crate) fn expect_struct(
        &self,
        decl_engine: &DeclEngine,
    ) -> CompileResult<TyStructDeclaration> {
        let warnings = vec![];
        let mut errors = vec![];
        match self {
            TyDeclaration::StructDeclaration { decl_id, .. } => {
                let decl = decl_engine.get_struct(decl_id);
                ok(decl, warnings, errors)
            }
            TyDeclaration::ErrorRecovery(_) => err(vec![], vec![]),
            decl => {
                errors.push(CompileError::DeclIsNotAStruct {
                    actually: decl.friendly_type_name().to_string(),
                    span: decl.span(),
                });
                err(warnings, errors)
            }
        }
    }

    /// Retrieves the declaration as a function declaration.
    ///
    /// Returns an error if `self` is not a [TyFunctionDeclaration].
    pub(crate) fn expect_function(
        &self,
        decl_engine: &DeclEngine,
    ) -> CompileResult<TyFunctionDeclaration> {
        let warnings = vec![];
        let mut errors = vec![];
        match self {
            TyDeclaration::FunctionDeclaration { decl_id, .. } => {
                let decl = decl_engine.get_function(decl_id);
                ok(decl, warnings, errors)
            }
            TyDeclaration::ErrorRecovery(_) => err(vec![], vec![]),
            decl => {
                errors.push(CompileError::DeclIsNotAFunction {
                    actually: decl.friendly_type_name().to_string(),
                    span: decl.span(),
                });
                err(warnings, errors)
            }
        }
    }

    /// Retrieves the declaration as a variable declaration.
    ///
    /// Returns an error if `self` is not a [TyVariableDeclaration].
    pub(crate) fn expect_variable(&self) -> CompileResult<&TyVariableDeclaration> {
        let warnings = vec![];
        let mut errors = vec![];
        match self {
            TyDeclaration::VariableDeclaration(decl) => ok(decl, warnings, errors),
            TyDeclaration::ErrorRecovery(_) => err(vec![], vec![]),
            decl => {
                errors.push(CompileError::DeclIsNotAVariable {
                    actually: decl.friendly_type_name().to_string(),
                    span: decl.span(),
                });
                err(warnings, errors)
            }
        }
    }

    /// Retrieves the declaration as an Abi declaration.
    ///
    /// Returns an error if `self` is not a [TyAbiDeclaration].
    pub(crate) fn expect_abi(&self, decl_engine: &DeclEngine) -> CompileResult<TyAbiDeclaration> {
        match self {
            TyDeclaration::AbiDeclaration { decl_id, .. } => {
                ok(decl_engine.get_abi(decl_id), vec![], vec![])
            }
            TyDeclaration::ErrorRecovery(_) => err(vec![], vec![]),
            decl => err(
                vec![],
                vec![CompileError::DeclIsNotAnAbi {
                    actually: decl.friendly_type_name().to_string(),
                    span: decl.span(),
                }],
            ),
        }
    }

    /// Retrieves the declaration as an Constant declaration.
    ///
    /// Returns an error if `self` is not a [TyConstantDeclaration].
    pub(crate) fn expect_const(
        &self,
        decl_engine: &DeclEngine,
    ) -> CompileResult<TyConstantDeclaration> {
        match self {
            TyDeclaration::ConstantDeclaration { decl_id, .. } => {
                ok(decl_engine.get_constant(decl_id), vec![], vec![])
            }
            TyDeclaration::ErrorRecovery(_) => err(vec![], vec![]),
            decl => {
                let errors = vec![
                    (CompileError::DeclIsNotAConstant {
                        actually: decl.friendly_type_name().to_string(),
                        span: decl.span(),
                    }),
                ];
                err(vec![], errors)
            }
        }
    }

    /// friendly name string used for error reporting,
    /// which consists of the the identifier for the declaration.
    pub fn friendly_name(&self, engines: &Engines) -> String {
        use TyDeclaration::*;
        let decl_engine = engines.de();
        let type_engine = engines.te();
        match self {
            ImplTrait { decl_id, .. } => {
                let decl = decl_engine.get_impl_trait(decl_id);
                let implementing_for_type_id = type_engine.get(decl.implementing_for.type_id);
                format!(
                    "{} for {}",
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
        use TyDeclaration::*;
        match self {
            VariableDeclaration(_) => "variable",
            ConstantDeclaration { .. } => "constant",
            FunctionDeclaration { .. } => "function",
            TraitDeclaration { .. } => "trait",
            StructDeclaration { .. } => "struct",
            EnumDeclaration { .. } => "enum",
            ImplTrait { .. } => "impl trait",
            AbiDeclaration { .. } => "abi",
            GenericTypeForFunctionScope { .. } => "generic type parameter",
            ErrorRecovery(_) => "error",
            StorageDeclaration { .. } => "contract storage declaration",
        }
    }

    /// name string used in `forc doc` file path generation that mirrors `cargo doc`.
    pub fn doc_name(&self) -> &'static str {
        use TyDeclaration::*;
        match self {
            StructDeclaration { .. } => "struct",
            EnumDeclaration { .. } => "enum",
            TraitDeclaration { .. } => "trait",
            AbiDeclaration { .. } => "abi",
            StorageDeclaration { .. } => "contract_storage",
            ImplTrait { .. } => "impl_trait",
            FunctionDeclaration { .. } => "fn",
            ConstantDeclaration { .. } => "constant",
            _ => unreachable!("these items are non-documentable"),
        }
    }

    pub(crate) fn return_type(&self, engines: Engines<'_>) -> CompileResult<TypeId> {
        let warnings = vec![];
        let mut errors = vec![];
        let type_engine = engines.te();
        let decl_engine = engines.de();
        let type_id = match self {
            TyDeclaration::VariableDeclaration(decl) => decl.body.return_type,
            TyDeclaration::FunctionDeclaration { decl_id, .. } => {
                let decl = decl_engine.get_function(decl_id);
                decl.return_type.type_id
            }
            TyDeclaration::StructDeclaration { decl_id, .. } => {
                let decl = decl_engine.get_struct(decl_id);
                decl.create_type_id(engines)
            }
            TyDeclaration::EnumDeclaration { decl_id, .. } => {
                let decl = decl_engine.get_enum(decl_id);
                decl.create_type_id(engines)
            }
            TyDeclaration::StorageDeclaration { decl_id, .. } => {
                let storage_decl = decl_engine.get_storage(decl_id);
                type_engine.insert(
                    decl_engine,
                    TypeInfo::Storage {
                        fields: storage_decl.fields_as_typed_struct_fields(),
                    },
                )
            }
            TyDeclaration::GenericTypeForFunctionScope { type_id, .. } => *type_id,
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

    pub(crate) fn visibility(&self, decl_engine: &DeclEngine) -> CompileResult<Visibility> {
        use TyDeclaration::*;
        let visibility = match self {
            TraitDeclaration { decl_id, .. } => {
                let TyTraitDeclaration { visibility, .. } = decl_engine.get_trait(decl_id);
                visibility
            }
            ConstantDeclaration { decl_id, .. } => {
                let TyConstantDeclaration { visibility, .. } = decl_engine.get_constant(decl_id);
                visibility
            }
            StructDeclaration { decl_id, .. } => {
                let TyStructDeclaration { visibility, .. } = decl_engine.get_struct(decl_id);
                visibility
            }
            EnumDeclaration { decl_id, .. } => {
                let TyEnumDeclaration { visibility, .. } = decl_engine.get_enum(decl_id);
                visibility
            }
            FunctionDeclaration { decl_id, .. } => {
                let TyFunctionDeclaration { visibility, .. } = decl_engine.get_function(decl_id);
                visibility
            }
            GenericTypeForFunctionScope { .. }
            | ImplTrait { .. }
            | StorageDeclaration { .. }
            | AbiDeclaration { .. }
            | ErrorRecovery(_) => Visibility::Public,
            VariableDeclaration(decl) => decl.mutability.visibility(),
        };
        ok(visibility, vec![], vec![])
    }
}

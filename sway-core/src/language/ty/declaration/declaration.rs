use sway_error::error::CompileError;
use sway_types::{Ident, Span, Spanned};

use crate::{
    declaration_engine::*,
    engine_threading::*,
    error::*,
    language::{ty::*, Visibility},
    type_system::*,
};

#[derive(Clone, Debug)]
pub enum TyDeclaration {
    VariableDeclaration(Box<TyVariableDeclaration>),
    ConstantDeclaration(DeclarationId),
    FunctionDeclaration(DeclarationId),
    TraitDeclaration(DeclarationId),
    StructDeclaration(DeclarationId),
    EnumDeclaration(DeclarationId),
    ImplTrait(DeclarationId),
    AbiDeclaration(DeclarationId),
    // If type parameters are defined for a function, they are put in the namespace just for
    // the body of that function.
    GenericTypeForFunctionScope { name: Ident, type_id: TypeId },
    ErrorRecovery(Span),
    StorageDeclaration(DeclarationId),
}

impl EqWithEngines for TyDeclaration {}
impl PartialEqWithEngines for TyDeclaration {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        match (self, other) {
            (Self::VariableDeclaration(x), Self::VariableDeclaration(y)) => x.eq(y, engines),
            (Self::ConstantDeclaration(x), Self::ConstantDeclaration(y)) => x.eq(y, engines),
            (Self::FunctionDeclaration(x), Self::FunctionDeclaration(y)) => x.eq(y, engines),
            (Self::TraitDeclaration(x), Self::TraitDeclaration(y)) => x.eq(y, engines),
            (Self::StructDeclaration(x), Self::StructDeclaration(y)) => x.eq(y, engines),
            (Self::EnumDeclaration(x), Self::EnumDeclaration(y)) => x.eq(y, engines),
            (Self::ImplTrait(x), Self::ImplTrait(y)) => x.eq(y, engines),
            (Self::AbiDeclaration(x), Self::AbiDeclaration(y)) => x.eq(y, engines),
            (Self::StorageDeclaration(x), Self::StorageDeclaration(y)) => x.eq(y, engines),
            (
                Self::GenericTypeForFunctionScope {
                    name: xn,
                    type_id: xti,
                },
                Self::GenericTypeForFunctionScope {
                    name: yn,
                    type_id: yti,
                },
            ) => xn == yn && xti == yti,
            (Self::ErrorRecovery(x), Self::ErrorRecovery(y)) => x == y,
            _ => false,
        }
    }
}

impl CopyTypes for TyDeclaration {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, engines: Engines<'_>) {
        use TyDeclaration::*;
        match self {
            VariableDeclaration(ref mut var_decl) => var_decl.copy_types(type_mapping, engines),
            FunctionDeclaration(ref mut decl_id) => decl_id.copy_types(type_mapping, engines),
            TraitDeclaration(ref mut decl_id) => decl_id.copy_types(type_mapping, engines),
            StructDeclaration(ref mut decl_id) => decl_id.copy_types(type_mapping, engines),
            EnumDeclaration(ref mut decl_id) => decl_id.copy_types(type_mapping, engines),
            ImplTrait(decl_id) => decl_id.copy_types(type_mapping, engines),
            // generics in an ABI is unsupported by design
            AbiDeclaration(..)
            | ConstantDeclaration(_)
            | StorageDeclaration(..)
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
            FunctionDeclaration(ref mut decl_id) => decl_id.replace_self_type(engines, self_type),
            TraitDeclaration(ref mut decl_id) => decl_id.replace_self_type(engines, self_type),
            StructDeclaration(ref mut decl_id) => decl_id.replace_self_type(engines, self_type),
            EnumDeclaration(ref mut decl_id) => decl_id.replace_self_type(engines, self_type),
            ImplTrait(decl_id) => decl_id.replace_self_type(engines, self_type),
            // generics in an ABI is unsupported by design
            AbiDeclaration(..)
            | ConstantDeclaration(_)
            | StorageDeclaration(..)
            | GenericTypeForFunctionScope { .. }
            | ErrorRecovery(_) => (),
        }
    }
}

impl Spanned for TyDeclaration {
    fn span(&self) -> Span {
        use TyDeclaration::*;
        match self {
            VariableDeclaration(decl) => decl.name.span(),
            ConstantDeclaration(decl_id) => decl_id.span(),
            FunctionDeclaration(decl_id) => decl_id.span(),
            TraitDeclaration(decl_id) => decl_id.span(),
            StructDeclaration(decl_id) => decl_id.span(),
            EnumDeclaration(decl_id) => decl_id.span(),
            AbiDeclaration(decl_id) => decl_id.span(),
            ImplTrait(decl_id) => decl_id.span(),
            StorageDeclaration(decl) => decl.span(),
            GenericTypeForFunctionScope { name, .. } => name.span(),
            ErrorRecovery(span) => span.clone(),
        }
    }
}

impl DisplayWithEngines for TyDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, type_engine: &TypeEngine) -> std::fmt::Result {
        write!(
            f,
            "{} declaration ({})",
            self.friendly_name(),
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
                        VariableMutability::ExportedConst => builder.push_str("pub const"),
                    }
                    builder.push_str(name.as_str());
                    builder.push_str(": ");
                    builder.push_str(
                        &type_engine
                            .help_out(type_engine.look_up_type_id(*type_ascription))
                            .to_string(),
                    );
                    builder.push_str(" = ");
                    builder.push_str(&type_engine.help_out(body).to_string());
                    builder
                }
                TyDeclaration::FunctionDeclaration(decl_id) => {
                    match de_get_function(decl_id.clone(), &decl_id.span()) {
                        Ok(TyFunctionDeclaration { name, .. }) => name.as_str().into(),
                        Err(_) => "unknown function".into(),
                    }
                }
                TyDeclaration::TraitDeclaration(decl_id) => {
                    match de_get_trait(decl_id.clone(), &decl_id.span()) {
                        Ok(TyTraitDeclaration { name, .. }) => name.as_str().into(),
                        Err(_) => "unknown trait".into(),
                    }
                }
                TyDeclaration::StructDeclaration(decl_id) => {
                    match de_get_struct(decl_id.clone(), &decl_id.span()) {
                        Ok(TyStructDeclaration { name, .. }) => name.as_str().into(),
                        Err(_) => "unknown struct".into(),
                    }
                }
                TyDeclaration::EnumDeclaration(decl_id) => {
                    match de_get_enum(decl_id.clone(), &decl_id.span()) {
                        Ok(TyEnumDeclaration { name, .. }) => name.as_str().into(),
                        Err(_) => "unknown enum".into(),
                    }
                }
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
        let metadata = match self {
            VariableDeclaration(decl) => {
                let mut body = check!(
                    decl.body.collect_types_metadata(ctx),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                body.append(&mut check!(
                    decl.type_ascription.collect_types_metadata(ctx),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
                body
            }
            FunctionDeclaration(decl_id) => match de_get_function(decl_id.clone(), &decl_id.span())
            {
                Ok(decl) => {
                    check!(
                        decl.collect_types_metadata(ctx),
                        return err(warnings, errors),
                        warnings,
                        errors
                    )
                }
                Err(e) => {
                    errors.push(e);
                    return err(warnings, errors);
                }
            },
            ConstantDeclaration(decl_id) => {
                match de_get_constant(decl_id.clone(), &decl_id.span()) {
                    Ok(TyConstantDeclaration { value, .. }) => {
                        check!(
                            value.collect_types_metadata(ctx),
                            return err(warnings, errors),
                            warnings,
                            errors
                        )
                    }
                    Err(e) => {
                        errors.push(e);
                        return err(warnings, errors);
                    }
                }
            }
            ErrorRecovery(_)
            | StorageDeclaration(_)
            | TraitDeclaration(_)
            | StructDeclaration(_)
            | EnumDeclaration(_)
            | ImplTrait { .. }
            | AbiDeclaration(_)
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
            TyDeclaration::ConstantDeclaration(decl) => {
                Some(de_get_constant(decl.clone(), &decl.span()).unwrap().name)
            }
            TyDeclaration::FunctionDeclaration(decl) => {
                Some(de_get_function(decl.clone(), &decl.span()).unwrap().name)
            }
            TyDeclaration::TraitDeclaration(decl) => {
                Some(de_get_trait(decl.clone(), &decl.span()).unwrap().name)
            }
            TyDeclaration::StructDeclaration(decl) => {
                Some(de_get_struct(decl.clone(), &decl.span()).unwrap().name)
            }
            TyDeclaration::EnumDeclaration(decl) => {
                Some(de_get_enum(decl.clone(), &decl.span()).unwrap().name)
            }
            TyDeclaration::ImplTrait(decl) => Some(
                de_get_impl_trait(decl.clone(), &decl.span())
                    .unwrap()
                    .trait_name
                    .suffix,
            ),
            TyDeclaration::AbiDeclaration(decl) => {
                Some(de_get_abi(decl.clone(), &decl.span()).unwrap().name)
            }
            TyDeclaration::GenericTypeForFunctionScope { name, .. } => Some(name.clone()),
            TyDeclaration::ErrorRecovery(_) => None,
            TyDeclaration::StorageDeclaration(_decl) => None,
        }
    }
}

impl TyDeclaration {
    /// Retrieves the declaration as an enum declaration.
    ///
    /// Returns an error if `self` is not a [TyEnumDeclaration].
    pub(crate) fn expect_enum(&self, access_span: &Span) -> CompileResult<TyEnumDeclaration> {
        match self {
            TyDeclaration::EnumDeclaration(decl_id) => {
                CompileResult::from(de_get_enum(decl_id.clone(), access_span))
            }
            TyDeclaration::ErrorRecovery(_) => err(vec![], vec![]),
            decl => err(
                vec![],
                vec![CompileError::DeclIsNotAnEnum {
                    actually: decl.friendly_name().to_string(),
                    span: decl.span(),
                }],
            ),
        }
    }

    /// Retrieves the declaration as a struct declaration.
    ///
    /// Returns an error if `self` is not a [TyStructDeclaration].
    pub(crate) fn expect_struct(&self, access_span: &Span) -> CompileResult<TyStructDeclaration> {
        let mut warnings = vec![];
        let mut errors = vec![];
        match self {
            TyDeclaration::StructDeclaration(decl_id) => {
                let decl = check!(
                    CompileResult::from(de_get_struct(decl_id.clone(), access_span)),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                ok(decl, warnings, errors)
            }
            TyDeclaration::ErrorRecovery(_) => err(vec![], vec![]),
            decl => {
                errors.push(CompileError::DeclIsNotAStruct {
                    actually: decl.friendly_name().to_string(),
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
        access_span: &Span,
    ) -> CompileResult<TyFunctionDeclaration> {
        let mut warnings = vec![];
        let mut errors = vec![];
        match self {
            TyDeclaration::FunctionDeclaration(decl_id) => {
                let decl = check!(
                    CompileResult::from(de_get_function(decl_id.clone(), access_span)),
                    return err(warnings, errors),
                    warnings,
                    errors,
                );
                ok(decl, warnings, errors)
            }
            TyDeclaration::ErrorRecovery(_) => err(vec![], vec![]),
            decl => {
                errors.push(CompileError::DeclIsNotAFunction {
                    actually: decl.friendly_name().to_string(),
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
                    actually: decl.friendly_name().to_string(),
                    span: decl.span(),
                });
                err(warnings, errors)
            }
        }
    }

    /// Retrieves the declaration as an Abi declaration.
    ///
    /// Returns an error if `self` is not a [TyAbiDeclaration].
    pub(crate) fn expect_abi(&self, access_span: &Span) -> CompileResult<TyAbiDeclaration> {
        match self {
            TyDeclaration::AbiDeclaration(decl_id) => {
                CompileResult::from(de_get_abi(decl_id.clone(), access_span))
            }
            TyDeclaration::ErrorRecovery(_) => err(vec![], vec![]),
            decl => err(
                vec![],
                vec![CompileError::DeclIsNotAnAbi {
                    actually: decl.friendly_name().to_string(),
                    span: decl.span(),
                }],
            ),
        }
    }

    /// Retrieves the declaration as an Constant declaration.
    ///
    /// Returns an error if `self` is not a [TyConstantDeclaration].
    pub(crate) fn expect_const(&self, access_span: &Span) -> CompileResult<TyConstantDeclaration> {
        match self {
            TyDeclaration::ConstantDeclaration(decl) => {
                CompileResult::from(de_get_constant(decl.clone(), access_span))
            }
            TyDeclaration::ErrorRecovery(_) => err(vec![], vec![]),
            decl => {
                let errors = vec![
                    (CompileError::DeclIsNotAConstant {
                        actually: decl.friendly_name().to_string(),
                        span: decl.span(),
                    }),
                ];
                err(vec![], errors)
            }
        }
    }

    /// friendly name string used for error reporting.
    pub fn friendly_name(&self) -> &'static str {
        use TyDeclaration::*;
        match self {
            VariableDeclaration(_) => "variable",
            ConstantDeclaration(_) => "constant",
            FunctionDeclaration(_) => "function",
            TraitDeclaration(_) => "trait",
            StructDeclaration(_) => "struct",
            EnumDeclaration(_) => "enum",
            ImplTrait { .. } => "impl trait",
            AbiDeclaration(..) => "abi",
            GenericTypeForFunctionScope { .. } => "generic type parameter",
            ErrorRecovery(_) => "error",
            StorageDeclaration(_) => "contract storage declaration",
        }
    }

    pub(crate) fn return_type(
        &self,
        access_span: &Span,
        type_engine: &TypeEngine,
    ) -> CompileResult<TypeId> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let type_id = match self {
            TyDeclaration::VariableDeclaration(decl) => decl.body.return_type,
            TyDeclaration::FunctionDeclaration(decl_id) => {
                let decl = check!(
                    CompileResult::from(de_get_function(decl_id.clone(), &self.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                decl.return_type
            }
            TyDeclaration::StructDeclaration(decl_id) => {
                let decl = check!(
                    CompileResult::from(de_get_struct(decl_id.clone(), &self.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                decl.create_type_id(type_engine)
            }
            TyDeclaration::EnumDeclaration(decl_id) => {
                let decl = check!(
                    CompileResult::from(de_get_enum(decl_id.clone(), access_span)),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                decl.create_type_id(type_engine)
            }
            TyDeclaration::StorageDeclaration(decl_id) => {
                let storage_decl = check!(
                    CompileResult::from(de_get_storage(decl_id.clone(), &self.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                type_engine.insert_type(TypeInfo::Storage {
                    fields: storage_decl.fields_as_typed_struct_fields(),
                })
            }
            TyDeclaration::GenericTypeForFunctionScope { type_id, .. } => *type_id,
            decl => {
                errors.push(CompileError::NotAType {
                    span: decl.span(),
                    name: type_engine.help_out(decl).to_string(),
                    actually_is: decl.friendly_name(),
                });
                return err(warnings, errors);
            }
        };
        ok(type_id, warnings, errors)
    }

    pub(crate) fn visibility(&self) -> CompileResult<Visibility> {
        use TyDeclaration::*;
        let mut warnings = vec![];
        let mut errors = vec![];
        let visibility = match self {
            TraitDeclaration(decl_id) => {
                let TyTraitDeclaration { visibility, .. } = check!(
                    CompileResult::from(de_get_trait(decl_id.clone(), &decl_id.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                visibility
            }
            ConstantDeclaration(decl_id) => {
                let TyConstantDeclaration { visibility, .. } = check!(
                    CompileResult::from(de_get_constant(decl_id.clone(), &decl_id.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                visibility
            }
            StructDeclaration(decl_id) => {
                let TyStructDeclaration { visibility, .. } = check!(
                    CompileResult::from(de_get_struct(decl_id.clone(), &decl_id.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                visibility
            }
            EnumDeclaration(decl_id) => {
                let TyEnumDeclaration { visibility, .. } = check!(
                    CompileResult::from(de_get_enum(decl_id.clone(), &decl_id.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                visibility
            }
            FunctionDeclaration(decl_id) => {
                let TyFunctionDeclaration { visibility, .. } = check!(
                    CompileResult::from(de_get_function(decl_id.clone(), &decl_id.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                visibility
            }
            GenericTypeForFunctionScope { .. }
            | ImplTrait { .. }
            | StorageDeclaration { .. }
            | AbiDeclaration(..)
            | ErrorRecovery(_) => Visibility::Public,
            VariableDeclaration(decl) => decl.mutability.visibility(),
        };
        ok(visibility, warnings, errors)
    }
}

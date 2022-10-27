use sway_types::Spanned;

use crate::{language::ty::*, type_system::*};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TyMethodDeclaration {
    MethodDef(TyFunctionDeclaration),
    MethodImpl(TyTraitFn),
}

impl CopyTypes for TyMethodDeclaration {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping) {
        match self {
            TyMethodDeclaration::MethodDef(decl) => decl.copy_types(type_mapping),
            TyMethodDeclaration::MethodImpl(decl) => decl.copy_types(type_mapping),
        }
    }
}

impl ReplaceSelfType for TyMethodDeclaration {
    fn replace_self_type(&mut self, self_type: TypeId) {
        match self {
            TyMethodDeclaration::MethodDef(decl) => decl.replace_self_type(self_type),
            TyMethodDeclaration::MethodImpl(decl) => decl.replace_self_type(self_type),
        }
    }
}

impl Spanned for TyMethodDeclaration {
    fn span(&self) -> sway_types::Span {
        match self {
            TyMethodDeclaration::MethodDef(decl) => decl.span(),
            TyMethodDeclaration::MethodImpl(decl) => decl.span(),
        }
    }
}

impl CollectTypesMetadata for TyMethodDeclaration {
    fn collect_types_metadata(
        &self,
        ctx: &mut CollectTypesMetadataContext,
    ) -> crate::CompileResult<Vec<TypeMetadata>> {
        match self {
            TyMethodDeclaration::MethodDef(decl) => decl.collect_types_metadata(ctx),
            TyMethodDeclaration::MethodImpl(decl) => decl.collect_types_metadata(ctx),
        }
    }
}

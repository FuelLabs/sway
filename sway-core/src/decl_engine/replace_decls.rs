use crate::{
    engine_threading::Engines,
    language::ty::{self, TyDeclaration},
};

use super::DeclMapping;

pub trait ReplaceDecls {
    fn replace_decls_inner(&mut self, decl_mapping: &DeclMapping, engines: Engines<'_>);

    fn replace_decls(&mut self, decl_mapping: &DeclMapping, engines: Engines<'_>) {
        if !decl_mapping.is_empty() {
            self.replace_decls_inner(decl_mapping, engines);
        }
    }
}

pub(crate) trait ReplaceFunctionImplementingType {
    fn replace_implementing_type(
        &mut self,
        engines: Engines<'_>,
        implementing_type: ty::TyDeclaration,
    );
}

pub(crate) trait UpdateConstantExpression {
    fn update_constant_expression(
        &mut self,
        engines: Engines<'_>,
        implementing_type: &TyDeclaration,
    );
}

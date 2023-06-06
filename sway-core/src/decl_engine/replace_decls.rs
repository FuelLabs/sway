use crate::{
    engine_threading::Engines,
    language::ty::{self, TyDecl},
};

use super::DeclMapping;

pub trait ReplaceDecls {
    fn replace_decls_inner(&mut self, decl_mapping: &DeclMapping, engines: &Engines);

    fn replace_decls(&mut self, decl_mapping: &DeclMapping, engines: &Engines) {
        if !decl_mapping.is_empty() {
            self.replace_decls_inner(decl_mapping, engines);
        }
    }
}

pub(crate) trait ReplaceFunctionImplementingType {
    fn replace_implementing_type(&mut self, engines: &Engines, implementing_type: ty::TyDecl);
}

pub(crate) trait UpdateConstantExpression {
    fn update_constant_expression(&mut self, engines: &Engines, implementing_type: &TyDecl);
}

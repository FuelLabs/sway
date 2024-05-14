use sway_error::handler::{ErrorEmitted, Handler};

use crate::{
    engine_threading::Engines,
    language::ty::{self, TyDecl},
    semantic_analysis::TypeCheckContext,
};

use super::DeclMapping;

pub trait ReplaceDecls {
    fn replace_decls_inner(
        &mut self,
        decl_mapping: &DeclMapping,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
    ) -> Result<bool, ErrorEmitted>;

    fn replace_decls(
        &mut self,
        decl_mapping: &DeclMapping,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
    ) -> Result<bool, ErrorEmitted> {
        if !decl_mapping.is_empty() {
            self.replace_decls_inner(decl_mapping, handler, ctx)
        } else {
            Ok(false)
        }
    }
}

pub(crate) trait ReplaceFunctionImplementingType {
    fn replace_implementing_type(&mut self, engines: &Engines, implementing_type: ty::TyDecl);
}

pub(crate) trait UpdateConstantExpression {
    fn update_constant_expression(&mut self, engines: &Engines, implementing_type: &TyDecl);
}

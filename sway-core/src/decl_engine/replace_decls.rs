use sway_error::handler::{ErrorEmitted, Handler};

use crate::{
    engine_threading::Engines,
    language::ty::{self, TyDecl, TyExpression},
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

impl<T: ReplaceDecls + Clone> ReplaceDecls for std::sync::Arc<T> {
    fn replace_decls_inner(
        &mut self,
        decl_mapping: &DeclMapping,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
    ) -> Result<bool, ErrorEmitted> {
        if let Some(item) = std::sync::Arc::get_mut(self) {
            item.replace_decls_inner(decl_mapping, handler, ctx)
        } else {
            let mut item = self.as_ref().clone();
            let r = item.replace_decls_inner(decl_mapping, handler, ctx)?;
            *self = std::sync::Arc::new(item);
            Ok(r)
        }
    }
}

pub(crate) trait ReplaceFunctionImplementingType {
    fn replace_implementing_type(&mut self, engines: &Engines, implementing_type: ty::TyDecl);
}

pub(crate) trait UpdateConstantExpression {
    fn update_constant_expression(&mut self, engines: &Engines, implementing_type: &TyDecl);
}

impl<T: UpdateConstantExpression + Clone> UpdateConstantExpression for std::sync::Arc<T> {
    fn update_constant_expression(&mut self, engines: &Engines, implementing_type: &TyDecl) {
        if let Some(item) = std::sync::Arc::get_mut(self) {
            item.update_constant_expression(engines, implementing_type);
        } else {
            let mut item = self.as_ref().clone();
            item.update_constant_expression(engines, implementing_type);
            *self = std::sync::Arc::new(item);
        }
    }
}

// Iterate the tree searching for references to a const generic,
// and initialize its value with the passed value
pub(crate) trait MaterializeConstGenerics {
    fn materialize_const_generics(
        &mut self,
        engines: &Engines,
        handler: &Handler,
        name: &str,
        value: &TyExpression,
    ) -> Result<(), ErrorEmitted>;
}

impl<T: MaterializeConstGenerics + Clone> MaterializeConstGenerics for std::sync::Arc<T> {
    fn materialize_const_generics(
        &mut self,
        engines: &Engines,
        handler: &Handler,
        name: &str,
        value: &TyExpression,
    ) -> Result<(), ErrorEmitted> {
        if let Some(item) = std::sync::Arc::get_mut(self) {
            item.materialize_const_generics(engines, handler, name, value)
        } else {
            let mut item = self.as_ref().clone();
            let r = item.materialize_const_generics(engines, handler, name, value);
            *self = std::sync::Arc::new(item);
            r
        }
    }
}

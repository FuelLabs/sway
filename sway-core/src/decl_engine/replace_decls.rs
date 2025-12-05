use std::borrow::Cow;

use crate::{
    GenericTypeArgument, Length, TypeArgs, TypeInfo, ast_elements::type_parameter::{ConstGenericExpr, ConstGenericExprTyDecl}, decl_engine::{DeclEngineGet as _, DeclEngineGetParsedDecl as _, DeclEngineInsert}, engine_threading::Engines, language::ty::{
        self, ConstGenericDecl, TyCodeBlock, TyConstGenericDecl, TyDecl, TyEnumDecl, TyExpression, TyFunctionDecl, TyStructDecl
    }, semantic_analysis::{TypeCheckContext, Visitor}
};
use sway_error::handler::{ErrorEmitted, Handler};

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

pub struct UpdateConstantExpressionVisitor<'a> {
    pub engines: &'a Engines,
    pub implementing_type: &'a TyDecl,
}

impl Visitor for UpdateConstantExpressionVisitor<'_> {
    fn visit_ty_constant_decl(&mut self, item: &mut std::borrow::Cow<ty::TyConstantDecl>) {
        if let Some(impl_const) =
            ty::find_const_decl_from_impl(self.implementing_type, self.engines.de(), item)
        {
            *item = Cow::Owned(impl_const);
        }
    }
}

pub fn update_constant_expression_visitor_on_block(
    engines: &Engines,
    implementing_type: &TyDecl,
    block: &mut TyCodeBlock,
) {
    let mut visitor = UpdateConstantExpressionVisitor {
        engines,
        implementing_type,
    };
    let mut cow = std::borrow::Cow::Borrowed(&*block);
    TyCodeBlock::visit(&mut cow, &mut visitor);
    if let std::borrow::Cow::Owned(new_block) = cow {
        *block = new_block;
    }
}

// pub(crate) trait UpdateConstantExpression {
//     fn update_constant_expression(&mut self, engines: &Engines, implementing_type: &TyDecl);
// }

// impl<T: UpdateConstantExpression + Clone> UpdateConstantExpression for std::sync::Arc<T> {
//     fn update_constant_expression(&mut self, engines: &Engines, implementing_type: &TyDecl) {
//         if let Some(item) = std::sync::Arc::get_mut(self) {
//             item.update_constant_expression(engines, implementing_type);
//         } else {
//             let mut item = self.as_ref().clone();
//             item.update_constant_expression(engines, implementing_type);
//             *self = std::sync::Arc::new(item);
//         }
//     }
// }

pub struct MaterializeConstGenericsVisitor<'a> {
    pub engines: &'a Engines,
    pub handler: &'a Handler,
    pub name: &'a str,
    pub value: &'a TyExpression,
}

impl Visitor for MaterializeConstGenericsVisitor<'_> {
    
    fn visit_ty_const_generic_decl(&mut self, item: &mut std::borrow::Cow<TyConstGenericDecl>) {
        if item.call_path.suffix.as_str() == self.name {
            match item.value.as_ref() {
                Some(v) => {
                    assert!(
                        v.extract_literal_value()
                            .unwrap()
                            .cast_value_to_u64()
                            .unwrap()
                            == self
                                .value
                                .extract_literal_value()
                                .unwrap()
                                .cast_value_to_u64()
                                .unwrap(),
                        "{v:?} {:?}",
                        self.value,
                    );
                }
                None => {
                    item.to_mut().value = Some(self.value.clone());
                }
            }
        }
    }

    fn visit_decl_ref_function(&mut self, item: &mut std::borrow::Cow<super::DeclRefFunction>) {
        let decl = self.engines.de().get(item.id());
        let mut cow = Cow::Borrowed(decl.as_ref());
        TyFunctionDecl::visit(&mut cow, self);

        if let Cow::Owned(new_decl) = cow {
            *item = Cow::Owned(self.engines.de().insert(new_decl, None));
        }
    }

    fn visit_decl_ref_enum(&mut self,item: &mut std::borrow::Cow<super::DeclRefEnum>) {
        let decl = self.engines.de().get(item.id());
        let mut cow = Cow::Borrowed(decl.as_ref());
        TyEnumDecl::visit(&mut cow, self);

        if let Cow::Owned(new_decl) = cow {
            *item = Cow::Owned(self.engines.de().insert(new_decl, None));
        }
    }

    fn visit_decl_id_ty_struct_decl(&mut self, item: &mut std::borrow::Cow<super::DeclId<TyStructDecl> >) {
        let decl = self.engines.de().get(item.as_ref());
        let mut cow = Cow::Borrowed(decl.as_ref());
        TyStructDecl::visit(&mut cow, self);

        if let Cow::Owned(new_decl) = cow {
            let parsed_decl = self.engines
                .de()
                .get_parsed_decl(item.as_ref())
                .unwrap()
                .to_struct_decl(self.handler, self.engines)
                .ok();
            
            let new_ref = self.engines.de().insert(new_decl, parsed_decl.as_ref());
            *item = Cow::Owned(*new_ref.id());
        }
    }

    fn visit_empty_type_binding(&mut self, item: &mut std::borrow::Cow<crate::ast_elements::binding::EmptyTypeBinding>) {
        let mut cow = Cow::Borrowed(&item.type_arguments);
        TypeArgs::visit(&mut cow, self);
        if let Cow::Owned(new_type_arguments) = cow {
            item.to_mut().type_arguments = new_type_arguments;
        }
    }

    fn visit_type_binding_call_path(&mut self, item: &mut std::borrow::Cow<crate::TypeBinding<crate::language::CallPath>>) {
        let mut cow = Cow::Borrowed(&item.type_arguments);
        TypeArgs::visit(&mut cow, self);
        if let Cow::Owned(new_type_arguments) = cow {
            item.to_mut().type_arguments = new_type_arguments;
        }
    }

    fn visit_type_id(&mut self, item: &mut std::borrow::Cow<crate::TypeId>) {
        match &*self.engines.te().get(*item.as_ref()) {
            TypeInfo::Array(
                element_type,
                Length(ConstGenericExpr::AmbiguousVariableExpression { ident, decl }),
            ) => {
                let mut cow = Cow::Borrowed(element_type);
                GenericTypeArgument::visit(&mut cow, self);
                if let Cow::Owned(new_element_type) = cow {
                    todo!()
                }

                if ident.as_str() == self.name {
                    let val = match &self.value.expression {
                        crate::language::ty::TyExpressionVariant::Literal(literal) => {
                            literal.cast_value_to_u64().unwrap()
                        }
                        _ => {
                            todo!("Will be implemented by https://github.com/FuelLabs/sway/issues/6860")
                        }
                    };

                    let new_type_id = self.engines.te().insert_array(
                        self.engines,
                        element_type.clone(),
                        Length(ConstGenericExpr::Literal {
                            val: val as usize,
                            span: self.value.span.clone(),
                        }),
                    );
                    *item = Cow::Owned(new_type_id);
                } else {
                    // let mut decl = decl.clone();
                    if let Some(decl) = decl.as_ref() {
                        // decl.materialize_const_generics(engines, handler, name, value)?;
                        let mut cow = Cow::Borrowed(decl);
                        ConstGenericExprTyDecl::visit(&mut cow, self);
                        if let Cow::Owned(new_decl) = cow {
                            todo!()
                        }
                    }

                    let new_type_id = self.engines.te().insert_array(
                        self.engines,
                        element_type.clone(),
                        Length(ConstGenericExpr::AmbiguousVariableExpression {
                            ident: ident.clone(),
                            decl: decl.clone(),
                        }),
                    );
                }
            }
            TypeInfo::Enum(id) => {
                // let decl = engines.de().get(id);
                // let mut decl = (*decl).clone();
                // decl.materialize_const_generics(engines, handler, name, value)?;

                // let parsed_decl = engines
                //     .de()
                //     .get_parsed_decl(id)
                //     .unwrap()
                //     .to_enum_decl(handler, engines)
                //     .ok();
                // let decl_ref = engines.de().insert(decl, parsed_decl.as_ref());

                // *self = engines.te().insert_enum(engines, *decl_ref.id());
                todo!()
            }
            TypeInfo::Struct(id) => {
                let mut cow = Cow::Borrowed(id);
                self.visit_decl_id_ty_struct_decl(&mut cow);
                if let Cow::Owned(new_decl_id) = cow {
                    let new_type_id = self.engines.te().insert_struct(self.engines, new_decl_id);
                    *item = Cow::Owned(new_type_id)
                }
            }
            TypeInfo::StringArray(Length(ConstGenericExpr::AmbiguousVariableExpression {
                ident,
                ..
            })) if ident.as_str() == self.name => {
                // let val = match &value.expression {
                //     crate::language::ty::TyExpressionVariant::Literal(literal) => {
                //         literal.cast_value_to_u64().unwrap()
                //     }
                //     _ => {
                //         todo!("Will be implemented by https://github.com/FuelLabs/sway/issues/6860")
                //     }
                // };

                // *self = engines.te().insert_string_array(
                //     engines,
                //     Length(ConstGenericExpr::Literal {
                //         val: val as usize,
                //         span: value.span.clone(),
                //     }),
                // );
                todo!()
            }
            TypeInfo::Ref {
                to_mutable_value,
                referenced_type,
                ..
            } => {
                let mut cow = Cow::Borrowed(referenced_type);
                GenericTypeArgument::visit(&mut cow, self);
                if let Cow::Owned(new_referenced_type) = cow {
                    let new_type_id = self.engines
                         .te()
                         .insert_ref(self.engines, *to_mutable_value, new_referenced_type);
                    *item = Cow::Owned(new_type_id)
                }
            }
            _ => {}
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

impl<T: MaterializeConstGenerics> MaterializeConstGenerics for Vec<T> {
    fn materialize_const_generics(
        &mut self,
        engines: &Engines,
        handler: &Handler,
        name: &str,
        value: &TyExpression,
    ) -> Result<(), ErrorEmitted> {
        for item in self.iter_mut() {
            item.materialize_const_generics(engines, handler, name, value)?;
        }
        Ok(())
    }
}

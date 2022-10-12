#![allow(dead_code)]

use crate::{
    core::token::{TokenMap, TypeDefinition, TypedAstToken},
    utils::token::{struct_declaration_of_type_id, to_ident_key},
};
use sway_core::{declaration_engine, language::ty};
use sway_types::{ident::Ident, Spanned};

pub fn traverse_node(node: &ty::TyAstNode, tokens: &TokenMap) {
    match &node.content {
        ty::TyAstNodeContent::Declaration(declaration) => handle_declaration(declaration, tokens),
        ty::TyAstNodeContent::Expression(expression) => handle_expression(expression, tokens),
        ty::TyAstNodeContent::ImplicitReturnExpression(expression) => {
            handle_expression(expression, tokens)
        }
        ty::TyAstNodeContent::SideEffect => (),
    };
}

fn handle_declaration(declaration: &ty::TyDeclaration, tokens: &TokenMap) {
    match declaration {
        ty::TyDeclaration::VariableDeclaration(variable) => {
            if let Some(mut token) = tokens.get_mut(&to_ident_key(&variable.name)) {
                token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
            }
            if let Some(type_ascription_span) = &variable.type_ascription_span {
                if let Some(mut token) =
                    tokens.get_mut(&to_ident_key(&Ident::new(type_ascription_span.clone())))
                {
                    token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                    token.type_def = Some(TypeDefinition::TypeId(variable.type_ascription));
                }
            }

            handle_expression(&variable.body, tokens);
        }
        ty::TyDeclaration::ConstantDeclaration(decl_id) => {
            if let Ok(const_decl) =
                declaration_engine::de_get_constant(decl_id.clone(), &decl_id.span())
            {
                if let Some(mut token) = tokens.get_mut(&to_ident_key(&const_decl.name)) {
                    token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                }
                handle_expression(&const_decl.value, tokens);
            }
        }
        ty::TyDeclaration::FunctionDeclaration(decl_id) => {
            if let Ok(func_decl) =
                declaration_engine::de_get_function(decl_id.clone(), &decl_id.span())
            {
                collect_typed_fn_decl(&func_decl, tokens);
            }
        }
        ty::TyDeclaration::TraitDeclaration(decl_id) => {
            if let Ok(trait_decl) =
                declaration_engine::de_get_trait(decl_id.clone(), &decl_id.span())
            {
                if let Some(mut token) = tokens.get_mut(&to_ident_key(&trait_decl.name)) {
                    token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                }

                for trait_fn in &trait_decl.interface_surface {
                    collect_typed_trait_fn_token(trait_fn, tokens);
                }
            }
        }
        ty::TyDeclaration::StructDeclaration(decl_id) => {
            if let Ok(struct_decl) =
                declaration_engine::de_get_struct(decl_id.clone(), &declaration.span())
            {
                if let Some(mut token) = tokens.get_mut(&to_ident_key(&struct_decl.name)) {
                    token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                }

                for field in &struct_decl.fields {
                    if let Some(mut token) = tokens.get_mut(&to_ident_key(&field.name)) {
                        token.typed = Some(TypedAstToken::TypedStructField(field.clone()));
                        token.type_def = Some(TypeDefinition::TypeId(field.type_id));
                    }

                    if let Some(mut token) =
                        tokens.get_mut(&to_ident_key(&Ident::new(field.type_span.clone())))
                    {
                        token.typed = Some(TypedAstToken::TypedStructField(field.clone()));
                        token.type_def = Some(TypeDefinition::TypeId(field.type_id));
                    }
                }

                for type_param in &struct_decl.type_parameters {
                    if let Some(mut token) = tokens.get_mut(&to_ident_key(&type_param.name_ident)) {
                        token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                        token.type_def = Some(TypeDefinition::TypeId(type_param.type_id));
                    }
                }
            }
        }
        ty::TyDeclaration::EnumDeclaration(decl_id) => {
            if let Ok(enum_decl) = declaration_engine::de_get_enum(decl_id.clone(), &decl_id.span())
            {
                if let Some(mut token) = tokens.get_mut(&to_ident_key(&enum_decl.name)) {
                    token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                }

                for type_param in &enum_decl.type_parameters {
                    if let Some(mut token) = tokens.get_mut(&to_ident_key(&type_param.name_ident)) {
                        token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                        token.type_def = Some(TypeDefinition::TypeId(type_param.type_id));
                    }
                }

                for variant in &enum_decl.variants {
                    if let Some(mut token) = tokens.get_mut(&to_ident_key(&variant.name)) {
                        token.typed = Some(TypedAstToken::TypedEnumVariant(variant.clone()));
                        token.type_def = Some(TypeDefinition::TypeId(variant.type_id));
                    }

                    if let Some(mut token) =
                        tokens.get_mut(&to_ident_key(&Ident::new(variant.type_span.clone())))
                    {
                        token.typed = Some(TypedAstToken::TypedEnumVariant(variant.clone()));
                        token.type_def = Some(TypeDefinition::TypeId(variant.type_id));
                    }
                }
            }
        }
        ty::TyDeclaration::ImplTrait(decl_id) => {
            if let Ok(ty::TyImplTrait {
                trait_name,
                methods,
                implementing_for_type_id,
                type_implementing_for_span,
                ..
            }) = declaration_engine::de_get_impl_trait(decl_id.clone(), &decl_id.span())
            {
                for ident in &trait_name.prefixes {
                    if let Some(mut token) = tokens.get_mut(&to_ident_key(ident)) {
                        token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                    }
                }

                if let Some(mut token) = tokens.get_mut(&to_ident_key(&trait_name.suffix)) {
                    token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                    token.type_def = Some(TypeDefinition::TypeId(implementing_for_type_id));
                }

                if let Some(mut token) =
                    tokens.get_mut(&to_ident_key(&Ident::new(type_implementing_for_span)))
                {
                    token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                    token.type_def = Some(TypeDefinition::TypeId(implementing_for_type_id));
                }

                for method in methods {
                    collect_typed_fn_decl(&method, tokens);
                }
            }
        }
        ty::TyDeclaration::AbiDeclaration(decl_id) => {
            if let Ok(abi_decl) = declaration_engine::de_get_abi(decl_id.clone(), &decl_id.span()) {
                if let Some(mut token) = tokens.get_mut(&to_ident_key(&abi_decl.name)) {
                    token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                }

                for trait_fn in &abi_decl.interface_surface {
                    collect_typed_trait_fn_token(trait_fn, tokens);
                }
            }
        }
        ty::TyDeclaration::GenericTypeForFunctionScope { name, .. } => {
            if let Some(mut token) = tokens.get_mut(&to_ident_key(name)) {
                token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
            }
        }
        ty::TyDeclaration::ErrorRecovery => {}
        ty::TyDeclaration::StorageDeclaration(decl_id) => {
            if let Ok(storage_decl) =
                declaration_engine::de_get_storage(decl_id.clone(), &decl_id.span())
            {
                for field in &storage_decl.fields {
                    if let Some(mut token) = tokens.get_mut(&to_ident_key(&field.name)) {
                        token.typed = Some(TypedAstToken::TypedStorageField(field.clone()));
                        token.type_def = Some(TypeDefinition::TypeId(field.type_id));
                    }

                    if let Some(mut token) =
                        tokens.get_mut(&to_ident_key(&Ident::new(field.type_span.clone())))
                    {
                        token.typed = Some(TypedAstToken::TypedStorageField(field.clone()));
                        token.type_def = Some(TypeDefinition::TypeId(field.type_id));
                    }

                    handle_expression(&field.initializer, tokens);
                }
            }
        }
    }
}

fn handle_expression(expression: &ty::TyExpression, tokens: &TokenMap) {
    match &expression.expression {
        ty::TyExpressionVariant::Literal { .. } => {
            if let Some(mut token) =
                tokens.get_mut(&to_ident_key(&Ident::new(expression.span.clone())))
            {
                token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
            }
        }
        ty::TyExpressionVariant::FunctionApplication {
            call_path,
            contract_call_params,
            arguments,
            function_decl,
            ..
        } => {
            for ident in &call_path.prefixes {
                if let Some(mut token) = tokens.get_mut(&to_ident_key(ident)) {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                    token.type_def = Some(TypeDefinition::TypeId(expression.return_type));
                }
            }

            if let Some(mut token) = tokens.get_mut(&to_ident_key(&call_path.suffix)) {
                token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                token.type_def = Some(TypeDefinition::Ident(function_decl.name.clone()));
            }

            for exp in contract_call_params.values() {
                handle_expression(exp, tokens);
            }

            for (ident, exp) in arguments {
                if let Some(mut token) = tokens.get_mut(&to_ident_key(ident)) {
                    token.typed = Some(TypedAstToken::TypedExpression(exp.clone()));
                }
                handle_expression(exp, tokens);
            }

            for node in &function_decl.body.contents {
                traverse_node(node, tokens);
            }
        }
        ty::TyExpressionVariant::LazyOperator { lhs, rhs, .. } => {
            handle_expression(lhs, tokens);
            handle_expression(rhs, tokens);
        }
        ty::TyExpressionVariant::VariableExpression {
            ref name, ref span, ..
        } => {
            if let Some(mut token) = tokens.get_mut(&to_ident_key(&Ident::new(span.clone()))) {
                token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                token.type_def = Some(TypeDefinition::Ident(name.clone()));
            }
        }
        ty::TyExpressionVariant::Tuple { fields } => {
            for exp in fields {
                handle_expression(exp, tokens);
            }
        }
        ty::TyExpressionVariant::Array { contents } => {
            for exp in contents {
                handle_expression(exp, tokens);
            }
        }
        ty::TyExpressionVariant::ArrayIndex { prefix, index } => {
            handle_expression(prefix, tokens);
            handle_expression(index, tokens);
        }
        ty::TyExpressionVariant::StructExpression { fields, span, .. } => {
            if let Some(mut token) = tokens.get_mut(&to_ident_key(&Ident::new(span.clone()))) {
                token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                token.type_def = Some(TypeDefinition::TypeId(expression.return_type));
            }

            for field in fields {
                if let Some(mut token) = tokens.get_mut(&to_ident_key(&field.name)) {
                    token.typed = Some(TypedAstToken::TypedExpression(field.value.clone()));

                    if let Some(struct_decl) =
                        &struct_declaration_of_type_id(&expression.return_type, tokens)
                    {
                        for decl_field in &struct_decl.fields {
                            if decl_field.name == field.name {
                                token.type_def =
                                    Some(TypeDefinition::Ident(decl_field.name.clone()));
                            }
                        }
                    }
                }
                handle_expression(&field.value, tokens);
            }
        }
        ty::TyExpressionVariant::CodeBlock(code_block) => {
            for node in &code_block.contents {
                traverse_node(node, tokens);
            }
        }
        ty::TyExpressionVariant::FunctionParameter { .. } => {}
        ty::TyExpressionVariant::IfExp {
            condition,
            then,
            r#else,
        } => {
            handle_expression(condition, tokens);
            handle_expression(then, tokens);
            if let Some(r#else) = r#else {
                handle_expression(r#else, tokens);
            }
        }
        ty::TyExpressionVariant::AsmExpression { .. } => {}
        ty::TyExpressionVariant::StructFieldAccess {
            prefix,
            field_to_access,
            field_instantiation_span,
            ..
        } => {
            handle_expression(prefix, tokens);

            if let Some(mut token) =
                tokens.get_mut(&to_ident_key(&Ident::new(field_instantiation_span.clone())))
            {
                token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                token.type_def = Some(TypeDefinition::Ident(field_to_access.name.clone()));
            }
        }
        ty::TyExpressionVariant::TupleElemAccess { prefix, .. } => {
            handle_expression(prefix, tokens);
        }
        ty::TyExpressionVariant::EnumInstantiation {
            variant_name,
            variant_instantiation_span,
            enum_decl,
            enum_instantiation_span,
            contents,
            ..
        } => {
            if let Some(mut token) =
                tokens.get_mut(&to_ident_key(&Ident::new(enum_instantiation_span.clone())))
            {
                token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                token.type_def = Some(TypeDefinition::Ident(enum_decl.name.clone()));
            }

            if let Some(mut token) = tokens.get_mut(&to_ident_key(&Ident::new(
                variant_instantiation_span.clone(),
            ))) {
                token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                token.type_def = Some(TypeDefinition::Ident(variant_name.clone()));
            }

            if let Some(contents) = contents.as_deref() {
                handle_expression(contents, tokens);
            }
        }
        ty::TyExpressionVariant::AbiCast {
            abi_name, address, ..
        } => {
            for ident in &abi_name.prefixes {
                if let Some(mut token) = tokens.get_mut(&to_ident_key(ident)) {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                }
            }

            if let Some(mut token) = tokens.get_mut(&to_ident_key(&abi_name.suffix)) {
                token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
            }

            handle_expression(address, tokens);
        }
        ty::TyExpressionVariant::StorageAccess(storage_access) => {
            for field in &storage_access.fields {
                if let Some(mut token) = tokens.get_mut(&to_ident_key(&field.name)) {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                }
            }
        }
        ty::TyExpressionVariant::IntrinsicFunction(kind) => {
            handle_intrinsic_function(kind, tokens);
        }
        ty::TyExpressionVariant::AbiName { .. } => {}
        ty::TyExpressionVariant::EnumTag { exp } => {
            handle_expression(exp, tokens);
        }
        ty::TyExpressionVariant::UnsafeDowncast { exp, variant } => {
            handle_expression(exp, tokens);
            if let Some(mut token) = tokens.get_mut(&to_ident_key(&variant.name)) {
                token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
            }
        }
        ty::TyExpressionVariant::WhileLoop {
            body, condition, ..
        } => handle_while_loop(body, condition, tokens),
        ty::TyExpressionVariant::Break => (),
        ty::TyExpressionVariant::Continue => (),
        ty::TyExpressionVariant::Reassignment(reassignment) => {
            handle_expression(&reassignment.rhs, tokens);

            if let Some(mut token) = tokens.get_mut(&to_ident_key(&reassignment.lhs_base_name)) {
                token.typed = Some(TypedAstToken::TypedReassignment((**reassignment).clone()));
            }

            for proj_kind in &reassignment.lhs_indices {
                if let ty::ProjectionKind::StructField { name } = proj_kind {
                    if let Some(mut token) = tokens.get_mut(&to_ident_key(name)) {
                        token.typed =
                            Some(TypedAstToken::TypedReassignment((**reassignment).clone()));
                        if let Some(struct_decl) =
                            &struct_declaration_of_type_id(&reassignment.lhs_type, tokens)
                        {
                            for decl_field in &struct_decl.fields {
                                if &decl_field.name == name {
                                    token.type_def =
                                        Some(TypeDefinition::Ident(decl_field.name.clone()));
                                }
                            }
                        }
                    }
                }
            }
        }
        ty::TyExpressionVariant::StorageReassignment(storage_reassignment) => {
            for field in &storage_reassignment.fields {
                if let Some(mut token) = tokens.get_mut(&to_ident_key(&field.name)) {
                    token.typed = Some(TypedAstToken::TypeCheckedStorageReassignDescriptor(
                        field.clone(),
                    ));
                }
            }
            handle_expression(&storage_reassignment.rhs, tokens);
        }
        ty::TyExpressionVariant::Return(exp) => handle_expression(exp, tokens),
    }
}

fn handle_intrinsic_function(
    ty::TyIntrinsicFunctionKind { arguments, .. }: &ty::TyIntrinsicFunctionKind,
    tokens: &TokenMap,
) {
    for arg in arguments {
        handle_expression(arg, tokens);
    }
}

fn handle_while_loop(body: &ty::TyCodeBlock, condition: &ty::TyExpression, tokens: &TokenMap) {
    handle_expression(condition, tokens);
    for node in &body.contents {
        traverse_node(node, tokens);
    }
}

fn collect_typed_trait_fn_token(trait_fn: &ty::TyTraitFn, tokens: &TokenMap) {
    if let Some(mut token) = tokens.get_mut(&to_ident_key(&trait_fn.name)) {
        token.typed = Some(TypedAstToken::TypedTraitFn(trait_fn.clone()));
    }

    for parameter in &trait_fn.parameters {
        collect_typed_fn_param_token(parameter, tokens);
    }

    let return_ident = Ident::new(trait_fn.return_type_span.clone());
    if let Some(mut token) = tokens.get_mut(&to_ident_key(&return_ident)) {
        token.typed = Some(TypedAstToken::TypedTraitFn(trait_fn.clone()));
        token.type_def = Some(TypeDefinition::TypeId(trait_fn.return_type));
    }
}

fn collect_typed_fn_param_token(param: &ty::TyFunctionParameter, tokens: &TokenMap) {
    let typed_token = TypedAstToken::TypedFunctionParameter(param.clone());
    if let Some(mut token) = tokens.get_mut(&to_ident_key(&param.name)) {
        token.typed = Some(typed_token.clone());
        token.type_def = Some(TypeDefinition::TypeId(param.type_id));
    }

    if let Some(mut token) = tokens.get_mut(&to_ident_key(&Ident::new(param.type_span.clone()))) {
        token.typed = Some(typed_token);
        token.type_def = Some(TypeDefinition::TypeId(param.type_id));
    }
}

fn collect_typed_fn_decl(func_decl: &ty::TyFunctionDeclaration, tokens: &TokenMap) {
    if let Some(mut token) = tokens.get_mut(&to_ident_key(&func_decl.name)) {
        token.typed = Some(TypedAstToken::TypedFunctionDeclaration(func_decl.clone()));
    }

    for node in &func_decl.body.contents {
        traverse_node(node, tokens);
    }
    for parameter in &func_decl.parameters {
        collect_typed_fn_param_token(parameter, tokens);
    }

    for type_param in &func_decl.type_parameters {
        if let Some(mut token) = tokens.get_mut(&to_ident_key(&type_param.name_ident)) {
            token.typed = Some(TypedAstToken::TypedFunctionDeclaration(func_decl.clone()));
            token.type_def = Some(TypeDefinition::TypeId(type_param.type_id));
        }
    }

    let return_type_ident = Ident::new(func_decl.return_type_span.clone());
    if let Some(mut token) = tokens.get_mut(&to_ident_key(&return_type_ident)) {
        token.typed = Some(TypedAstToken::TypedFunctionDeclaration(func_decl.clone()));
        token.type_def = Some(TypeDefinition::TypeId(func_decl.return_type));
    }
}

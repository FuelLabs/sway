#![allow(dead_code)]

use crate::{
    core::token::{
        to_ident_key, type_info_to_symbol_kind, AstToken, SymbolKind, Token, TypeDefinition,
        TypedAstToken,
    },
    traverse::{Parse, ParseContext},
};
use dashmap::mapref::one::RefMut;
use sway_core::{
    language::ty::{self, TyEnumVariant, TyStructField},
    AbiName, TraitConstraint, TypeArgument, TypeId, TypeInfo, TypeParameter,
};
use sway_types::{Ident, Span, Spanned};

pub fn traverse_node(ctx: &ParseContext, node: &ty::TyAstNode) {
    match &node.content {
        ty::TyAstNodeContent::Declaration(declaration) => handle_declaration(ctx, declaration),
        ty::TyAstNodeContent::Expression(expression)
        | ty::TyAstNodeContent::ImplicitReturnExpression(expression) => {
            handle_expression(ctx, expression)
        }
        ty::TyAstNodeContent::SideEffect => (),
    };
}

fn handle_declaration(ctx: &ParseContext, declaration: &ty::TyDeclaration) {
    let decl_engine = ctx.engines.de();
    match declaration {
        ty::TyDeclaration::VariableDeclaration(variable) => {
            let typed_token = TypedAstToken::TypedDeclaration(declaration.clone());
            if let Some(mut token) = ctx
                .tokens
                .try_get_mut(&to_ident_key(&variable.name))
                .try_unwrap()
            {
                token.typed = Some(typed_token.clone());
                token.type_def = Some(TypeDefinition::Ident(variable.name.clone()));
            }
            if let Some(type_ascription_span) = &variable.type_ascription_span {
                collect_type_id(
                    ctx,
                    variable.type_ascription,
                    &typed_token,
                    type_ascription_span.clone(),
                );
            }

            handle_expression(ctx, &variable.body);
        }
        ty::TyDeclaration::ConstantDeclaration(decl_id) => {
            if let Ok(const_decl) = decl_engine.get_constant(decl_id.clone(), &decl_id.span()) {
                if let Some(mut token) = ctx
                    .tokens
                    .try_get_mut(&to_ident_key(&const_decl.name))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                    token.type_def = Some(TypeDefinition::Ident(const_decl.name.clone()));
                }

                if let Some(type_ascription_span) = &const_decl.type_ascription_span {
                    if let Some(mut token) = ctx
                        .tokens
                        .try_get_mut(&to_ident_key(&Ident::new(type_ascription_span.clone())))
                        .try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                        token.type_def = Some(TypeDefinition::TypeId(const_decl.return_type));
                    }
                };
                handle_expression(ctx, &const_decl.value);
            }
        }
        ty::TyDeclaration::FunctionDeclaration(decl_id) => {
            if let Ok(func_decl) = decl_engine.get_function(decl_id.clone(), &decl_id.span()) {
                collect_typed_fn_decl(ctx, &func_decl);
            }
        }
        ty::TyDeclaration::TraitDeclaration(decl_id) => {
            if let Ok(trait_decl) = decl_engine.get_trait(decl_id.clone(), &decl_id.span()) {
                if let Some(mut token) = ctx
                    .tokens
                    .try_get_mut(&to_ident_key(&trait_decl.name))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                    token.type_def = Some(TypeDefinition::Ident(trait_decl.name.clone()));
                }

                for trait_fn_decl_id in &trait_decl.interface_surface {
                    if let Ok(trait_fn) =
                        decl_engine.get_trait_fn(trait_fn_decl_id.clone(), &trait_fn_decl_id.span())
                    {
                        collect_typed_trait_fn_token(ctx, &trait_fn);
                    }
                }
            }
        }
        ty::TyDeclaration::StructDeclaration(decl_id) => {
            if let Ok(struct_decl) = decl_engine.get_struct(decl_id.clone(), &declaration.span()) {
                if let Some(mut token) = ctx
                    .tokens
                    .try_get_mut(&to_ident_key(&struct_decl.name))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                    token.type_def = Some(TypeDefinition::Ident(struct_decl.name));
                }

                for field in &struct_decl.fields {
                    collect_ty_struct_field(ctx, field);
                }

                for type_param in &struct_decl.type_parameters {
                    if let Some(mut token) = ctx
                        .tokens
                        .try_get_mut(&to_ident_key(&type_param.name_ident))
                        .try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                        token.type_def = Some(TypeDefinition::TypeId(type_param.type_id));
                    }
                }
            }
        }
        ty::TyDeclaration::EnumDeclaration(decl_id) => {
            if let Ok(enum_decl) = decl_engine.get_enum(decl_id.clone(), &decl_id.span()) {
                if let Some(mut token) = ctx
                    .tokens
                    .try_get_mut(&to_ident_key(&enum_decl.name))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                    token.type_def = Some(TypeDefinition::Ident(enum_decl.name.clone()));
                }

                for type_param in &enum_decl.type_parameters {
                    if let Some(mut token) = ctx
                        .tokens
                        .try_get_mut(&to_ident_key(&type_param.name_ident))
                        .try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                        token.type_def = Some(TypeDefinition::TypeId(type_param.type_id));
                    }
                }

                for variant in &enum_decl.variants {
                    collect_ty_enum_variant(ctx, variant);
                }
            }
        }
        ty::TyDeclaration::ImplTrait(decl_id) => {
            if let Ok(ty::TyImplTrait {
                impl_type_parameters,
                trait_name,
                trait_type_arguments,
                methods,
                implementing_for_type_id,
                type_implementing_for_span,
                ..
            }) = decl_engine.get_impl_trait(decl_id.clone(), &decl_id.span())
            {
                for param in impl_type_parameters {
                    collect_type_id(
                        ctx,
                        param.type_id,
                        &TypedAstToken::TypedParameter(param.clone()),
                        param.name_ident.span().clone(),
                    );
                }

                for ident in &trait_name.prefixes {
                    if let Some(mut token) =
                        ctx.tokens.try_get_mut(&to_ident_key(ident)).try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                    }
                }

                if let Some(mut token) = ctx
                    .tokens
                    .try_get_mut(&to_ident_key(&trait_name.suffix))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                    token.type_def = Some(TypeDefinition::TypeId(implementing_for_type_id));
                }

                for type_arg in trait_type_arguments {
                    collect_type_id(
                        ctx,
                        type_arg.type_id,
                        &TypedAstToken::TypedArgument(type_arg.clone()),
                        type_arg.span().clone(),
                    );
                }

                for method_id in methods {
                    if let Ok(method) = decl_engine.get_function(method_id.clone(), &decl_id.span())
                    {
                        collect_typed_fn_decl(ctx, &method);
                    }
                }

                collect_type_id(
                    ctx,
                    implementing_for_type_id,
                    &TypedAstToken::TypedDeclaration(declaration.clone()),
                    type_implementing_for_span,
                );
            }
        }
        ty::TyDeclaration::AbiDeclaration(decl_id) => {
            if let Ok(abi_decl) = decl_engine.get_abi(decl_id.clone(), &decl_id.span()) {
                if let Some(mut token) = ctx
                    .tokens
                    .try_get_mut(&to_ident_key(&abi_decl.name))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                    token.type_def = Some(TypeDefinition::Ident(abi_decl.name.clone()));
                }

                for trait_fn_decl_id in &abi_decl.interface_surface {
                    if let Ok(trait_fn) =
                        decl_engine.get_trait_fn(trait_fn_decl_id.clone(), &trait_fn_decl_id.span())
                    {
                        collect_typed_trait_fn_token(ctx, &trait_fn);
                    }
                }
            }
        }
        ty::TyDeclaration::GenericTypeForFunctionScope { name, .. } => {
            if let Some(mut token) = ctx.tokens.try_get_mut(&to_ident_key(name)).try_unwrap() {
                token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
            }
        }
        ty::TyDeclaration::ErrorRecovery(_) => {}
        ty::TyDeclaration::StorageDeclaration(decl_id) => {
            if let Ok(storage_decl) = decl_engine.get_storage(decl_id.clone(), &decl_id.span()) {
                for field in &storage_decl.fields {
                    if let Some(mut token) = ctx
                        .tokens
                        .try_get_mut(&to_ident_key(&field.name))
                        .try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::TypedStorageField(field.clone()));
                        token.type_def = Some(TypeDefinition::TypeId(field.type_id));
                    }

                    if let Some(mut token) = ctx
                        .tokens
                        .try_get_mut(&to_ident_key(&Ident::new(field.type_span.clone())))
                        .try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::TypedStorageField(field.clone()));
                        token.type_def = Some(TypeDefinition::TypeId(field.type_id));
                    }

                    handle_expression(ctx, &field.initializer);
                }
            }
        }
    }
}

fn handle_expression(ctx: &ParseContext, expression: &ty::TyExpression) {
    let decl_engine = ctx.engines.de();
    match &expression.expression {
        ty::TyExpressionVariant::Literal { .. } => {
            if let Some(mut token) = ctx
                .tokens
                .try_get_mut(&to_ident_key(&Ident::new(expression.span.clone())))
                .try_unwrap()
            {
                token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
            }
        }
        ty::TyExpressionVariant::FunctionApplication {
            call_path,
            contract_call_params,
            arguments,
            function_decl_id,
            ..
        } => {
            for ident in &call_path.prefixes {
                if let Some(mut token) = ctx.tokens.try_get_mut(&to_ident_key(ident)).try_unwrap() {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                    token.type_def = Some(TypeDefinition::TypeId(expression.return_type));
                }
            }

            if let Some(mut token) = ctx
                .tokens
                .try_get_mut(&to_ident_key(&call_path.suffix))
                .try_unwrap()
            {
                token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                if let Ok(function_decl) =
                    decl_engine.get_function(function_decl_id.clone(), &call_path.span())
                {
                    token.type_def = Some(TypeDefinition::Ident(function_decl.name));
                }
            }

            for exp in contract_call_params.values() {
                handle_expression(ctx, exp);
            }

            for (ident, exp) in arguments {
                if let Some(mut token) = ctx.tokens.try_get_mut(&to_ident_key(ident)).try_unwrap() {
                    token.typed = Some(TypedAstToken::TypedExpression(exp.clone()));
                }
                handle_expression(ctx, exp);
            }

            if let Ok(function_decl) =
                decl_engine.get_function(function_decl_id.clone(), &call_path.span())
            {
                for node in &function_decl.body.contents {
                    traverse_node(ctx, node);
                }
            }
        }
        ty::TyExpressionVariant::LazyOperator { lhs, rhs, .. } => {
            handle_expression(ctx, lhs);
            handle_expression(ctx, rhs);
        }
        ty::TyExpressionVariant::VariableExpression {
            ref name, ref span, ..
        } => {
            if let Some(mut token) = ctx
                .tokens
                .try_get_mut(&to_ident_key(&Ident::new(span.clone())))
                .try_unwrap()
            {
                token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                token.type_def = Some(TypeDefinition::Ident(name.clone()));
            }
        }
        ty::TyExpressionVariant::Tuple { fields } => {
            for exp in fields {
                handle_expression(ctx, exp);
            }
        }
        ty::TyExpressionVariant::Array { contents } => {
            for exp in contents {
                handle_expression(ctx, exp);
            }
        }
        ty::TyExpressionVariant::ArrayIndex { prefix, index } => {
            handle_expression(ctx, prefix);
            handle_expression(ctx, index);
        }
        ty::TyExpressionVariant::StructExpression { fields, span, .. } => {
            if let Some(mut token) = ctx
                .tokens
                .try_get_mut(&to_ident_key(&Ident::new(span.clone())))
                .try_unwrap()
            {
                token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                token.type_def = Some(TypeDefinition::TypeId(expression.return_type));
            }

            for field in fields {
                if let Some(mut token) = ctx
                    .tokens
                    .try_get_mut(&to_ident_key(&field.name))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedExpression(field.value.clone()));

                    if let Some(struct_decl) = ctx
                        .tokens
                        .struct_declaration_of_type_id(ctx.engines, &expression.return_type)
                    {
                        for decl_field in &struct_decl.fields {
                            if decl_field.name == field.name {
                                token.type_def =
                                    Some(TypeDefinition::Ident(decl_field.name.clone()));
                            }
                        }
                    }
                }
                handle_expression(ctx, &field.value);
            }
        }
        ty::TyExpressionVariant::CodeBlock(code_block) => {
            for node in &code_block.contents {
                traverse_node(ctx, node);
            }
        }
        ty::TyExpressionVariant::FunctionParameter { .. } => {}
        ty::TyExpressionVariant::IfExp {
            condition,
            then,
            r#else,
        } => {
            handle_expression(ctx, condition);
            handle_expression(ctx, then);
            if let Some(r#else) = r#else {
                handle_expression(ctx, r#else);
            }
        }
        ty::TyExpressionVariant::AsmExpression { .. } => {}
        ty::TyExpressionVariant::StructFieldAccess {
            prefix,
            field_to_access,
            field_instantiation_span,
            ..
        } => {
            handle_expression(ctx, prefix);

            if let Some(mut token) = ctx
                .tokens
                .try_get_mut(&to_ident_key(&Ident::new(field_instantiation_span.clone())))
                .try_unwrap()
            {
                token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                token.type_def = Some(TypeDefinition::Ident(field_to_access.name.clone()));
            }
        }
        ty::TyExpressionVariant::TupleElemAccess {
            prefix,
            elem_to_access_span,
            ..
        } => {
            handle_expression(ctx, prefix);

            if let Some(mut token) = ctx
                .tokens
                .try_get_mut(&to_ident_key(&Ident::new(elem_to_access_span.clone())))
                .try_unwrap()
            {
                token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
            }
        }
        ty::TyExpressionVariant::EnumInstantiation {
            variant_name,
            variant_instantiation_span,
            enum_decl,
            enum_instantiation_span,
            contents,
            ..
        } => {
            if let Some(mut token) = ctx
                .tokens
                .try_get_mut(&to_ident_key(&Ident::new(enum_instantiation_span.clone())))
                .try_unwrap()
            {
                token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                token.type_def = Some(TypeDefinition::Ident(enum_decl.name.clone()));
            }

            if let Some(mut token) = ctx
                .tokens
                .try_get_mut(&to_ident_key(&Ident::new(
                    variant_instantiation_span.clone(),
                )))
                .try_unwrap()
            {
                token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                token.type_def = Some(TypeDefinition::Ident(variant_name.clone()));
            }

            if let Some(contents) = contents.as_deref() {
                handle_expression(ctx, contents);
            }
        }
        ty::TyExpressionVariant::AbiCast {
            abi_name, address, ..
        } => {
            for ident in &abi_name.prefixes {
                if let Some(mut token) = ctx.tokens.try_get_mut(&to_ident_key(ident)).try_unwrap() {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                }
            }

            if let Some(mut token) = ctx
                .tokens
                .try_get_mut(&to_ident_key(&abi_name.suffix))
                .try_unwrap()
            {
                token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
            }

            handle_expression(ctx, address);
        }
        ty::TyExpressionVariant::StorageAccess(storage_access) => {
            for field in &storage_access.fields {
                if let Some(mut token) = ctx
                    .tokens
                    .try_get_mut(&to_ident_key(&field.name))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                }
            }
        }
        ty::TyExpressionVariant::IntrinsicFunction(kind) => {
            handle_intrinsic_function(ctx, kind);
        }
        ty::TyExpressionVariant::AbiName { .. } => {}
        ty::TyExpressionVariant::EnumTag { exp } => {
            handle_expression(ctx, exp);
        }
        ty::TyExpressionVariant::UnsafeDowncast { exp, variant } => {
            handle_expression(ctx, exp);
            if let Some(mut token) = ctx
                .tokens
                .try_get_mut(&to_ident_key(&variant.name))
                .try_unwrap()
            {
                token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
            }
        }
        ty::TyExpressionVariant::WhileLoop {
            body, condition, ..
        } => handle_while_loop(ctx, body, condition),
        ty::TyExpressionVariant::Break => (),
        ty::TyExpressionVariant::Continue => (),
        ty::TyExpressionVariant::Reassignment(reassignment) => {
            handle_expression(ctx, &reassignment.rhs);

            if let Some(mut token) = ctx
                .tokens
                .try_get_mut(&to_ident_key(&reassignment.lhs_base_name))
                .try_unwrap()
            {
                token.typed = Some(TypedAstToken::TypedReassignment((**reassignment).clone()));
            }

            for proj_kind in &reassignment.lhs_indices {
                if let ty::ProjectionKind::StructField { name } = proj_kind {
                    if let Some(mut token) =
                        ctx.tokens.try_get_mut(&to_ident_key(name)).try_unwrap()
                    {
                        token.typed =
                            Some(TypedAstToken::TypedReassignment((**reassignment).clone()));
                        if let Some(struct_decl) = ctx
                            .tokens
                            .struct_declaration_of_type_id(ctx.engines, &reassignment.lhs_type)
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
                if let Some(mut token) = ctx
                    .tokens
                    .try_get_mut(&to_ident_key(&field.name))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypeCheckedStorageReassignDescriptor(
                        field.clone(),
                    ));
                }
            }
            handle_expression(ctx, &storage_reassignment.rhs);
        }
        ty::TyExpressionVariant::Return(exp) => handle_expression(ctx, exp),
    }
}

fn handle_intrinsic_function(
    ctx: &ParseContext,
    ty::TyIntrinsicFunctionKind { arguments, .. }: &ty::TyIntrinsicFunctionKind,
) {
    for arg in arguments {
        handle_expression(ctx, arg);
    }
}

fn handle_while_loop(ctx: &ParseContext, body: &ty::TyCodeBlock, condition: &ty::TyExpression) {
    handle_expression(ctx, condition);
    for node in &body.contents {
        traverse_node(ctx, node);
    }
}

fn collect_typed_trait_fn_token(ctx: &ParseContext, trait_fn: &ty::TyTraitFn) {
    if let Some(mut token) = ctx
        .tokens
        .try_get_mut(&to_ident_key(&trait_fn.name))
        .try_unwrap()
    {
        token.typed = Some(TypedAstToken::TypedTraitFn(trait_fn.clone()));
        token.type_def = Some(TypeDefinition::Ident(trait_fn.name.clone()));
    }

    for parameter in &trait_fn.parameters {
        collect_typed_fn_param_token(ctx, parameter);
    }

    let return_ident = Ident::new(trait_fn.return_type_span.clone());
    if let Some(mut token) = ctx
        .tokens
        .try_get_mut(&to_ident_key(&return_ident))
        .try_unwrap()
    {
        token.typed = Some(TypedAstToken::TypedTraitFn(trait_fn.clone()));
        token.type_def = Some(TypeDefinition::TypeId(trait_fn.return_type));
    }
}

fn collect_typed_fn_param_token(ctx: &ParseContext, param: &ty::TyFunctionParameter) {
    let typed_token = TypedAstToken::TypedFunctionParameter(param.clone());
    if let Some(mut token) = ctx
        .tokens
        .try_get_mut(&to_ident_key(&param.name))
        .try_unwrap()
    {
        token.typed = Some(typed_token.clone());
        token.type_def = Some(TypeDefinition::TypeId(param.type_id));
    }

    collect_type_id(ctx, param.type_id, &typed_token, param.type_span.clone());
}

fn collect_type_id(
    ctx: &ParseContext,
    type_id: TypeId,
    typed_token: &TypedAstToken,
    type_span: Span,
) {
    let type_engine = ctx.engines.te();
    let type_info = type_engine.get(type_id);
    let symbol_kind = type_info_to_symbol_kind(type_engine, &type_info);
    match &type_info {
        TypeInfo::Array(type_arg, ..) => {
            collect_type_id(
                ctx,
                type_arg.type_id,
                &TypedAstToken::TypedArgument(type_arg.clone()),
                type_arg.span(),
            );
        }
        TypeInfo::Tuple(type_arguments) => {
            for type_arg in type_arguments {
                collect_type_id(
                    ctx,
                    type_arg.type_id,
                    &TypedAstToken::TypedArgument(type_arg.clone()),
                    type_arg.span().clone(),
                );
            }
        }
        TypeInfo::Enum {
            type_parameters,
            variant_types,
            ..
        } => {
            if let Some(token) = ctx
                .tokens
                .try_get_mut(&to_ident_key(&Ident::new(type_span)))
                .try_unwrap()
            {
                assign_type_to_token(token, symbol_kind, typed_token.clone(), type_id);
            }

            for param in type_parameters {
                collect_type_id(
                    ctx,
                    param.type_id,
                    &TypedAstToken::TypedParameter(param.clone()),
                    param.name_ident.span().clone(),
                );
            }

            for variant in variant_types {
                collect_ty_enum_variant(ctx, variant);
            }
        }
        TypeInfo::Struct {
            type_parameters,
            fields,
            ..
        } => {
            if let Some(token) = ctx
                .tokens
                .try_get_mut(&to_ident_key(&Ident::new(type_span)))
                .try_unwrap()
            {
                assign_type_to_token(token, symbol_kind, typed_token.clone(), type_id);
            }

            for param in type_parameters {
                collect_type_id(
                    ctx,
                    param.type_id,
                    &TypedAstToken::TypedParameter(param.clone()),
                    param.name_ident.span().clone(),
                );
            }

            for field in fields {
                collect_ty_struct_field(ctx, field);
            }
        }
        TypeInfo::Custom { type_arguments, .. } => {
            if let Some(token) = ctx
                .tokens
                .try_get_mut(&to_ident_key(&Ident::new(type_span)))
                .try_unwrap()
            {
                assign_type_to_token(token, symbol_kind, typed_token.clone(), type_id);
            }

            if let Some(type_arguments) = type_arguments {
                for type_arg in type_arguments {
                    collect_type_id(
                        ctx,
                        type_arg.type_id,
                        &TypedAstToken::TypedArgument(type_arg.clone()),
                        type_arg.span().clone(),
                    );
                }
            }
        }
        TypeInfo::Storage { fields } => {
            for field in fields {
                collect_ty_struct_field(ctx, field);
            }
        }
        _ => {
            if let Some(token) = ctx
                .tokens
                .try_get_mut(&to_ident_key(&Ident::new(type_span)))
                .try_unwrap()
            {
                assign_type_to_token(token, symbol_kind, typed_token.clone(), type_id);
            }
        }
    }
}

fn collect_typed_fn_decl(ctx: &ParseContext, func_decl: &ty::TyFunctionDeclaration) {
    let typed_token = TypedAstToken::TypedFunctionDeclaration(func_decl.clone());
    if let Some(mut token) = ctx
        .tokens
        .try_get_mut(&to_ident_key(&func_decl.name))
        .try_unwrap()
    {
        token.typed = Some(typed_token.clone());
        token.type_def = Some(TypeDefinition::Ident(func_decl.name.clone()));
    }

    for node in &func_decl.body.contents {
        traverse_node(ctx, node);
    }
    for parameter in &func_decl.parameters {
        collect_typed_fn_param_token(ctx, parameter);
    }

    for type_param in &func_decl.type_parameters {
        collect_type_id(
            ctx,
            type_param.type_id,
            &typed_token,
            type_param.name_ident.span().clone(),
        );
    }

    collect_type_id(
        ctx,
        func_decl.return_type,
        &typed_token,
        func_decl.return_type_span.clone(),
    );
}

fn collect_ty_enum_variant(ctx: &ParseContext, enum_variant: &TyEnumVariant) {
    let typed_token = TypedAstToken::TypedEnumVariant(enum_variant.clone());
    if let Some(mut token) = ctx
        .tokens
        .try_get_mut(&to_ident_key(&enum_variant.name))
        .try_unwrap()
    {
        token.typed = Some(typed_token.clone());
        token.type_def = Some(TypeDefinition::TypeId(enum_variant.type_id));
    }

    collect_type_id(
        ctx,
        enum_variant.type_id,
        &typed_token,
        enum_variant.type_span.clone(),
    );
}

fn collect_ty_struct_field(ctx: &ParseContext, field: &ty::TyStructField) {
    if let Some(mut token) = ctx
        .tokens
        .try_get_mut(&to_ident_key(&field.name))
        .try_unwrap()
    {
        token.typed = Some(TypedAstToken::TypedStructField(field.clone()));
        token.type_def = Some(TypeDefinition::TypeId(field.type_id));
    }

    let typed_token = TypedAstToken::TypedStructField(field.clone());
    collect_type_id(ctx, field.type_id, &typed_token, field.type_span.clone());
}

fn assign_type_to_token(
    mut token: RefMut<(Ident, Span), Token>,
    symbol_kind: SymbolKind,
    typed_token: TypedAstToken,
    type_id: TypeId,
) {
    token.kind = symbol_kind;
    token.typed = Some(typed_token);
    token.type_def = Some(TypeDefinition::TypeId(type_id));
}

impl Parse for TyStructField {
    fn parse(&self, ctx: &ParseContext) {}
}

impl Parse for TypeArgument {
    fn parse(&self, ctx: &ParseContext) {
        let type_info = ctx.engines.te().get(self.type_id);
        type_info.parse(ctx);
    }
}

impl Parse for TypeParameter {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert_typed(
            self.name_ident.clone(),
            TypedAstToken::TypedParameter(self.clone()),
            SymbolKind::TypeParameter,
        );
        let type_info = ctx.engines.te().get(self.type_id);
        type_info.parse(ctx);
        self.trait_constraints.iter().for_each(|trait_constraint| {
            trait_constraint.parse(ctx);
        });
    }
}

impl Parse for TraitConstraint {
    fn parse(&self, ctx: &ParseContext) {
        self.trait_name.prefixes.iter().for_each(|prefix| {
            ctx.tokens.insert_parsed(
                prefix.clone(),
                AstToken::Ident(prefix.clone()),
                SymbolKind::Module,
            );
        });
        ctx.tokens.insert_typed(
            self.trait_name.suffix.clone(),
            TypedAstToken::TraitConstraint(self.clone()),
            SymbolKind::Trait,
        );
        self.type_arguments.iter().for_each(|type_arg| {
            type_arg.parse(ctx);
        });
    }
}

impl Parse for TypeInfo {
    fn parse(&self, ctx: &ParseContext) {
        let mut symbol_kind = type_info_to_symbol_kind(ctx.engines.te(), &self);
        let mut type_def = None;
        let ident = match &self {
            TypeInfo::UnknownGeneric {
                name,
                trait_constraints,
            } => {
                trait_constraints.iter().for_each(|trait_constraint| {
                    trait_constraint.parse(ctx);
                });
                Some(Ident::new(name.span()))
            }
            TypeInfo::Placeholder(type_param) => {
                type_param.parse(ctx);
                None
            }
            TypeInfo::Str(length) => Some(Ident::new(length.span())),
            TypeInfo::Enum {
                name,
                type_parameters,
                variant_types,
            } => {
                type_parameters.iter().for_each(|type_param| {
                    type_param.parse(ctx);
                });
                variant_types.iter().for_each(|enum_variant| {
                    enum_variant.parse(ctx);
                });
                Some(name.clone())
            }
            TypeInfo::Struct {
                name,
                type_parameters,
                fields,
            } => {
                type_parameters.iter().for_each(|type_param| {
                    type_param.parse(ctx);
                });
                fields.iter().for_each(|field| {
                    field.parse(ctx);
                });
                Some(name.clone())
            }
            TypeInfo::Tuple(type_arguments) => {
                type_arguments.iter().for_each(|type_arg| {
                    type_arg.parse(ctx);
                });
                None
            }
            TypeInfo::ContractCaller { abi_name, address } => {
                if let Some(address) = address {
                    address.parse(ctx);
                }
                if let AbiName::Known(abi_name) = abi_name {
                    symbol_kind = SymbolKind::Trait;
                    Some(abi_name.suffix.clone())
                } else {
                    None
                }
            }
            TypeInfo::Custom {
                name,
                type_arguments,
            } => {
                type_def = Some(TypeDefinition::Ident(name.clone()));
                if let Some(type_arguments) = type_arguments {
                    type_arguments.iter().for_each(|type_arg| {
                        type_arg.parse(ctx);
                    });
                }
                Some(name.clone())
            }
            TypeInfo::Array(type_arg, length) => {
                type_arg.parse(ctx);
                symbol_kind = SymbolKind::NumericLiteral;
                Some(Ident::new(length.span()))
            }
            TypeInfo::Storage { fields } => {
                fields.iter().for_each(|field| {
                    field.parse(ctx);
                });
                None
            }
            _ => None,
        };
        if let Some(ident) = ident {
            ctx.tokens.insert_type_info(
                ident,
                TypedAstToken::TypeInfo(self.clone()),
                symbol_kind,
                type_def,
            );
        }
    }
}

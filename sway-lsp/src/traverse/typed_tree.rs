#![allow(dead_code)]

use crate::core::{
    token::{
        to_ident_key, type_info_to_symbol_kind, SymbolKind, Token, TypeDefinition, TypedAstToken,
    },
    token_map::TokenMap,
};
use dashmap::mapref::one::RefMut;
use sway_core::{
    declaration_engine::{self, de_get_function},
    language::ty::{self, TyEnumVariant},
    TypeEngine, TypeId, TypeInfo,
};
use sway_types::{Ident, Span, Spanned};

pub struct TypedTree<'a> {
    type_engine: &'a TypeEngine,
    tokens: &'a TokenMap,
}

impl<'a> TypedTree<'a> {
    pub fn new(type_engine: &'a TypeEngine, tokens: &'a TokenMap) -> Self {
        Self {
            type_engine,
            tokens,
        }
    }

    pub fn traverse_node(&self, node: &ty::TyAstNode) {
        match &node.content {
            ty::TyAstNodeContent::Declaration(declaration) => self.handle_declaration(declaration),
            ty::TyAstNodeContent::Expression(expression)
            | ty::TyAstNodeContent::ImplicitReturnExpression(expression) => {
                self.handle_expression(expression)
            }
            ty::TyAstNodeContent::SideEffect => (),
        };
    }

    fn handle_declaration(&self, declaration: &ty::TyDeclaration) {
        match declaration {
            ty::TyDeclaration::VariableDeclaration(variable) => {
                let typed_token = TypedAstToken::TypedDeclaration(declaration.clone());
                if let Some(mut token) = self.tokens.get_mut(&to_ident_key(&variable.name)) {
                    token.typed = Some(typed_token.clone());
                    token.type_def = Some(TypeDefinition::Ident(variable.name.clone()));
                }
                if let Some(type_ascription_span) = &variable.type_ascription_span {
                    self.collect_type_id(
                        variable.type_ascription,
                        &typed_token,
                        type_ascription_span.clone(),
                    );
                }

                self.handle_expression(&variable.body);
            }
            ty::TyDeclaration::ConstantDeclaration(decl_id) => {
                if let Ok(const_decl) =
                    declaration_engine::de_get_constant(decl_id.clone(), &decl_id.span())
                {
                    if let Some(mut token) = self.tokens.get_mut(&to_ident_key(&const_decl.name)) {
                        token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                        token.type_def = Some(TypeDefinition::Ident(const_decl.name.clone()));
                    }
                    self.handle_expression(&const_decl.value);
                }
            }
            ty::TyDeclaration::FunctionDeclaration(decl_id) => {
                if let Ok(func_decl) =
                    declaration_engine::de_get_function(decl_id.clone(), &decl_id.span())
                {
                    self.collect_typed_fn_decl(&func_decl);
                }
            }
            ty::TyDeclaration::TraitDeclaration(decl_id) => {
                if let Ok(trait_decl) =
                    declaration_engine::de_get_trait(decl_id.clone(), &decl_id.span())
                {
                    if let Some(mut token) = self.tokens.get_mut(&to_ident_key(&trait_decl.name)) {
                        token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                        token.type_def = Some(TypeDefinition::Ident(trait_decl.name.clone()));
                    }

                    for trait_fn_decl_id in &trait_decl.interface_surface {
                        if let Ok(trait_fn) = declaration_engine::de_get_trait_fn(
                            trait_fn_decl_id.clone(),
                            &trait_fn_decl_id.span(),
                        ) {
                            self.collect_typed_trait_fn_token(&trait_fn);
                        }
                    }
                }
            }
            ty::TyDeclaration::StructDeclaration(decl_id) => {
                if let Ok(struct_decl) =
                    declaration_engine::de_get_struct(decl_id.clone(), &declaration.span())
                {
                    if let Some(mut token) = self.tokens.get_mut(&to_ident_key(&struct_decl.name)) {
                        token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                        token.type_def = Some(TypeDefinition::Ident(struct_decl.name));
                    }

                    for field in &struct_decl.fields {
                        self.collect_ty_struct_field(field);
                    }

                    for type_param in &struct_decl.type_parameters {
                        if let Some(mut token) =
                            self.tokens.get_mut(&to_ident_key(&type_param.name_ident))
                        {
                            token.typed =
                                Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                            token.type_def = Some(TypeDefinition::TypeId(type_param.type_id));
                        }
                    }
                }
            }
            ty::TyDeclaration::EnumDeclaration(decl_id) => {
                if let Ok(enum_decl) =
                    declaration_engine::de_get_enum(decl_id.clone(), &decl_id.span())
                {
                    if let Some(mut token) = self.tokens.get_mut(&to_ident_key(&enum_decl.name)) {
                        token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                        token.type_def = Some(TypeDefinition::Ident(enum_decl.name.clone()));
                    }

                    for type_param in &enum_decl.type_parameters {
                        if let Some(mut token) =
                            self.tokens.get_mut(&to_ident_key(&type_param.name_ident))
                        {
                            token.typed =
                                Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                            token.type_def = Some(TypeDefinition::TypeId(type_param.type_id));
                        }
                    }

                    for variant in &enum_decl.variants {
                        self.collect_ty_enum_variant(variant);
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
                }) = declaration_engine::de_get_impl_trait(decl_id.clone(), &decl_id.span())
                {
                    for param in impl_type_parameters {
                        self.collect_type_id(
                            param.type_id,
                            &TypedAstToken::TypedParameter(param.clone()),
                            param.name_ident.span().clone(),
                        );
                    }

                    for ident in &trait_name.prefixes {
                        if let Some(mut token) = self.tokens.get_mut(&to_ident_key(ident)) {
                            token.typed =
                                Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                        }
                    }

                    if let Some(mut token) = self.tokens.get_mut(&to_ident_key(&trait_name.suffix))
                    {
                        token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                        token.type_def = Some(TypeDefinition::TypeId(implementing_for_type_id));
                    }

                    for type_arg in trait_type_arguments {
                        self.collect_type_id(
                            type_arg.type_id,
                            &TypedAstToken::TypedArgument(type_arg.clone()),
                            type_arg.span().clone(),
                        );
                    }

                    for method_id in methods {
                        if let Ok(method) =
                            declaration_engine::de_get_function(method_id.clone(), &decl_id.span())
                        {
                            self.collect_typed_fn_decl(&method);
                        }
                    }

                    self.collect_type_id(
                        implementing_for_type_id,
                        &TypedAstToken::TypedDeclaration(declaration.clone()),
                        type_implementing_for_span,
                    );
                }
            }
            ty::TyDeclaration::AbiDeclaration(decl_id) => {
                if let Ok(abi_decl) =
                    declaration_engine::de_get_abi(decl_id.clone(), &decl_id.span())
                {
                    if let Some(mut token) = self.tokens.get_mut(&to_ident_key(&abi_decl.name)) {
                        token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                        token.type_def = Some(TypeDefinition::Ident(abi_decl.name.clone()));
                    }

                    for trait_fn_decl_id in &abi_decl.interface_surface {
                        if let Ok(trait_fn) = declaration_engine::de_get_trait_fn(
                            trait_fn_decl_id.clone(),
                            &trait_fn_decl_id.span(),
                        ) {
                            self.collect_typed_trait_fn_token(&trait_fn);
                        }
                    }
                }
            }
            ty::TyDeclaration::GenericTypeForFunctionScope { name, .. } => {
                if let Some(mut token) = self.tokens.get_mut(&to_ident_key(name)) {
                    token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                }
            }
            ty::TyDeclaration::ErrorRecovery(_) => {}
            ty::TyDeclaration::StorageDeclaration(decl_id) => {
                if let Ok(storage_decl) =
                    declaration_engine::de_get_storage(decl_id.clone(), &decl_id.span())
                {
                    for field in &storage_decl.fields {
                        if let Some(mut token) = self.tokens.get_mut(&to_ident_key(&field.name)) {
                            token.typed = Some(TypedAstToken::TypedStorageField(field.clone()));
                            token.type_def = Some(TypeDefinition::TypeId(field.type_id));
                        }

                        if let Some(mut token) = self
                            .tokens
                            .get_mut(&to_ident_key(&Ident::new(field.type_span.clone())))
                        {
                            token.typed = Some(TypedAstToken::TypedStorageField(field.clone()));
                            token.type_def = Some(TypeDefinition::TypeId(field.type_id));
                        }

                        self.handle_expression(&field.initializer);
                    }
                }
            }
        }
    }

    fn handle_expression(&self, expression: &ty::TyExpression) {
        match &expression.expression {
            ty::TyExpressionVariant::Literal { .. } => {
                if let Some(mut token) = self
                    .tokens
                    .get_mut(&to_ident_key(&Ident::new(expression.span.clone())))
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
                    if let Some(mut token) = self.tokens.get_mut(&to_ident_key(ident)) {
                        token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                        token.type_def = Some(TypeDefinition::TypeId(expression.return_type));
                    }
                }

                if let Some(mut token) = self.tokens.get_mut(&to_ident_key(&call_path.suffix)) {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                    if let Ok(function_decl) =
                        de_get_function(function_decl_id.clone(), &call_path.span())
                    {
                        token.type_def = Some(TypeDefinition::Ident(function_decl.name));
                    }
                }

                for exp in contract_call_params.values() {
                    self.handle_expression(exp);
                }

                for (ident, exp) in arguments {
                    if let Some(mut token) = self.tokens.get_mut(&to_ident_key(ident)) {
                        token.typed = Some(TypedAstToken::TypedExpression(exp.clone()));
                    }
                    self.handle_expression(exp);
                }

                if let Ok(function_decl) =
                    de_get_function(function_decl_id.clone(), &call_path.span())
                {
                    for node in &function_decl.body.contents {
                        self.traverse_node(node);
                    }
                }
            }
            ty::TyExpressionVariant::LazyOperator { lhs, rhs, .. } => {
                self.handle_expression(lhs);
                self.handle_expression(rhs);
            }
            ty::TyExpressionVariant::VariableExpression {
                ref name, ref span, ..
            } => {
                if let Some(mut token) = self
                    .tokens
                    .get_mut(&to_ident_key(&Ident::new(span.clone())))
                {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                    token.type_def = Some(TypeDefinition::Ident(name.clone()));
                }
            }
            ty::TyExpressionVariant::Tuple { fields } => {
                for exp in fields {
                    self.handle_expression(exp);
                }
            }
            ty::TyExpressionVariant::Array { contents } => {
                for exp in contents {
                    self.handle_expression(exp);
                }
            }
            ty::TyExpressionVariant::ArrayIndex { prefix, index } => {
                self.handle_expression(prefix);
                self.handle_expression(index);
            }
            ty::TyExpressionVariant::StructExpression { fields, span, .. } => {
                if let Some(mut token) = self
                    .tokens
                    .get_mut(&to_ident_key(&Ident::new(span.clone())))
                {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                    token.type_def = Some(TypeDefinition::TypeId(expression.return_type));
                }

                for field in fields {
                    if let Some(mut token) = self.tokens.get_mut(&to_ident_key(&field.name)) {
                        token.typed = Some(TypedAstToken::TypedExpression(field.value.clone()));

                        if let Some(struct_decl) = &self.tokens.struct_declaration_of_type_id(
                            self.type_engine,
                            &expression.return_type,
                        ) {
                            for decl_field in &struct_decl.fields {
                                if decl_field.name == field.name {
                                    token.type_def =
                                        Some(TypeDefinition::Ident(decl_field.name.clone()));
                                }
                            }
                        }
                    }
                    self.handle_expression(&field.value);
                }
            }
            ty::TyExpressionVariant::CodeBlock(code_block) => {
                for node in &code_block.contents {
                    self.traverse_node(node);
                }
            }
            ty::TyExpressionVariant::FunctionParameter { .. } => {}
            ty::TyExpressionVariant::IfExp {
                condition,
                then,
                r#else,
            } => {
                self.handle_expression(condition);
                self.handle_expression(then);
                if let Some(r#else) = r#else {
                    self.handle_expression(r#else);
                }
            }
            ty::TyExpressionVariant::AsmExpression { .. } => {}
            ty::TyExpressionVariant::StructFieldAccess {
                prefix,
                field_to_access,
                field_instantiation_span,
                ..
            } => {
                self.handle_expression(prefix);

                if let Some(mut token) = self
                    .tokens
                    .get_mut(&to_ident_key(&Ident::new(field_instantiation_span.clone())))
                {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                    token.type_def = Some(TypeDefinition::Ident(field_to_access.name.clone()));
                }
            }
            ty::TyExpressionVariant::TupleElemAccess { prefix, .. } => {
                self.handle_expression(prefix);
            }
            ty::TyExpressionVariant::EnumInstantiation {
                variant_name,
                variant_instantiation_span,
                enum_decl,
                enum_instantiation_span,
                contents,
                ..
            } => {
                if let Some(mut token) = self
                    .tokens
                    .get_mut(&to_ident_key(&Ident::new(enum_instantiation_span.clone())))
                {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                    token.type_def = Some(TypeDefinition::Ident(enum_decl.name.clone()));
                }

                if let Some(mut token) = self.tokens.get_mut(&to_ident_key(&Ident::new(
                    variant_instantiation_span.clone(),
                ))) {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                    token.type_def = Some(TypeDefinition::Ident(variant_name.clone()));
                }

                if let Some(contents) = contents.as_deref() {
                    self.handle_expression(contents);
                }
            }
            ty::TyExpressionVariant::AbiCast {
                abi_name, address, ..
            } => {
                for ident in &abi_name.prefixes {
                    if let Some(mut token) = self.tokens.get_mut(&to_ident_key(ident)) {
                        token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                    }
                }

                if let Some(mut token) = self.tokens.get_mut(&to_ident_key(&abi_name.suffix)) {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                }

                self.handle_expression(address);
            }
            ty::TyExpressionVariant::StorageAccess(storage_access) => {
                for field in &storage_access.fields {
                    if let Some(mut token) = self.tokens.get_mut(&to_ident_key(&field.name)) {
                        token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                    }
                }
            }
            ty::TyExpressionVariant::IntrinsicFunction(kind) => {
                self.handle_intrinsic_function(kind);
            }
            ty::TyExpressionVariant::AbiName { .. } => {}
            ty::TyExpressionVariant::EnumTag { exp } => {
                self.handle_expression(exp);
            }
            ty::TyExpressionVariant::UnsafeDowncast { exp, variant } => {
                self.handle_expression(exp);
                if let Some(mut token) = self.tokens.get_mut(&to_ident_key(&variant.name)) {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                }
            }
            ty::TyExpressionVariant::WhileLoop {
                body, condition, ..
            } => self.handle_while_loop(body, condition),
            ty::TyExpressionVariant::Break => (),
            ty::TyExpressionVariant::Continue => (),
            ty::TyExpressionVariant::Reassignment(reassignment) => {
                self.handle_expression(&reassignment.rhs);

                if let Some(mut token) = self
                    .tokens
                    .get_mut(&to_ident_key(&reassignment.lhs_base_name))
                {
                    token.typed = Some(TypedAstToken::TypedReassignment((**reassignment).clone()));
                }

                for proj_kind in &reassignment.lhs_indices {
                    if let ty::ProjectionKind::StructField { name } = proj_kind {
                        if let Some(mut token) = self.tokens.get_mut(&to_ident_key(name)) {
                            token.typed =
                                Some(TypedAstToken::TypedReassignment((**reassignment).clone()));
                            if let Some(struct_decl) = &self.tokens.struct_declaration_of_type_id(
                                self.type_engine,
                                &reassignment.lhs_type,
                            ) {
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
                    if let Some(mut token) = self.tokens.get_mut(&to_ident_key(&field.name)) {
                        token.typed = Some(TypedAstToken::TypeCheckedStorageReassignDescriptor(
                            field.clone(),
                        ));
                    }
                }
                self.handle_expression(&storage_reassignment.rhs);
            }
            ty::TyExpressionVariant::Return(exp) => self.handle_expression(exp),
        }
    }

    fn handle_intrinsic_function(
        &self,
        ty::TyIntrinsicFunctionKind { arguments, .. }: &ty::TyIntrinsicFunctionKind,
    ) {
        for arg in arguments {
            self.handle_expression(arg);
        }
    }

    fn handle_while_loop(&self, body: &ty::TyCodeBlock, condition: &ty::TyExpression) {
        self.handle_expression(condition);
        for node in &body.contents {
            self.traverse_node(node);
        }
    }

    fn collect_typed_trait_fn_token(&self, trait_fn: &ty::TyTraitFn) {
        if let Some(mut token) = self.tokens.get_mut(&to_ident_key(&trait_fn.name)) {
            token.typed = Some(TypedAstToken::TypedTraitFn(trait_fn.clone()));
            token.type_def = Some(TypeDefinition::Ident(trait_fn.name.clone()));
        }

        for parameter in &trait_fn.parameters {
            self.collect_typed_fn_param_token(parameter);
        }

        let return_ident = Ident::new(trait_fn.return_type_span.clone());
        if let Some(mut token) = self.tokens.get_mut(&to_ident_key(&return_ident)) {
            token.typed = Some(TypedAstToken::TypedTraitFn(trait_fn.clone()));
            token.type_def = Some(TypeDefinition::TypeId(trait_fn.return_type));
        }
    }

    fn collect_typed_fn_param_token(&self, param: &ty::TyFunctionParameter) {
        let typed_token = TypedAstToken::TypedFunctionParameter(param.clone());
        if let Some(mut token) = self.tokens.get_mut(&to_ident_key(&param.name)) {
            token.typed = Some(typed_token.clone());
            token.type_def = Some(TypeDefinition::TypeId(param.type_id));
        }

        self.collect_type_id(param.type_id, &typed_token, param.type_span.clone());
    }

    fn collect_type_id(&self, type_id: TypeId, typed_token: &TypedAstToken, type_span: Span) {
        let type_info = self.type_engine.look_up_type_id(type_id);
        let symbol_kind = type_info_to_symbol_kind(self.type_engine, &type_info);
        match &type_info {
            TypeInfo::Array(type_arg, ..) => {
                self.collect_type_id(
                    type_arg.type_id,
                    &TypedAstToken::TypedArgument(type_arg.clone()),
                    type_arg.span(),
                );
            }
            TypeInfo::Tuple(type_arguments) => {
                for type_arg in type_arguments {
                    self.collect_type_id(
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
                if let Some(token) = self.tokens.get_mut(&to_ident_key(&Ident::new(type_span))) {
                    assign_type_to_token(token, symbol_kind, typed_token.clone(), type_id);
                }

                for param in type_parameters {
                    self.collect_type_id(
                        param.type_id,
                        &TypedAstToken::TypedParameter(param.clone()),
                        param.name_ident.span().clone(),
                    );
                }

                for variant in variant_types {
                    self.collect_ty_enum_variant(variant);
                }
            }
            TypeInfo::Struct {
                type_parameters,
                fields,
                ..
            } => {
                if let Some(token) = self.tokens.get_mut(&to_ident_key(&Ident::new(type_span))) {
                    assign_type_to_token(token, symbol_kind, typed_token.clone(), type_id);
                }

                for param in type_parameters {
                    self.collect_type_id(
                        param.type_id,
                        &TypedAstToken::TypedParameter(param.clone()),
                        param.name_ident.span().clone(),
                    );
                }

                for field in fields {
                    self.collect_ty_struct_field(field);
                }
            }
            TypeInfo::Custom { type_arguments, .. } => {
                if let Some(token) = self.tokens.get_mut(&to_ident_key(&Ident::new(type_span))) {
                    assign_type_to_token(token, symbol_kind, typed_token.clone(), type_id);
                }

                if let Some(type_arguments) = type_arguments {
                    for type_arg in type_arguments {
                        self.collect_type_id(
                            type_arg.type_id,
                            &TypedAstToken::TypedArgument(type_arg.clone()),
                            type_arg.span().clone(),
                        );
                    }
                }
            }
            TypeInfo::Storage { fields } => {
                for field in fields {
                    self.collect_ty_struct_field(field);
                }
            }
            _ => {
                if let Some(token) = self.tokens.get_mut(&to_ident_key(&Ident::new(type_span))) {
                    assign_type_to_token(token, symbol_kind, typed_token.clone(), type_id);
                }
            }
        }
    }

    fn collect_typed_fn_decl(&self, func_decl: &ty::TyFunctionDeclaration) {
        let typed_token = TypedAstToken::TypedFunctionDeclaration(func_decl.clone());
        if let Some(mut token) = self.tokens.get_mut(&to_ident_key(&func_decl.name)) {
            token.typed = Some(typed_token.clone());
            token.type_def = Some(TypeDefinition::Ident(func_decl.name.clone()));
        }

        for node in &func_decl.body.contents {
            self.traverse_node(node);
        }
        for parameter in &func_decl.parameters {
            self.collect_typed_fn_param_token(parameter);
        }

        for type_param in &func_decl.type_parameters {
            self.collect_type_id(
                type_param.type_id,
                &typed_token,
                type_param.name_ident.span().clone(),
            );
        }

        self.collect_type_id(
            func_decl.return_type,
            &typed_token,
            func_decl.return_type_span.clone(),
        );
    }

    fn collect_ty_enum_variant(&self, enum_variant: &TyEnumVariant) {
        let typed_token = TypedAstToken::TypedEnumVariant(enum_variant.clone());
        if let Some(mut token) = self.tokens.get_mut(&to_ident_key(&enum_variant.name)) {
            token.typed = Some(typed_token.clone());
            token.type_def = Some(TypeDefinition::TypeId(enum_variant.type_id));
        }

        self.collect_type_id(
            enum_variant.type_id,
            &typed_token,
            enum_variant.type_span.clone(),
        );
    }

    fn collect_ty_struct_field(&self, field: &ty::TyStructField) {
        if let Some(mut token) = self.tokens.get_mut(&to_ident_key(&field.name)) {
            token.typed = Some(TypedAstToken::TypedStructField(field.clone()));
            token.type_def = Some(TypeDefinition::TypeId(field.type_id));
        }

        let typed_token = TypedAstToken::TypedStructField(field.clone());
        self.collect_type_id(field.type_id, &typed_token, field.type_span.clone());
    }
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

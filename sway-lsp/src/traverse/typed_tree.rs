#![allow(dead_code)]

use crate::core::{
    token::{
        to_ident_key, type_info_to_symbol_kind, SymbolKind, Token, TypeDefinition, TypedAstToken,
    },
    token_map::TokenMap,
};
use dashmap::mapref::one::RefMut;
use sway_core::{
    decl_engine::DeclId,
    language::{
        parsed::Supertrait,
        ty::{self, GetDeclIdent, TyEnumVariant, TyModule, TyProgram, TyProgramKind, TySubmodule},
        CallPath,
    },
    namespace,
    type_system::TypeArgument,
    Engines, TypeId, TypeInfo,
};
use sway_types::{Ident, Span, SpanTree, Spanned};

pub struct TypedTree<'a> {
    engines: Engines<'a>,
    tokens: &'a TokenMap,
}

impl<'a> TypedTree<'a> {
    pub fn new(engines: Engines<'a>, tokens: &'a TokenMap) -> Self {
        Self { engines, tokens }
    }

    pub fn traverse_node(&self, node: &ty::TyAstNode, namespace: &namespace::Module) {
        match &node.content {
            ty::TyAstNodeContent::Declaration(declaration) => {
                self.handle_declaration(declaration, namespace)
            }
            ty::TyAstNodeContent::Expression(expression)
            | ty::TyAstNodeContent::ImplicitReturnExpression(expression) => {
                self.handle_expression(expression, namespace)
            }
            ty::TyAstNodeContent::SideEffect => (),
        };
    }

    pub fn collect_module_spans(&self, typed_program: &TyProgram) {
        self.collect_program_kind(&typed_program.kind);
        self.collect_module(&typed_program.root);
    }

    pub fn collect_module(&self, typed_module: &TyModule) {
        for (
            _,
            TySubmodule {
                library_name,
                module,
                dependency_path_span,
            },
        ) in &typed_module.submodules
        {
            if let Some(mut token) = self
                .tokens
                .try_get_mut(&to_ident_key(&Ident::new(dependency_path_span.clone())))
                .try_unwrap()
            {
                token.typed = Some(TypedAstToken::TypedIncludeStatement);
                token.type_def = Some(TypeDefinition::Ident(library_name.clone()));
            }

            if let Some(mut token) = self
                .tokens
                .try_get_mut(&to_ident_key(library_name))
                .try_unwrap()
            {
                token.typed = Some(TypedAstToken::TypedLibraryName(library_name.clone()));
                token.type_def = Some(TypeDefinition::Ident(library_name.clone()));
            }

            self.collect_module(module);
        }
    }

    fn collect_program_kind(&self, program_kind: &TyProgramKind) {
        use TyProgramKind::*;
        match program_kind {
            Library { name } => {
                if let Some(mut token) = self.tokens.try_get_mut(&to_ident_key(name)).try_unwrap() {
                    token.typed = Some(TypedAstToken::TypedProgramKind(program_kind.clone()));
                    token.type_def = Some(TypeDefinition::Ident(name.clone()));
                }
            }
            Script { .. } | Contract { .. } | Predicate { .. } => {}
        }
    }

    fn handle_declaration(&self, declaration: &ty::TyDeclaration, namespace: &namespace::Module) {
        let decl_engine = self.engines.de();
        match declaration {
            ty::TyDeclaration::VariableDeclaration(variable) => {
                let typed_token = TypedAstToken::TypedDeclaration(declaration.clone());
                if let Some(mut token) = self
                    .tokens
                    .try_get_mut(&to_ident_key(&variable.name))
                    .try_unwrap()
                {
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

                self.handle_expression(&variable.body, namespace);
            }
            ty::TyDeclaration::ConstantDeclaration(decl_id) => {
                if let Ok(const_decl) = decl_engine.get_constant(decl_id.clone(), &decl_id.span()) {
                    if let Some(mut token) = self
                        .tokens
                        .try_get_mut(&to_ident_key(&const_decl.name))
                        .try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                        token.type_def = Some(TypeDefinition::Ident(const_decl.name.clone()));
                    }

                    if let Some(type_ascription_span) = &const_decl.type_ascription_span {
                        if let Some(mut token) = self
                            .tokens
                            .try_get_mut(&to_ident_key(&Ident::new(type_ascription_span.clone())))
                            .try_unwrap()
                        {
                            token.typed =
                                Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                            token.type_def = Some(TypeDefinition::TypeId(const_decl.return_type));
                        }
                    };
                    self.handle_expression(&const_decl.value, namespace);
                }
            }
            ty::TyDeclaration::FunctionDeclaration(decl_id) => {
                if let Ok(func_decl) = decl_engine.get_function(decl_id.clone(), &decl_id.span()) {
                    self.collect_typed_fn_decl(&func_decl, namespace);
                }
            }
            ty::TyDeclaration::TraitDeclaration(decl_id) => {
                if let Ok(trait_decl) = decl_engine.get_trait(decl_id.clone(), &decl_id.span()) {
                    if let Some(mut token) = self
                        .tokens
                        .try_get_mut(&to_ident_key(&trait_decl.name))
                        .try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                        token.type_def = Some(TypeDefinition::Ident(trait_decl.name.clone()));
                    }

                    for trait_fn_decl_id in &trait_decl.interface_surface {
                        if let Ok(trait_fn) = decl_engine
                            .get_trait_fn(trait_fn_decl_id.clone(), &trait_fn_decl_id.span())
                        {
                            self.collect_typed_trait_fn_token(&trait_fn);
                        }
                    }
                    for supertrait in trait_decl.supertraits {
                        self.collect_supertrait(&supertrait);
                    }
                }
            }
            ty::TyDeclaration::StructDeclaration(decl_id) => {
                if let Ok(struct_decl) =
                    decl_engine.get_struct(decl_id.clone(), &declaration.span())
                {
                    if let Some(mut token) = self
                        .tokens
                        .try_get_mut(&to_ident_key(&struct_decl.call_path.suffix))
                        .try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                        token.type_def = Some(TypeDefinition::Ident(struct_decl.call_path.suffix));
                    }

                    for field in &struct_decl.fields {
                        self.collect_ty_struct_field(field);
                    }

                    for type_param in &struct_decl.type_parameters {
                        if let Some(mut token) = self
                            .tokens
                            .try_get_mut(&to_ident_key(&type_param.name_ident))
                            .try_unwrap()
                        {
                            token.typed =
                                Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                            token.type_def = Some(TypeDefinition::TypeId(type_param.type_id));
                        }
                    }
                }
            }
            ty::TyDeclaration::EnumDeclaration(decl_id) => {
                if let Ok(enum_decl) = decl_engine.get_enum(decl_id.clone(), &decl_id.span()) {
                    if let Some(mut token) = self
                        .tokens
                        .try_get_mut(&to_ident_key(&enum_decl.call_path.suffix))
                        .try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                        token.type_def =
                            Some(TypeDefinition::Ident(enum_decl.call_path.suffix.clone()));
                    }

                    for type_param in &enum_decl.type_parameters {
                        if let Some(mut token) = self
                            .tokens
                            .try_get_mut(&to_ident_key(&type_param.name_ident))
                            .try_unwrap()
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
                    trait_decl_id,
                    methods,
                    implementing_for_type_id,
                    type_implementing_for_span,
                    ..
                }) = decl_engine.get_impl_trait(decl_id.clone(), &decl_id.span())
                {
                    for param in impl_type_parameters {
                        self.collect_type_id(
                            param.type_id,
                            &TypedAstToken::TypedParameter(param.clone()),
                            param.name_ident.span().clone(),
                        );
                    }

                    for ident in &trait_name.prefixes {
                        if let Some(mut token) =
                            self.tokens.try_get_mut(&to_ident_key(ident)).try_unwrap()
                        {
                            token.typed =
                                Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                        }
                    }

                    if let Some(mut token) = self
                        .tokens
                        .try_get_mut(&to_ident_key(&trait_name.suffix))
                        .try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                        token.type_def = if let Some(decl_id) = &trait_decl_id {
                            if let Ok(trait_decl) =
                                decl_engine.get_trait(decl_id.clone(), &decl_id.span())
                            {
                                Some(TypeDefinition::Ident(trait_decl.name))
                            } else {
                                Some(TypeDefinition::TypeId(implementing_for_type_id))
                            }
                        } else {
                            Some(TypeDefinition::TypeId(implementing_for_type_id))
                        };
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
                            decl_engine.get_function(method_id.clone(), &decl_id.span())
                        {
                            self.collect_typed_fn_decl(&method, namespace);
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
                if let Ok(abi_decl) = decl_engine.get_abi(decl_id.clone(), &decl_id.span()) {
                    if let Some(mut token) = self
                        .tokens
                        .try_get_mut(&to_ident_key(&abi_decl.name))
                        .try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                        token.type_def = Some(TypeDefinition::Ident(abi_decl.name.clone()));
                    }

                    for trait_fn_decl_id in &abi_decl.interface_surface {
                        if let Ok(trait_fn) = decl_engine
                            .get_trait_fn(trait_fn_decl_id.clone(), &trait_fn_decl_id.span())
                        {
                            self.collect_typed_trait_fn_token(&trait_fn);
                        }
                    }
                }
            }
            ty::TyDeclaration::GenericTypeForFunctionScope { name, .. } => {
                if let Some(mut token) = self.tokens.try_get_mut(&to_ident_key(name)).try_unwrap() {
                    token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                }
            }
            ty::TyDeclaration::ErrorRecovery(_) => {}
            ty::TyDeclaration::StorageDeclaration(decl_id) => {
                if let Ok(storage_decl) = decl_engine.get_storage(decl_id.clone(), &decl_id.span())
                {
                    for field in &storage_decl.fields {
                        if let Some(mut token) = self
                            .tokens
                            .try_get_mut(&to_ident_key(&field.name))
                            .try_unwrap()
                        {
                            token.typed = Some(TypedAstToken::TypedStorageField(field.clone()));
                            token.type_def = Some(TypeDefinition::TypeId(field.type_id));
                        }

                        if let Some(mut token) = self
                            .tokens
                            .try_get_mut(&to_ident_key(&Ident::new(field.type_span.clone())))
                            .try_unwrap()
                        {
                            token.typed = Some(TypedAstToken::TypedStorageField(field.clone()));
                            token.type_def = Some(TypeDefinition::TypeId(field.type_id));
                        }

                        self.handle_expression(&field.initializer, namespace);
                    }
                }
            }
        }
    }

    fn handle_expression(&self, expression: &ty::TyExpression, namespace: &namespace::Module) {
        let decl_engine = self.engines.de();
        match &expression.expression {
            ty::TyExpressionVariant::Literal { .. } => {
                if let Some(mut token) = self
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
                type_binding,
                ..
            } => {
                if let Some(type_binding) = type_binding {
                    for type_arg in &type_binding.type_arguments {
                        self.collect_type_argument(type_arg);
                    }
                }

                self.collect_call_path_prefixes(
                    call_path,
                    expression,
                    Some(function_decl_id),
                    namespace,
                );

                if let Some(mut token) = self
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
                    self.handle_expression(exp, namespace);
                }

                for (ident, exp) in arguments {
                    if let Some(mut token) =
                        self.tokens.try_get_mut(&to_ident_key(ident)).try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::TypedExpression(exp.clone()));
                    }
                    self.handle_expression(exp, namespace);
                }

                if let Ok(function_decl) =
                    decl_engine.get_function(function_decl_id.clone(), &call_path.span())
                {
                    for node in &function_decl.body.contents {
                        self.traverse_node(node, namespace);
                    }
                }
            }
            ty::TyExpressionVariant::LazyOperator { lhs, rhs, .. } => {
                self.handle_expression(lhs, namespace);
                self.handle_expression(rhs, namespace);
            }
            ty::TyExpressionVariant::VariableExpression {
                ref name,
                ref span,
                ref call_path,
                ..
            } => {
                if let Some(call_path) = call_path {
                    self.collect_call_path_prefixes(call_path, expression, None, namespace);
                }

                let span = if let Some(call_path) = call_path {
                    call_path.suffix.span()
                } else {
                    span.clone()
                };

                if let Some(mut token) = self
                    .tokens
                    .try_get_mut(&to_ident_key(&Ident::new(span)))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                    token.type_def = Some(TypeDefinition::Ident(name.clone()));
                }
            }
            ty::TyExpressionVariant::Tuple { fields } => {
                for exp in fields {
                    self.handle_expression(exp, namespace);
                }
            }
            ty::TyExpressionVariant::Array { contents } => {
                for exp in contents {
                    self.handle_expression(exp, namespace);
                }
            }
            ty::TyExpressionVariant::ArrayIndex { prefix, index } => {
                self.handle_expression(prefix, namespace);
                self.handle_expression(index, namespace);
            }
            ty::TyExpressionVariant::StructExpression {
                fields,
                span,
                call_path_binding,
                ..
            } => {
                if let Some(mut token) = self
                    .tokens
                    .try_get_mut(&to_ident_key(&Ident::new(span.clone())))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                    token.type_def = Some(TypeDefinition::TypeId(expression.return_type));
                }

                for type_arg in &call_path_binding.type_arguments {
                    self.collect_type_argument(type_arg);
                }

                self.collect_call_path_prefixes(
                    &call_path_binding.inner,
                    expression,
                    None,
                    namespace,
                );

                for field in fields {
                    if let Some(mut token) = self
                        .tokens
                        .try_get_mut(&to_ident_key(&field.name))
                        .try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::TypedExpression(field.value.clone()));

                        if let Some(struct_decl) = &self
                            .tokens
                            .struct_declaration_of_type_id(self.engines, &expression.return_type)
                        {
                            for decl_field in &struct_decl.fields {
                                if decl_field.name == field.name {
                                    token.type_def =
                                        Some(TypeDefinition::Ident(decl_field.name.clone()));
                                }
                            }
                        }
                    }
                    self.handle_expression(&field.value, namespace);
                }
            }
            ty::TyExpressionVariant::CodeBlock(code_block) => {
                for node in &code_block.contents {
                    self.traverse_node(node, namespace);
                }
            }
            ty::TyExpressionVariant::FunctionParameter { .. } => {}
            ty::TyExpressionVariant::IfExp {
                condition,
                then,
                r#else,
            } => {
                self.handle_expression(condition, namespace);
                self.handle_expression(then, namespace);
                if let Some(r#else) = r#else {
                    self.handle_expression(r#else, namespace);
                }
            }
            ty::TyExpressionVariant::AsmExpression { .. } => {}
            ty::TyExpressionVariant::StructFieldAccess {
                prefix,
                field_to_access,
                field_instantiation_span,
                ..
            } => {
                self.handle_expression(prefix, namespace);

                if let Some(mut token) = self
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
                self.handle_expression(prefix, namespace);

                if let Some(mut token) = self
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
                call_path_binding,
                ..
            } => {
                if let Some(mut token) = self
                    .tokens
                    .try_get_mut(&to_ident_key(&Ident::new(enum_instantiation_span.clone())))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                    token.type_def =
                        Some(TypeDefinition::Ident(enum_decl.call_path.suffix.clone()));
                }

                for type_arg in &call_path_binding.type_arguments {
                    self.collect_type_argument(type_arg);
                }

                self.collect_call_path_prefixes(
                    &call_path_binding.inner,
                    expression,
                    None,
                    namespace,
                );

                if let Some(mut token) = self
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
                    self.handle_expression(contents, namespace);
                }
            }
            ty::TyExpressionVariant::AbiCast {
                abi_name, address, ..
            } => {
                for ident in &abi_name.prefixes {
                    if let Some(mut token) =
                        self.tokens.try_get_mut(&to_ident_key(ident)).try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                    }
                }

                if let Some(mut token) = self
                    .tokens
                    .try_get_mut(&to_ident_key(&abi_name.suffix))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                }

                self.handle_expression(address, namespace);
            }
            ty::TyExpressionVariant::StorageAccess(storage_access) => {
                for field in &storage_access.fields {
                    if let Some(mut token) = self
                        .tokens
                        .try_get_mut(&to_ident_key(&field.name))
                        .try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                    }
                }
            }
            ty::TyExpressionVariant::IntrinsicFunction(kind) => {
                self.handle_intrinsic_function(kind, namespace);
            }
            ty::TyExpressionVariant::AbiName { .. } => {}
            ty::TyExpressionVariant::EnumTag { exp } => {
                self.handle_expression(exp, namespace);
            }
            ty::TyExpressionVariant::UnsafeDowncast { exp, variant } => {
                self.handle_expression(exp, namespace);
                if let Some(mut token) = self
                    .tokens
                    .try_get_mut(&to_ident_key(&variant.name))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                }
            }
            ty::TyExpressionVariant::WhileLoop {
                body, condition, ..
            } => self.handle_while_loop(body, condition, namespace),
            ty::TyExpressionVariant::Break => (),
            ty::TyExpressionVariant::Continue => (),
            ty::TyExpressionVariant::Reassignment(reassignment) => {
                self.handle_expression(&reassignment.rhs, namespace);

                if let Some(mut token) = self
                    .tokens
                    .try_get_mut(&to_ident_key(&reassignment.lhs_base_name))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedReassignment((**reassignment).clone()));
                }

                for proj_kind in &reassignment.lhs_indices {
                    if let ty::ProjectionKind::StructField { name } = proj_kind {
                        if let Some(mut token) =
                            self.tokens.try_get_mut(&to_ident_key(name)).try_unwrap()
                        {
                            token.typed =
                                Some(TypedAstToken::TypedReassignment((**reassignment).clone()));
                            if let Some(struct_decl) = &self
                                .tokens
                                .struct_declaration_of_type_id(self.engines, &reassignment.lhs_type)
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
                    if let Some(mut token) = self
                        .tokens
                        .try_get_mut(&to_ident_key(&field.name))
                        .try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::TypeCheckedStorageReassignDescriptor(
                            field.clone(),
                        ));
                    }
                }
                self.handle_expression(&storage_reassignment.rhs, namespace);
            }
            ty::TyExpressionVariant::Return(exp) => self.handle_expression(exp, namespace),
        }
    }

    fn handle_intrinsic_function(
        &self,
        ty::TyIntrinsicFunctionKind { arguments, .. }: &ty::TyIntrinsicFunctionKind,
        namespace: &namespace::Module,
    ) {
        for arg in arguments {
            self.handle_expression(arg, namespace);
        }
    }

    fn handle_while_loop(
        &self,
        body: &ty::TyCodeBlock,
        condition: &ty::TyExpression,
        namespace: &namespace::Module,
    ) {
        self.handle_expression(condition, namespace);
        for node in &body.contents {
            self.traverse_node(node, namespace);
        }
    }

    fn collect_call_path_prefixes(
        &self,
        call_path: &CallPath,
        expression: &ty::TyExpression,
        function_decl_id: Option<&DeclId>,
        namespace: &namespace::Module,
    ) {
        let decl_engine = self.engines.de();

        let implementing_type_name = function_decl_id
            .and_then(|decl_id| {
                decl_engine
                    .get_function(decl_id.clone(), &call_path.span())
                    .ok()
            })
            .and_then(|function_decl| function_decl.implementing_type)
            .and_then(|impl_type| impl_type.get_decl_ident(decl_engine));

        pub fn iter_prefixes<T>(slice: &[T]) -> impl Iterator<Item = &[T]> + DoubleEndedIterator {
            (1..=slice.len()).map(move |len| &slice[..len])
        }

        let prefixes = if let Some(impl_type_name) = implementing_type_name {
            // the last prefix of the call path is not a module but a type
            if let Some((last, prefixes)) = call_path.prefixes.split_last() {
                if let Some(mut token) = self.tokens.try_get_mut(&to_ident_key(last)).try_unwrap() {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                    token.type_def = Some(TypeDefinition::Ident(impl_type_name));
                }
                prefixes
            } else {
                &call_path.prefixes
            }
        } else {
            &call_path.prefixes
        };

        for (mod_path, ident) in iter_prefixes(prefixes).zip(prefixes) {
            if let Some(mut token) = self.tokens.try_get_mut(&to_ident_key(ident)).try_unwrap() {
                token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));

                if let Some(name) = namespace
                    .submodule(mod_path)
                    .and_then(|tgt_submod| tgt_submod.name.clone())
                {
                    token.type_def = Some(TypeDefinition::Ident(name));
                }
            }
        }
    }

    fn collect_supertrait(&self, supertrait: &Supertrait) {
        if let Some(mut token) = self
            .tokens
            .try_get_mut(&to_ident_key(&supertrait.name.suffix))
            .try_unwrap()
        {
            token.typed = Some(TypedAstToken::TypedSupertrait(supertrait.clone()));
            token.type_def = if let Some(decl_id) = &supertrait.decl_id {
                let decl_engine = self.engines.de();
                if let Ok(trait_decl) = decl_engine.get_trait(decl_id.clone(), &decl_id.span()) {
                    Some(TypeDefinition::Ident(trait_decl.name))
                } else {
                    Some(TypeDefinition::Ident(supertrait.name.suffix.clone()))
                }
            } else {
                Some(TypeDefinition::Ident(supertrait.name.suffix.clone()))
            }
        }
    }

    fn collect_typed_trait_fn_token(&self, trait_fn: &ty::TyTraitFn) {
        if let Some(mut token) = self
            .tokens
            .try_get_mut(&to_ident_key(&trait_fn.name))
            .try_unwrap()
        {
            token.typed = Some(TypedAstToken::TypedTraitFn(trait_fn.clone()));
            token.type_def = Some(TypeDefinition::Ident(trait_fn.name.clone()));
        }

        for parameter in &trait_fn.parameters {
            self.collect_typed_fn_param_token(parameter);
        }

        let return_ident = Ident::new(trait_fn.return_type_span.clone());
        if let Some(mut token) = self
            .tokens
            .try_get_mut(&to_ident_key(&return_ident))
            .try_unwrap()
        {
            token.typed = Some(TypedAstToken::TypedTraitFn(trait_fn.clone()));
            token.type_def = Some(TypeDefinition::TypeId(trait_fn.return_type));
        }
    }

    fn collect_typed_fn_param_token(&self, param: &ty::TyFunctionParameter) {
        let typed_token = TypedAstToken::TypedFunctionParameter(param.clone());
        if let Some(mut token) = self
            .tokens
            .try_get_mut(&to_ident_key(&param.name))
            .try_unwrap()
        {
            token.typed = Some(typed_token.clone());
            token.type_def = Some(TypeDefinition::TypeId(param.type_id));
        }

        self.collect_type_id(param.type_id, &typed_token, param.type_span.clone());
    }

    fn collect_type_argument(&self, type_arg: &TypeArgument) {
        if let Some(name_spans) = &type_arg.name_spans {
            self.collect_span_tree(name_spans, type_arg);
        } else {
            self.collect_type_id(
                type_arg.type_id,
                &TypedAstToken::TypedArgument(type_arg.clone()),
                type_arg.span(),
            );
        }
    }

    fn collect_span_tree(&self, span_tree: &SpanTree, type_arg: &TypeArgument) {
        let type_engine = self.engines.te();
        let type_info = type_engine.get(type_arg.type_id);
        match &type_info {
            TypeInfo::Enum {
                type_parameters, ..
            }
            | TypeInfo::Struct {
                type_parameters, ..
            } => {
                self.collect_type_id(
                    type_arg.type_id,
                    &TypedAstToken::TypedArgument(type_arg.clone()),
                    span_tree.span.clone(),
                );

                let child_type_args = type_parameters.iter().map(TypeArgument::from);
                for (child_tree, type_arg) in span_tree.children.iter().zip(child_type_args) {
                    self.collect_span_tree(child_tree, &type_arg);
                }
            }
            TypeInfo::Custom { type_arguments, .. } => {
                self.collect_type_id(
                    type_arg.type_id,
                    &TypedAstToken::TypedArgument(type_arg.clone()),
                    span_tree.span.clone(),
                );

                if let Some(type_args) = type_arguments {
                    for (child_tree, type_arg) in span_tree.children.iter().zip(type_args.iter()) {
                        self.collect_span_tree(child_tree, type_arg);
                    }
                }
            }
            _ => {
                self.collect_type_id(
                    type_arg.type_id,
                    &TypedAstToken::TypedArgument(type_arg.clone()),
                    // use the whole span instead of just the name if we don't know
                    // how to walk it
                    type_arg.span(),
                );
            }
        };
    }

    fn collect_type_id(&self, type_id: TypeId, typed_token: &TypedAstToken, type_span: Span) {
        let type_engine = self.engines.te();
        let type_info = type_engine.get(type_id);
        let symbol_kind = type_info_to_symbol_kind(type_engine, &type_info);
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
                    self.collect_type_argument(type_arg);
                }
            }
            TypeInfo::Enum {
                type_parameters,
                variant_types,
                ..
            } => {
                if let Some(token) = self
                    .tokens
                    .try_get_mut(&to_ident_key(&Ident::new(type_span)))
                    .try_unwrap()
                {
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
                if let Some(token) = self
                    .tokens
                    .try_get_mut(&to_ident_key(&Ident::new(type_span)))
                    .try_unwrap()
                {
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
            TypeInfo::Custom {
                type_arguments,
                name,
            } => {
                if let Some(token) = self
                    .tokens
                    .try_get_mut(&to_ident_key(&Ident::new(name.span())))
                    .try_unwrap()
                {
                    assign_type_to_token(token, symbol_kind, typed_token.clone(), type_id);
                }

                if let Some(type_arguments) = type_arguments {
                    for type_arg in type_arguments {
                        self.collect_type_argument(type_arg);
                    }
                }
            }
            TypeInfo::Storage { fields } => {
                for field in fields {
                    self.collect_ty_struct_field(field);
                }
            }
            _ => {
                if let Some(token) = self
                    .tokens
                    .try_get_mut(&to_ident_key(&Ident::new(type_span)))
                    .try_unwrap()
                {
                    assign_type_to_token(token, symbol_kind, typed_token.clone(), type_id);
                }
            }
        }
    }

    fn collect_typed_fn_decl(
        &self,
        func_decl: &ty::TyFunctionDeclaration,
        namespace: &namespace::Module,
    ) {
        let typed_token = TypedAstToken::TypedFunctionDeclaration(func_decl.clone());
        if let Some(mut token) = self
            .tokens
            .try_get_mut(&to_ident_key(&func_decl.name))
            .try_unwrap()
        {
            token.typed = Some(typed_token.clone());
            token.type_def = Some(TypeDefinition::Ident(func_decl.name.clone()));
        }

        for node in &func_decl.body.contents {
            self.traverse_node(node, namespace);
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
        if let Some(mut token) = self
            .tokens
            .try_get_mut(&to_ident_key(&enum_variant.name))
            .try_unwrap()
        {
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
        if let Some(mut token) = self
            .tokens
            .try_get_mut(&to_ident_key(&field.name))
            .try_unwrap()
        {
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

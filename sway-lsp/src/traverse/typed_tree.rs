#![allow(dead_code)]
use crate::core::{
    token::{
        to_ident_key, type_info_to_symbol_kind, SymbolKind, Token, TypeDefinition, TypedAstToken,
    },
    token_map::TokenMap,
};
use dashmap::mapref::one::RefMut;
use sway_core::{
    decl_engine::InterfaceDeclId,
    language::{
        parsed::{ImportType, Supertrait},
        ty::{self, GetDeclIdent, TyEnumVariant, TyModule, TyProgram, TyProgramKind, TySubmodule},
        CallPathTree,
    },
    namespace,
    type_system::TypeArgument,
    Engines, TraitConstraint, TypeId, TypeInfo,
};
use sway_types::{Ident, Span, Spanned};

pub struct TypedTree<'a> {
    engines: Engines<'a>,
    tokens: &'a TokenMap,
    namespace: &'a namespace::Module,
}

impl<'a> TypedTree<'a> {
    pub fn new(
        engines: Engines<'a>,
        tokens: &'a TokenMap,
        namespace: &'a namespace::Module,
    ) -> Self {
        Self {
            engines,
            tokens,
            namespace,
        }
    }

    pub fn traverse_node(&self, node: &ty::TyAstNode) {
        match &node.content {
            ty::TyAstNodeContent::Declaration(declaration) => self.handle_declaration(declaration),
            ty::TyAstNodeContent::Expression(expression)
            | ty::TyAstNodeContent::ImplicitReturnExpression(expression) => {
                self.handle_expression(expression)
            }
            ty::TyAstNodeContent::SideEffect(side_effect) => self.handle_side_effect(side_effect),
        };
    }

    /// Collects the library name and the module name from the dep statement
    pub fn collect_module_spans(&self, typed_program: &TyProgram) {
        if let TyProgramKind::Library { name } = &typed_program.kind {
            if let Some(mut token) = self.tokens.try_get_mut(&to_ident_key(name)).try_unwrap() {
                token.typed = Some(TypedAstToken::TypedProgramKind(typed_program.kind.clone()));
                token.type_def = Some(TypeDefinition::Ident(name.clone()));
            }
        }
        self.collect_module(&typed_program.root);
    }

    fn collect_module(&self, typed_module: &TyModule) {
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

    fn handle_declaration(&self, declaration: &ty::TyDeclaration) {
        let decl_engine = self.engines.de();
        match declaration {
            ty::TyDeclaration::VariableDeclaration(variable) => {
                if let Some(mut token) = self
                    .tokens
                    .try_get_mut(&to_ident_key(&variable.name))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                    token.type_def = Some(TypeDefinition::Ident(variable.name.clone()));
                }
                if let Some(call_path_tree) = &variable.type_ascription.call_path_tree {
                    self.collect_call_path_tree(call_path_tree, &variable.type_ascription);
                }
                self.handle_expression(&variable.body);
            }
            ty::TyDeclaration::ConstantDeclaration { decl_id, .. } => {
                let const_decl = decl_engine.get_constant(decl_id);
                self.collect_const_decl(&const_decl);
            }
            ty::TyDeclaration::FunctionDeclaration { decl_id, .. } => {
                let func_decl = decl_engine.get_function(decl_id);
                self.collect_typed_fn_decl(&func_decl);
            }
            ty::TyDeclaration::TraitDeclaration { decl_id, .. } => {
                let trait_decl = decl_engine.get_trait(decl_id);
                if let Some(mut token) = self
                    .tokens
                    .try_get_mut(&to_ident_key(&trait_decl.name))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                    token.type_def = Some(TypeDefinition::Ident(trait_decl.name.clone()));
                }

                for item in &trait_decl.interface_surface {
                    match item {
                        ty::TyTraitInterfaceItem::TraitFn(trait_fn_decl_ref) => {
                            let trait_fn = decl_engine.get_trait_fn(trait_fn_decl_ref);
                            self.collect_typed_trait_fn_token(&trait_fn);
                        }
                    }
                }
                for supertrait in trait_decl.supertraits {
                    self.collect_supertrait(&supertrait);
                }
            }
            ty::TyDeclaration::StructDeclaration { decl_id, .. } => {
                let struct_decl = decl_engine.get_struct(decl_id);
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
                        token.typed = Some(TypedAstToken::TypedParameter(type_param.clone()));
                        token.type_def = Some(TypeDefinition::TypeId(type_param.type_id));
                    }
                }
            }
            ty::TyDeclaration::EnumDeclaration { decl_id, .. } => {
                let enum_decl = decl_engine.get_enum(decl_id);
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
                        token.typed = Some(TypedAstToken::TypedParameter(type_param.clone()));
                        token.type_def = Some(TypeDefinition::TypeId(type_param.type_id));
                    }
                }

                for variant in &enum_decl.variants {
                    self.collect_ty_enum_variant(variant);
                }
            }
            ty::TyDeclaration::ImplTrait { decl_id, .. } => {
                let ty::TyImplTrait {
                    impl_type_parameters,
                    trait_name,
                    trait_type_arguments,
                    trait_decl_ref,
                    items,
                    implementing_for,
                    ..
                } = decl_engine.get_impl_trait(decl_id);
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
                        token.typed = Some(TypedAstToken::Ident(ident.clone()));
                    }
                }

                if let Some(mut token) = self
                    .tokens
                    .try_get_mut(&to_ident_key(&trait_name.suffix))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));

                    token.type_def = if let Some(decl_ref) = &trait_decl_ref {
                        match &decl_ref.id {
                            InterfaceDeclId::Abi(decl_id) => {
                                let abi_decl = decl_engine.get_abi(decl_id);
                                Some(TypeDefinition::Ident(abi_decl.name))
                            }
                            InterfaceDeclId::Trait(decl_id) => {
                                let trait_decl = decl_engine.get_trait(decl_id);
                                Some(TypeDefinition::Ident(trait_decl.name))
                            }
                        }
                    } else {
                        Some(TypeDefinition::TypeId(implementing_for.type_id))
                    };
                }

                for type_arg in trait_type_arguments {
                    self.collect_type_argument(&type_arg);
                }

                for item in items {
                    match item {
                        ty::TyTraitItem::Fn(method_ref) => {
                            let method = decl_engine.get_function(&method_ref);
                            self.collect_typed_fn_decl(&method);
                        }
                    }
                }

                self.collect_type_argument(&implementing_for);

                // collect the root type argument again with declaration info this time so the
                // impl is registered
                self.collect_type_id(
                    implementing_for.type_id,
                    &TypedAstToken::TypedDeclaration(declaration.clone()),
                    implementing_for
                        .call_path_tree
                        .as_ref()
                        .map(|tree| tree.call_path.suffix.span())
                        .unwrap_or(implementing_for.span()),
                );
            }
            ty::TyDeclaration::AbiDeclaration { decl_id, .. } => {
                let abi_decl = decl_engine.get_abi(decl_id);
                if let Some(mut token) = self
                    .tokens
                    .try_get_mut(&to_ident_key(&abi_decl.name))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                    token.type_def = Some(TypeDefinition::Ident(abi_decl.name.clone()));
                }

                for item in &abi_decl.interface_surface {
                    match item {
                        ty::TyTraitInterfaceItem::TraitFn(trait_fn_decl_ref) => {
                            let trait_fn = decl_engine.get_trait_fn(trait_fn_decl_ref);
                            self.collect_typed_trait_fn_token(&trait_fn);
                        }
                    }
                }

                for supertrait in abi_decl.supertraits {
                    self.collect_supertrait(&supertrait);
                }
            }
            ty::TyDeclaration::GenericTypeForFunctionScope { name, type_id } => {
                if let Some(mut token) = self.tokens.try_get_mut(&to_ident_key(name)).try_unwrap() {
                    token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                    token.type_def = Some(TypeDefinition::TypeId(*type_id));
                }
            }
            ty::TyDeclaration::ErrorRecovery(_) => {}
            ty::TyDeclaration::StorageDeclaration { decl_id, .. } => {
                let storage_decl = decl_engine.get_storage(decl_id);
                for field in &storage_decl.fields {
                    if let Some(mut token) = self
                        .tokens
                        .try_get_mut(&to_ident_key(&field.name))
                        .try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::TypedStorageField(field.clone()));
                        token.type_def = Some(TypeDefinition::Ident(field.name.clone()));
                    }

                    self.collect_type_argument(&field.type_argument);

                    self.handle_expression(&field.initializer);
                }
            }
        }
    }

    fn handle_side_effect(&self, side_effect: &ty::TySideEffect) {
        use ty::TySideEffectVariant::*;
        match &side_effect.side_effect {
            UseStatement(
                use_statement @ ty::TyUseStatement {
                    call_path,
                    import_type,
                    alias,
                    is_absolute: _,
                },
            ) => {
                for (mod_path, ident) in iter_prefixes(call_path).zip(call_path) {
                    if let Some(mut token) =
                        self.tokens.try_get_mut(&to_ident_key(ident)).try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::TypedUseStatement(use_statement.clone()));

                        if let Some(name) = self
                            .namespace
                            .submodule(mod_path)
                            .and_then(|tgt_submod| tgt_submod.name.clone())
                        {
                            token.type_def = Some(TypeDefinition::Ident(name));
                        }
                    }
                }

                match &import_type {
                    ImportType::Item(item) => {
                        if let Some(mut token) =
                            self.tokens.try_get_mut(&to_ident_key(item)).try_unwrap()
                        {
                            token.typed =
                                Some(TypedAstToken::TypedUseStatement(use_statement.clone()));

                            let mut symbol_kind = SymbolKind::Unknown;
                            let mut type_def = None;

                            if let Some(decl_ident) = self
                                .namespace
                                .submodule(call_path)
                                .and_then(|module| module.symbols().get(item))
                                .and_then(|decl| decl.get_decl_ident())
                            {
                                // Update the symbol kind to match the declarations symbol kind
                                if let Some(decl) =
                                    self.tokens.try_get(&to_ident_key(&decl_ident)).try_unwrap()
                                {
                                    symbol_kind = decl.value().kind.clone();
                                }
                                type_def = Some(TypeDefinition::Ident(decl_ident));
                            }

                            token.kind = symbol_kind.clone();
                            token.type_def = type_def.clone();

                            // the alias should take on the same symbol kind and type definition
                            if let Some(alias) = alias {
                                if let Some(mut token) =
                                    self.tokens.try_get_mut(&to_ident_key(alias)).try_unwrap()
                                {
                                    token.typed = Some(TypedAstToken::TypedUseStatement(
                                        use_statement.clone(),
                                    ));
                                    token.kind = symbol_kind;
                                    token.type_def = type_def;
                                }
                            }
                        }
                    }
                    ImportType::SelfImport(span) => {
                        if let Some(mut token) = self
                            .tokens
                            .try_get_mut(&to_ident_key(&Ident::new(span.clone())))
                            .try_unwrap()
                        {
                            token.typed =
                                Some(TypedAstToken::TypedUseStatement(use_statement.clone()));

                            if let Some(name) = self
                                .namespace
                                .submodule(call_path)
                                .and_then(|tgt_submod| tgt_submod.name.clone())
                            {
                                token.type_def = Some(TypeDefinition::Ident(name));
                            }
                        }
                    }
                    ImportType::Star => {}
                }
            }
            IncludeStatement => {}
        }
    }

    fn handle_expression(&self, expression: &ty::TyExpression) {
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
                function_decl_ref,
                type_binding,
                ..
            } => {
                if let Some(type_binding) = type_binding {
                    for type_arg in &type_binding.type_arguments.to_vec() {
                        self.collect_type_argument(type_arg);
                    }
                }

                let implementing_type_name = decl_engine
                    .get_function(function_decl_ref)
                    .implementing_type
                    .and_then(|impl_type| impl_type.get_decl_ident());

                let prefixes = if let Some(impl_type_name) = implementing_type_name {
                    // the last prefix of the call path is not a module but a type
                    if let Some((last, prefixes)) = call_path.prefixes.split_last() {
                        if let Some(mut token) =
                            self.tokens.try_get_mut(&to_ident_key(last)).try_unwrap()
                        {
                            token.typed = Some(TypedAstToken::Ident(impl_type_name.clone()));
                            token.type_def = Some(TypeDefinition::Ident(impl_type_name));
                        }
                        prefixes
                    } else {
                        &call_path.prefixes
                    }
                } else {
                    &call_path.prefixes
                };
                self.collect_call_path_prefixes(prefixes);

                if let Some(mut token) = self
                    .tokens
                    .try_get_mut(&to_ident_key(&call_path.suffix))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                    let function_decl = decl_engine.get_function(function_decl_ref);
                    token.type_def = Some(TypeDefinition::Ident(function_decl.name));
                }

                for exp in contract_call_params.values() {
                    self.handle_expression(exp);
                }

                for (ident, exp) in arguments {
                    if let Some(mut token) =
                        self.tokens.try_get_mut(&to_ident_key(ident)).try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::Ident(ident.clone()));
                    }
                    self.handle_expression(exp);
                }

                let function_decl = decl_engine.get_function(function_decl_ref);
                for node in &function_decl.body.contents {
                    self.traverse_node(node);
                }
            }
            ty::TyExpressionVariant::LazyOperator { lhs, rhs, .. } => {
                self.handle_expression(lhs);
                self.handle_expression(rhs);
            }
            ty::TyExpressionVariant::VariableExpression {
                ref name,
                ref span,
                ref call_path,
                ..
            } => {
                if let Some(call_path) = call_path {
                    self.collect_call_path_prefixes(&call_path.prefixes);
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
            ty::TyExpressionVariant::StructExpression {
                fields,
                call_path_binding,
                ..
            } => {
                if let Some(mut token) = self
                    .tokens
                    .try_get_mut(&to_ident_key(&call_path_binding.inner.suffix))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                    token.type_def = Some(TypeDefinition::TypeId(expression.return_type));
                }

                for type_arg in &call_path_binding.type_arguments.to_vec() {
                    self.collect_type_argument(type_arg);
                }

                self.collect_call_path_prefixes(&call_path_binding.inner.prefixes);

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
                    self.handle_expression(&field.value);
                }
            }
            ty::TyExpressionVariant::CodeBlock(code_block) => {
                for node in &code_block.contents {
                    self.traverse_node(node);
                }
            }
            ty::TyExpressionVariant::FunctionParameter { .. } => {}
            ty::TyExpressionVariant::MatchExp {
                desugared,
                scrutinees,
            } => {
                // Order is important here, the expression must be processed first otherwise the
                // scrutinee information will get overwritten by processing the underlying tree of
                // conditions
                self.handle_expression(desugared);
                for s in scrutinees {
                    self.handle_scrutinee(s);
                }
            }
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
            ty::TyExpressionVariant::AsmExpression { registers, .. } => {
                for register in registers {
                    if let Some(initializer) = &register.initializer {
                        self.handle_expression(initializer);
                    }
                }
            }
            ty::TyExpressionVariant::StructFieldAccess {
                prefix,
                field_to_access,
                field_instantiation_span,
                ..
            } => {
                self.handle_expression(prefix);

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
                self.handle_expression(prefix);

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
                contents,
                call_path_binding,
                ..
            } => {
                if let Some(mut token) = self
                    .tokens
                    .try_get_mut(&to_ident_key(&call_path_binding.inner.suffix))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                    token.type_def =
                        Some(TypeDefinition::Ident(enum_decl.call_path.suffix.clone()));
                }

                for type_arg in &call_path_binding.type_arguments.to_vec() {
                    self.collect_type_argument(type_arg);
                }

                self.collect_call_path_prefixes(&call_path_binding.inner.prefixes);

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
                    self.handle_expression(contents);
                }
            }
            ty::TyExpressionVariant::AbiCast {
                abi_name, address, ..
            } => {
                self.collect_call_path_prefixes(&abi_name.prefixes);

                if let Some(mut token) = self
                    .tokens
                    .try_get_mut(&to_ident_key(&abi_name.suffix))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                    if let Some(abi_def_ident) = self
                        .namespace
                        .submodule(&abi_name.prefixes)
                        .and_then(|module| module.symbols().get(&abi_name.suffix))
                        .and_then(|decl| decl.get_decl_ident())
                    {
                        token.type_def = Some(TypeDefinition::Ident(abi_def_ident));
                    }
                }

                self.handle_expression(address);
            }
            ty::TyExpressionVariant::StorageAccess(storage_access) => {
                for field in &storage_access.fields {
                    if let Some(mut token) = self
                        .tokens
                        .try_get_mut(&to_ident_key(&field.name))
                        .try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::TyStorageAccessDescriptor(field.clone()));
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
            } => self.handle_while_loop(body, condition),
            ty::TyExpressionVariant::Break => (),
            ty::TyExpressionVariant::Continue => (),
            ty::TyExpressionVariant::Reassignment(reassignment) => {
                self.handle_expression(&reassignment.rhs);

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
                self.handle_expression(&storage_reassignment.rhs);
            }
            ty::TyExpressionVariant::Return(exp) => self.handle_expression(exp),
        }
    }

    fn handle_scrutinee(&self, scrutinee: &ty::TyScrutinee) {
        use ty::TyScrutineeVariant::*;
        match &scrutinee.variant {
            CatchAll => {}
            Constant(name, _, const_decl) => {
                if let Some(mut token) = self.tokens.try_get_mut(&to_ident_key(name)).try_unwrap() {
                    token.typed = Some(TypedAstToken::TypedScrutinee(scrutinee.clone()));
                    token.type_def = Some(TypeDefinition::Ident(const_decl.name.clone()));
                }
            }
            Literal(_) => {
                if let Some(mut token) = self
                    .tokens
                    .try_get_mut(&to_ident_key(&Ident::new(scrutinee.span.clone())))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedScrutinee(scrutinee.clone()));
                }
            }
            Variable(ident) => {
                if let Some(mut token) = self.tokens.try_get_mut(&to_ident_key(ident)).try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedScrutinee(scrutinee.clone()));
                }
            }
            StructScrutinee {
                struct_name,
                decl_name,
                fields,
            } => {
                if let Some(mut token) = self
                    .tokens
                    .try_get_mut(&to_ident_key(struct_name))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedScrutinee(scrutinee.clone()));
                    token.type_def = Some(TypeDefinition::Ident(decl_name.clone()));
                }

                for field in fields {
                    if let Some(mut token) = self
                        .tokens
                        .try_get_mut(&to_ident_key(&field.field))
                        .try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::TypedScrutinee(scrutinee.clone()));
                        token.type_def = Some(TypeDefinition::Ident(field.field_def_name.clone()));
                    }

                    if let Some(scrutinee) = &field.scrutinee {
                        self.handle_scrutinee(scrutinee);
                    }
                }
            }
            EnumScrutinee {
                call_path,
                decl_name,
                variant,
                value,
            } => {
                let prefixes = if let Some((last, prefixes)) = call_path.prefixes.split_last() {
                    // the last prefix of the call path is not a module but a type
                    if let Some(mut token) =
                        self.tokens.try_get_mut(&to_ident_key(last)).try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::TypedScrutinee(scrutinee.clone()));
                        token.type_def = Some(TypeDefinition::Ident(decl_name.clone()));
                    }
                    prefixes
                } else {
                    &call_path.prefixes
                };
                self.collect_call_path_prefixes(prefixes);

                if let Some(mut token) = self
                    .tokens
                    .try_get_mut(&to_ident_key(&call_path.suffix))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedScrutinee(scrutinee.clone()));
                    token.type_def = Some(TypeDefinition::Ident(variant.name.clone()));
                }

                self.handle_scrutinee(value);
            }
            Tuple(scrutinees) => {
                for s in scrutinees {
                    self.handle_scrutinee(s);
                }
            }
        }
    }

    fn handle_intrinsic_function(
        &self,
        ty::TyIntrinsicFunctionKind {
            arguments,
            type_arguments,
            ..
        }: &ty::TyIntrinsicFunctionKind,
    ) {
        for type_arg in type_arguments {
            self.collect_type_argument(type_arg);
        }
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

    fn collect_call_path_prefixes(&self, prefixes: &[Ident]) {
        for (mod_path, ident) in iter_prefixes(prefixes).zip(prefixes) {
            if let Some(mut token) = self.tokens.try_get_mut(&to_ident_key(ident)).try_unwrap() {
                token.typed = Some(TypedAstToken::Ident(ident.clone()));

                if let Some(name) = self
                    .namespace
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
            token.type_def = if let Some(decl_ref) = &supertrait.decl_ref {
                let decl_engine = self.engines.de();
                let trait_decl = decl_engine.get_trait(decl_ref);
                Some(TypeDefinition::Ident(trait_decl.name))
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
            token.typed = Some(typed_token);
            token.type_def = Some(TypeDefinition::TypeId(param.type_argument.type_id));
        }

        self.collect_type_argument(&param.type_argument);
    }

    fn collect_trait_constraint(
        &self,
        trait_constraint @ TraitConstraint {
            trait_name,
            type_arguments,
        }: &TraitConstraint,
    ) {
        self.collect_call_path_prefixes(&trait_name.prefixes);

        if let Some(mut token) = self
            .tokens
            .try_get_mut(&to_ident_key(&trait_name.suffix))
            .try_unwrap()
        {
            token.typed = Some(TypedAstToken::TypedTraitConstraint(
                trait_constraint.clone(),
            ));
            if let Some(trait_def_ident) = self
                .namespace
                .submodule(&trait_name.prefixes)
                .and_then(|module| module.symbols().get(&trait_name.suffix))
                .and_then(|decl| decl.get_decl_ident())
            {
                token.type_def = Some(TypeDefinition::Ident(trait_def_ident));
            }
        }

        for type_arg in type_arguments {
            self.collect_type_argument(type_arg);
        }
    }

    fn collect_type_argument(&self, type_arg: &TypeArgument) {
        if let Some(call_path_tree) = &type_arg.call_path_tree {
            self.collect_call_path_tree(call_path_tree, type_arg);
        } else {
            self.collect_type_id(
                type_arg.type_id,
                &TypedAstToken::TypedArgument(type_arg.clone()),
                type_arg.span(),
            );
        }
    }

    fn collect_call_path_tree(&self, tree: &CallPathTree, type_arg: &TypeArgument) {
        let type_engine = self.engines.te();
        let type_info = type_engine.get(type_arg.type_id);

        self.collect_call_path_prefixes(&tree.call_path.prefixes);
        self.collect_type_id(
            type_arg.type_id,
            &TypedAstToken::TypedArgument(type_arg.clone()),
            tree.call_path.suffix.span(),
        );

        match &type_info {
            TypeInfo::Enum {
                type_parameters, ..
            }
            | TypeInfo::Struct {
                type_parameters, ..
            } => {
                let child_type_args = type_parameters.iter().map(TypeArgument::from);
                for (child_tree, type_arg) in tree.children.iter().zip(child_type_args) {
                    self.collect_call_path_tree(child_tree, &type_arg);
                }
            }
            TypeInfo::Custom {
                type_arguments: Some(type_args),
                ..
            } => {
                for (child_tree, type_arg) in tree.children.iter().zip(type_args.iter()) {
                    self.collect_call_path_tree(child_tree, type_arg);
                }
            }
            TypeInfo::ContractCaller { .. } => {
                // single generic argument to ContractCaller<_> has to be a single ABI
                // definition call path which we can collect without recursion
                if let Some(child_tree) = tree.children.first() {
                    let abi_call_path = &child_tree.call_path;

                    self.collect_call_path_prefixes(&abi_call_path.prefixes);
                    if let Some(mut token) = self
                        .tokens
                        .try_get_mut(&to_ident_key(&abi_call_path.suffix))
                        .try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::TypedArgument(type_arg.clone()));
                        if let Some(abi_def_ident) = self
                            .namespace
                            .submodule(&abi_call_path.prefixes)
                            .and_then(|module| module.symbols().get(&abi_call_path.suffix))
                            .and_then(|decl| decl.get_decl_ident())
                        {
                            token.type_def = Some(TypeDefinition::Ident(abi_def_ident));
                        }
                    }
                }
            }
            _ => {}
        };
    }

    fn collect_type_id(&self, type_id: TypeId, typed_token: &TypedAstToken, type_span: Span) {
        let type_engine = self.engines.te();
        let type_info = type_engine.get(type_id);
        let symbol_kind = type_info_to_symbol_kind(type_engine, &type_info);
        match &type_info {
            TypeInfo::Array(type_arg, ..) => {
                self.collect_type_argument(type_arg);
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
                call_path: name,
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

    fn collect_const_decl(&self, const_decl: &ty::TyConstantDeclaration) {
        if let Some(mut token) = self
            .tokens
            .try_get_mut(&to_ident_key(&const_decl.name))
            .try_unwrap()
        {
            token.typed = Some(TypedAstToken::TypedConstantDeclaration(const_decl.clone()));
            token.type_def = Some(TypeDefinition::Ident(const_decl.name.clone()));
        }

        if let Some(call_path_tree) = &const_decl.type_ascription.call_path_tree {
            self.collect_call_path_tree(call_path_tree, &const_decl.type_ascription);
        }

        if let Some(value) = &const_decl.value {
            self.handle_expression(value);
        }
    }

    fn collect_typed_fn_decl(&self, func_decl: &ty::TyFunctionDeclaration) {
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

        self.collect_type_argument(&func_decl.return_type);

        for (ident, trait_constraints) in &func_decl.where_clause {
            for constraint in trait_constraints {
                self.collect_trait_constraint(constraint);
            }

            if let Some(mut token) = self.tokens.try_get_mut(&to_ident_key(ident)).try_unwrap() {
                token.typed = Some(typed_token.clone());
                if let Some(param_decl_ident) = func_decl
                    .type_parameters
                    .iter()
                    .find(|type_param| type_param.name_ident.as_str() == ident.as_str())
                    .map(|type_param| type_param.name_ident.clone())
                {
                    token.type_def = Some(TypeDefinition::Ident(param_decl_ident));
                }
            }
        }
    }

    fn collect_ty_enum_variant(&self, enum_variant: &TyEnumVariant) {
        let typed_token = TypedAstToken::TypedEnumVariant(enum_variant.clone());
        if let Some(mut token) = self
            .tokens
            .try_get_mut(&to_ident_key(&enum_variant.name))
            .try_unwrap()
        {
            token.typed = Some(typed_token);
            token.type_def = Some(TypeDefinition::TypeId(enum_variant.type_argument.type_id));
        }

        self.collect_type_argument(&enum_variant.type_argument);
    }

    fn collect_ty_struct_field(&self, field: &ty::TyStructField) {
        if let Some(mut token) = self
            .tokens
            .try_get_mut(&to_ident_key(&field.name))
            .try_unwrap()
        {
            token.typed = Some(TypedAstToken::TypedStructField(field.clone()));
            token.type_def = Some(TypeDefinition::TypeId(field.type_argument.type_id));
        }

        self.collect_type_argument(&field.type_argument);
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

fn iter_prefixes<T>(slice: &[T]) -> impl Iterator<Item = &[T]> + DoubleEndedIterator {
    (1..=slice.len()).map(move |len| &slice[..len])
}

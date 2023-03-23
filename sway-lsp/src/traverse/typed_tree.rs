#![allow(dead_code)]
use crate::{
    core::token::{
        to_ident_key, type_info_to_symbol_kind, SymbolKind, Token, TypeDefinition, TypedAstToken,
    },
    traverse::{Parse, ParseContext},
};
use dashmap::mapref::one::RefMut;
use sway_core::{
    decl_engine::{id::DeclId, InterfaceDeclId},
    language::{
        parsed::{ImportType, Supertrait},
        ty::{self, GetDeclIdent, TyModule, TyProgram, TySubmodule},
        CallPathTree,
    },
    type_system::TypeArgument,
    TraitConstraint, TypeId, TypeInfo,
};
use sway_types::{Ident, Span, Spanned};

pub struct TypedTree<'a> {
    ctx: &'a ParseContext<'a>,
}

impl<'a> TypedTree<'a> {
    pub fn new(ctx: &'a ParseContext<'a>) -> Self {
        Self { ctx }
    }

    pub fn traverse_node(&self, node: &ty::TyAstNode) {
        node.parse(self.ctx);
    }

    /// Collects module names from the mod statements
    pub fn collect_module_spans(&self, typed_program: &TyProgram) {
        self.collect_module(&typed_program.root);
    }

    fn collect_module(&self, typed_module: &TyModule) {
        for (
            _,
            TySubmodule {
                module,
                mod_name_span,
            },
        ) in &typed_module.submodules
        {
            if let Some(mut token) = self
                .ctx
                .tokens
                .try_get_mut(&to_ident_key(&Ident::new(mod_name_span.clone())))
                .try_unwrap()
            {
                token.typed = Some(TypedAstToken::TypedIncludeStatement);
                token.type_def = Some(TypeDefinition::Ident(Ident::new(module.span.clone())));
            }
            self.collect_module(module);
        }
    }
}

impl Parse for ty::TyAstNode {
    fn parse(&self, ctx: &ParseContext) {
        match &self.content {
            ty::TyAstNodeContent::Declaration(declaration) => declaration.parse(ctx),
            ty::TyAstNodeContent::Expression(expression)
            | ty::TyAstNodeContent::ImplicitReturnExpression(expression) => expression.parse(ctx),
            ty::TyAstNodeContent::SideEffect(side_effect) => side_effect.parse(ctx),
        };
    }
}

impl Parse for ty::TyDecl {
    fn parse(&self, ctx: &ParseContext) {
        match self {
            ty::TyDecl::VariableDecl(decl) => decl.parse(ctx),
            ty::TyDecl::ConstantDecl(decl) => decl.parse(ctx),
            ty::TyDecl::FunctionDecl(decl) => decl.parse(ctx),
            ty::TyDecl::TraitDecl(decl) => decl.parse(ctx),
            ty::TyDecl::StructDecl(decl) => decl.parse(ctx),
            ty::TyDecl::EnumDecl(decl) => collect_enum(ctx, &decl.decl_id, self),
            ty::TyDecl::EnumVariantDecl(decl) => collect_enum(ctx, decl.enum_ref.id(), self),
            ty::TyDecl::ImplTrait(decl) => decl.parse(ctx),
            ty::TyDecl::AbiDecl(decl) => decl.parse(ctx),
            ty::TyDecl::GenericTypeForFunctionScope(decl) => decl.parse(ctx),
            ty::TyDecl::ErrorRecovery(_) => {}
            ty::TyDecl::StorageDecl(decl) => decl.parse(ctx),
            ty::TyDecl::TypeAliasDecl(decl) => decl.parse(ctx),
        }
    }
}

impl Parse for ty::TySideEffect {
    fn parse(&self, ctx: &ParseContext) {
        use ty::TySideEffectVariant::*;
        match &self.side_effect {
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
                        ctx.tokens.try_get_mut(&to_ident_key(ident)).try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::TypedUseStatement(use_statement.clone()));

                        if let Some(span) = ctx
                            .namespace
                            .submodule(mod_path)
                            .and_then(|tgt_submod| tgt_submod.span.clone())
                        {
                            token.type_def = Some(TypeDefinition::Ident(Ident::new(span)));
                        }
                    }
                }
                match &import_type {
                    ImportType::Item(item) => {
                        if let Some(mut token) =
                            ctx.tokens.try_get_mut(&to_ident_key(item)).try_unwrap()
                        {
                            token.typed =
                                Some(TypedAstToken::TypedUseStatement(use_statement.clone()));
                            let mut symbol_kind = SymbolKind::Unknown;
                            let mut type_def = None;
                            if let Some(decl_ident) = ctx
                                .namespace
                                .submodule(call_path)
                                .and_then(|module| module.symbols().get(item))
                                .and_then(|decl| decl.get_decl_ident())
                            {
                                // Update the symbol kind to match the declarations symbol kind
                                if let Some(decl) =
                                    ctx.tokens.try_get(&to_ident_key(&decl_ident)).try_unwrap()
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
                                    ctx.tokens.try_get_mut(&to_ident_key(alias)).try_unwrap()
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
                        if let Some(mut token) = ctx
                            .tokens
                            .try_get_mut(&to_ident_key(&Ident::new(span.clone())))
                            .try_unwrap()
                        {
                            token.typed =
                                Some(TypedAstToken::TypedUseStatement(use_statement.clone()));
                            if let Some(span) = ctx
                                .namespace
                                .submodule(call_path)
                                .and_then(|tgt_submod| tgt_submod.span.clone())
                            {
                                token.type_def = Some(TypeDefinition::Ident(Ident::new(span)));
                            }
                        }
                    }
                    ImportType::Star => {}
                }
            }
            IncludeStatement => {}
        }
    }
}

impl Parse for ty::TyExpression {
    fn parse(&self, ctx: &ParseContext) {
        match &self.expression {
            ty::TyExpressionVariant::Literal { .. } => {
                if let Some(mut token) = ctx
                    .tokens
                    .try_get_mut(&to_ident_key(&Ident::new(self.span.clone())))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedExpression(self.clone()));
                }
            }
            ty::TyExpressionVariant::FunctionApplication {
                call_path,
                contract_call_params,
                arguments,
                fn_ref,
                type_binding,
                ..
            } => {
                if let Some(type_binding) = type_binding {
                    type_binding
                        .type_arguments
                        .to_vec()
                        .iter()
                        .for_each(|type_arg| {
                            collect_type_argument(ctx, type_arg);
                        });
                }
                let implementing_type_name = ctx
                    .engines
                    .de()
                    .get_function(fn_ref)
                    .implementing_type
                    .and_then(|impl_type| impl_type.get_decl_ident());
                let prefixes = if let Some(impl_type_name) = implementing_type_name {
                    // the last prefix of the call path is not a module but a type
                    if let Some((last, prefixes)) = call_path.prefixes.split_last() {
                        if let Some(mut token) =
                            ctx.tokens.try_get_mut(&to_ident_key(last)).try_unwrap()
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
                collect_call_path_prefixes(ctx, prefixes);
                if let Some(mut token) = ctx
                    .tokens
                    .try_get_mut(&to_ident_key(&call_path.suffix))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedExpression(self.clone()));
                    let function_decl = ctx.engines.de().get_function(fn_ref);
                    token.type_def = Some(TypeDefinition::Ident(function_decl.name));
                }
                contract_call_params.values().for_each(|exp| exp.parse(ctx));
                for (ident, exp) in arguments {
                    if let Some(mut token) =
                        ctx.tokens.try_get_mut(&to_ident_key(ident)).try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::Ident(ident.clone()));
                    }
                    exp.parse(ctx);
                }
                let function_decl = ctx.engines.de().get_function(fn_ref);
                function_decl
                    .body
                    .contents
                    .iter()
                    .for_each(|node| node.parse(ctx));
            }
            ty::TyExpressionVariant::LazyOperator { lhs, rhs, .. } => {
                lhs.parse(ctx);
                rhs.parse(ctx);
            }
            ty::TyExpressionVariant::ConstantExpression {
                ref const_decl,
                span,
                call_path,
            } => {
                collect_const_decl(ctx, const_decl, span);
                if let Some(call_path) = call_path {
                    collect_call_path_prefixes(ctx, &call_path.prefixes);
                }
            }
            ty::TyExpressionVariant::VariableExpression {
                ref name,
                ref span,
                ref call_path,
                ..
            } => {
                if let Some(call_path) = call_path {
                    collect_call_path_prefixes(ctx, &call_path.prefixes);
                }
                if let Some(mut token) = ctx
                    .tokens
                    .try_get_mut(&to_ident_key(&Ident::new(span.clone())))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedExpression(self.clone()));
                    token.type_def = Some(TypeDefinition::Ident(name.clone()));
                }
            }
            ty::TyExpressionVariant::Tuple { fields } => {
                fields.iter().for_each(|field| field.parse(ctx));
            }
            ty::TyExpressionVariant::Array {
                elem_type: _,
                contents,
            } => {
                contents.iter().for_each(|exp| exp.parse(ctx));
            }
            ty::TyExpressionVariant::ArrayIndex { prefix, index } => {
                prefix.parse(ctx);
                index.parse(ctx);
            }
            ty::TyExpressionVariant::StructExpression {
                fields,
                call_path_binding,
                ..
            } => {
                if let Some(mut token) = ctx
                    .tokens
                    .try_get_mut(&to_ident_key(&call_path_binding.inner.suffix))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedExpression(self.clone()));
                    token.type_def = Some(TypeDefinition::TypeId(self.return_type));
                }
                call_path_binding
                    .type_arguments
                    .to_vec()
                    .iter()
                    .for_each(|type_arg| {
                        collect_type_argument(ctx, type_arg);
                    });
                collect_call_path_prefixes(ctx, &call_path_binding.inner.prefixes);
                fields.iter().for_each(|field| {
                    if let Some(mut token) = ctx
                        .tokens
                        .try_get_mut(&to_ident_key(&field.name))
                        .try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::TypedExpression(field.value.clone()));

                        if let Some(struct_decl) = &ctx
                            .tokens
                            .struct_declaration_of_type_id(ctx.engines, &self.return_type)
                        {
                            struct_decl.fields.iter().for_each(|decl_field| {
                                if decl_field.name == field.name {
                                    token.type_def =
                                        Some(TypeDefinition::Ident(decl_field.name.clone()));
                                }
                            });
                        }
                    }
                    field.value.parse(ctx);
                });
            }
            ty::TyExpressionVariant::CodeBlock(code_block) => {
                code_block.contents.iter().for_each(|node| node.parse(ctx));
            }
            ty::TyExpressionVariant::FunctionParameter { .. } => {}
            ty::TyExpressionVariant::MatchExp {
                desugared,
                scrutinees,
            } => {
                // Order is important here, the expression must be processed first otherwise the
                // scrutinee information will get overwritten by processing the underlying tree of
                // conditions
                desugared.parse(ctx);
                scrutinees.iter().for_each(|s| s.parse(ctx));
            }
            ty::TyExpressionVariant::IfExp {
                condition,
                then,
                r#else,
            } => {
                condition.parse(ctx);
                then.parse(ctx);
                if let Some(r#else) = r#else {
                    r#else.parse(ctx);
                }
            }
            ty::TyExpressionVariant::AsmExpression { registers, .. } => {
                registers.iter().for_each(|r| {
                    if let Some(initializer) = &r.initializer {
                        initializer.parse(ctx);
                    }
                });
            }
            ty::TyExpressionVariant::StructFieldAccess {
                prefix,
                field_to_access,
                field_instantiation_span,
                ..
            } => {
                prefix.parse(ctx);
                if let Some(mut token) = ctx
                    .tokens
                    .try_get_mut(&to_ident_key(&Ident::new(field_instantiation_span.clone())))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedExpression(self.clone()));
                    token.type_def = Some(TypeDefinition::Ident(field_to_access.name.clone()));
                }
            }
            ty::TyExpressionVariant::TupleElemAccess {
                prefix,
                elem_to_access_span,
                ..
            } => {
                prefix.parse(ctx);
                if let Some(mut token) = ctx
                    .tokens
                    .try_get_mut(&to_ident_key(&Ident::new(elem_to_access_span.clone())))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedExpression(self.clone()));
                }
            }
            ty::TyExpressionVariant::EnumInstantiation {
                variant_name,
                variant_instantiation_span,
                enum_ref,
                contents,
                call_path_binding,
                ..
            } => {
                if let Some(mut token) = ctx
                    .tokens
                    .try_get_mut(&to_ident_key(&call_path_binding.inner.suffix))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedExpression(self.clone()));
                    token.type_def = Some(TypeDefinition::Ident(enum_ref.name().clone()));
                }
                call_path_binding
                    .type_arguments
                    .to_vec()
                    .iter()
                    .for_each(|type_arg| {
                        collect_type_argument(ctx, type_arg);
                    });
                collect_call_path_prefixes(ctx, &call_path_binding.inner.prefixes);
                if let Some(mut token) = ctx
                    .tokens
                    .try_get_mut(&to_ident_key(&Ident::new(
                        variant_instantiation_span.clone(),
                    )))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedExpression(self.clone()));
                    token.type_def = Some(TypeDefinition::Ident(variant_name.clone()));
                }
                if let Some(contents) = contents.as_deref() {
                    contents.parse(ctx);
                }
            }
            ty::TyExpressionVariant::AbiCast {
                abi_name, address, ..
            } => {
                collect_call_path_prefixes(ctx, &abi_name.prefixes);
                if let Some(mut token) = ctx
                    .tokens
                    .try_get_mut(&to_ident_key(&abi_name.suffix))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedExpression(self.clone()));
                    if let Some(abi_def_ident) = ctx
                        .namespace
                        .submodule(&abi_name.prefixes)
                        .and_then(|module| module.symbols().get(&abi_name.suffix))
                        .and_then(|decl| decl.get_decl_ident())
                    {
                        token.type_def = Some(TypeDefinition::Ident(abi_def_ident));
                    }
                }
                address.parse(ctx);
            }
            ty::TyExpressionVariant::StorageAccess(storage_access) => {
                storage_access.fields.iter().for_each(|field| {
                    if let Some(mut token) = ctx
                        .tokens
                        .try_get_mut(&to_ident_key(&field.name))
                        .try_unwrap()
                    {
                        token.typed = Some(TypedAstToken::TyStorageAccessDescriptor(field.clone()));
                    }
                });
            }
            ty::TyExpressionVariant::IntrinsicFunction(kind) => {
                kind.parse(ctx);
            }
            ty::TyExpressionVariant::AbiName { .. } => {}
            ty::TyExpressionVariant::EnumTag { exp } => {
                exp.parse(ctx);
            }
            ty::TyExpressionVariant::UnsafeDowncast {
                exp,
                variant,
                call_path_decl: _,
            } => {
                exp.parse(ctx);
                if let Some(mut token) = ctx
                    .tokens
                    .try_get_mut(&to_ident_key(&variant.name))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedExpression(self.clone()));
                }
            }
            ty::TyExpressionVariant::WhileLoop {
                body, condition, ..
            } => {
                condition.parse(ctx);
                body.contents.iter().for_each(|node| node.parse(ctx));
            }
            ty::TyExpressionVariant::Break => (),
            ty::TyExpressionVariant::Continue => (),
            ty::TyExpressionVariant::Reassignment(reassignment) => {
                reassignment.parse(ctx);
            }
            ty::TyExpressionVariant::StorageReassignment(storage_reassignment) => {
                storage_reassignment.parse(ctx);
            }
            ty::TyExpressionVariant::Return(exp) => exp.parse(ctx),
        }
    }
}

impl Parse for ty::TyVariableDecl {
    fn parse(&self, ctx: &ParseContext) {
        if let Some(mut token) = ctx
            .tokens
            .try_get_mut(&to_ident_key(&self.name))
            .try_unwrap()
        {
            token.typed = Some(TypedAstToken::TypedDeclaration(ty::TyDecl::VariableDecl(
                Box::new(self.clone()),
            )));
            token.type_def = Some(TypeDefinition::Ident(self.name.clone()));
        }
        if let Some(call_path_tree) = &self.type_ascription.call_path_tree {
            collect_call_path_tree(ctx, call_path_tree, &self.type_ascription);
        }
        self.body.parse(ctx);
    }
}

impl Parse for ty::ConstantDecl {
    fn parse(&self, ctx: &ParseContext) {
        let const_decl = ctx.engines.de().get_constant(&self.decl_id);
        collect_const_decl(ctx, &const_decl, &self.decl_span);
    }
}

impl Parse for ty::FunctionDecl {
    fn parse(&self, ctx: &ParseContext) {
        let func_decl = ctx.engines.de().get_function(&self.decl_id);
        let typed_token = TypedAstToken::TypedFunctionDeclaration(func_decl.clone());
        if let Some(mut token) = ctx
            .tokens
            .try_get_mut(&to_ident_key(&func_decl.name))
            .try_unwrap()
        {
            token.typed = Some(typed_token.clone());
            token.type_def = Some(TypeDefinition::Ident(func_decl.name.clone()));
        }
        func_decl
            .body
            .contents
            .iter()
            .for_each(|node| node.parse(ctx));
        func_decl
            .parameters
            .iter()
            .for_each(|param| param.parse(ctx));
        func_decl.type_parameters.iter().for_each(|type_param| {
            collect_type_id(
                ctx,
                type_param.type_id,
                &typed_token,
                type_param.name_ident.span(),
            );
        });
        collect_type_argument(ctx, &func_decl.return_type);
        for (ident, trait_constraints) in &func_decl.where_clause {
            trait_constraints.iter().for_each(|constraint| {
                collect_trait_constraint(ctx, constraint);
            });
            if let Some(mut token) = ctx.tokens.try_get_mut(&to_ident_key(ident)).try_unwrap() {
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
}

impl Parse for ty::TraitDecl {
    fn parse(&self, ctx: &ParseContext) {
        let trait_decl = ctx.engines.de().get_trait(&self.decl_id);
        if let Some(mut token) = ctx
            .tokens
            .try_get_mut(&to_ident_key(&trait_decl.name))
            .try_unwrap()
        {
            token.typed = Some(TypedAstToken::TypedDeclaration(ty::TyDecl::TraitDecl(
                self.clone(),
            )));
            token.type_def = Some(TypeDefinition::Ident(trait_decl.name.clone()));
        }
        trait_decl
            .interface_surface
            .iter()
            .for_each(|item| match item {
                ty::TyTraitInterfaceItem::TraitFn(trait_fn_decl_ref) => {
                    let trait_fn = ctx.engines.de().get_trait_fn(trait_fn_decl_ref);
                    trait_fn.parse(ctx);
                }
                ty::TyTraitInterfaceItem::Constant(decl_ref) => {
                    let constant = ctx.engines.de().get_constant(decl_ref);
                    collect_const_decl(ctx, &constant, &decl_ref.span());
                }
            });
        trait_decl.supertraits.iter().for_each(|supertrait| {
            collect_supertrait(ctx, supertrait);
        });
    }
}

impl Parse for ty::StructDecl {
    fn parse(&self, ctx: &ParseContext) {
        let struct_decl = ctx.engines.de().get_struct(&self.decl_id);
        if let Some(mut token) = ctx
            .tokens
            .try_get_mut(&to_ident_key(&struct_decl.call_path.suffix))
            .try_unwrap()
        {
            token.typed = Some(TypedAstToken::TypedDeclaration(ty::TyDecl::StructDecl(
                self.clone(),
            )));
            token.type_def = Some(TypeDefinition::Ident(struct_decl.call_path.suffix));
        }
        struct_decl.fields.iter().for_each(|field| {
            field.parse(ctx);
        });
        struct_decl.type_parameters.iter().for_each(|type_param| {
            if let Some(mut token) = ctx
                .tokens
                .try_get_mut(&to_ident_key(&type_param.name_ident))
                .try_unwrap()
            {
                token.typed = Some(TypedAstToken::TypedParameter(type_param.clone()));
                token.type_def = Some(TypeDefinition::TypeId(type_param.type_id));
            }
        });
    }
}

impl Parse for ty::ImplTrait {
    fn parse(&self, ctx: &ParseContext) {
        let ty::TyImplTrait {
            impl_type_parameters,
            trait_name,
            trait_type_arguments,
            trait_decl_ref,
            items,
            implementing_for,
            ..
        } = ctx.engines.de().get_impl_trait(&self.decl_id);
        impl_type_parameters.iter().for_each(|param| {
            collect_type_id(
                ctx,
                param.type_id,
                &TypedAstToken::TypedParameter(param.clone()),
                param.name_ident.span(),
            );
        });
        trait_name.prefixes.iter().for_each(|ident| {
            if let Some(mut token) = ctx.tokens.try_get_mut(&to_ident_key(ident)).try_unwrap() {
                token.typed = Some(TypedAstToken::Ident(ident.clone()));
            }
        });

        // Which typed token should be used for collect_type_id
        // if trait_decl_ref is some, then our ImplTrait is for an ABI or Trait. In this instance,
        // we want to use the TypedArgument(implementing_for) type as the typed token.
        //
        // Otherwise, we use the TypedDeclaration(declaration.clone()) type as the typed token.
        let mut typed_token = None;
        if let Some(mut token) = ctx
            .tokens
            .try_get_mut(&to_ident_key(&trait_name.suffix))
            .try_unwrap()
        {
            token.typed = Some(TypedAstToken::TypedDeclaration(ty::TyDecl::ImplTrait(
                self.clone(),
            )));
            token.type_def = if let Some(decl_ref) = &trait_decl_ref {
                typed_token = Some(TypedAstToken::TypedArgument(implementing_for.clone()));
                match &decl_ref.id().clone() {
                    InterfaceDeclId::Abi(decl_id) => {
                        let abi_decl = ctx.engines.de().get_abi(decl_id);
                        Some(TypeDefinition::Ident(abi_decl.name))
                    }
                    InterfaceDeclId::Trait(decl_id) => {
                        let trait_decl = ctx.engines.de().get_trait(decl_id);
                        Some(TypeDefinition::Ident(trait_decl.name))
                    }
                }
            } else {
                typed_token = token.typed.clone();
                Some(TypeDefinition::TypeId(implementing_for.type_id))
            };
        }
        trait_type_arguments.iter().for_each(|type_arg| {
            collect_type_argument(ctx, type_arg);
        });
        items.iter().for_each(|item| match item {
            ty::TyTraitItem::Fn(method_ref) => {
                let method = ctx.engines.de().get_function(method_ref);
                method.parse(ctx);
            }
            ty::TyTraitItem::Constant(const_ref) => {
                let constant = ctx.engines.de().get_constant(const_ref);
                collect_const_decl(ctx, &constant, &const_ref.span());
            }
        });
        collect_type_argument(ctx, &implementing_for);

        // collect the root type argument again with declaration info this time so the
        // impl is registered
        if let Some(typed_token) = typed_token {
            collect_type_id(
                ctx,
                implementing_for.type_id,
                &typed_token,
                implementing_for
                    .call_path_tree
                    .as_ref()
                    .map(|tree| tree.call_path.suffix.span())
                    .unwrap_or(implementing_for.span()),
            );
        }
    }
}

impl Parse for ty::AbiDecl {
    fn parse(&self, ctx: &ParseContext) {
        let abi_decl = ctx.engines.de().get_abi(&self.decl_id);
        if let Some(mut token) = ctx
            .tokens
            .try_get_mut(&to_ident_key(&abi_decl.name))
            .try_unwrap()
        {
            token.typed = Some(TypedAstToken::TypedDeclaration(ty::TyDecl::AbiDecl(
                self.clone(),
            )));
            token.type_def = Some(TypeDefinition::Ident(abi_decl.name.clone()));
        }
        abi_decl
            .interface_surface
            .iter()
            .for_each(|item| match item {
                ty::TyTraitInterfaceItem::TraitFn(trait_fn_decl_ref) => {
                    let trait_fn = ctx.engines.de().get_trait_fn(trait_fn_decl_ref);
                    trait_fn.parse(ctx);
                }
                ty::TyTraitInterfaceItem::Constant(const_ref) => {
                    let constant = ctx.engines.de().get_constant(const_ref);
                    collect_const_decl(ctx, &constant, &const_ref.span());
                }
            });
        abi_decl.supertraits.iter().for_each(|supertrait| {
            supertrait.parse(ctx);
        });
    }
}

impl Parse for ty::GenericTypeForFunctionScope {
    fn parse(&self, ctx: &ParseContext) {
        if let Some(mut token) = ctx
            .tokens
            .try_get_mut(&to_ident_key(&self.name))
            .try_unwrap()
        {
            token.typed = Some(TypedAstToken::TypedDeclaration(
                ty::TyDecl::GenericTypeForFunctionScope(self.clone()),
            ));
            token.type_def = Some(TypeDefinition::TypeId(self.type_id));
        }
    }
}

impl Parse for ty::StorageDecl {
    fn parse(&self, ctx: &ParseContext) {
        let storage_decl = ctx.engines.de().get_storage(&self.decl_id);
        for field in &storage_decl.fields {
            if let Some(mut token) = ctx
                .tokens
                .try_get_mut(&to_ident_key(&field.name))
                .try_unwrap()
            {
                token.typed = Some(TypedAstToken::TypedStorageField(field.clone()));
                token.type_def = Some(TypeDefinition::Ident(field.name.clone()));
            }
            collect_type_argument(ctx, &field.type_argument);
            field.initializer.parse(ctx);
        }
    }
}

impl Parse for ty::TypeAliasDecl {
    fn parse(&self, ctx: &ParseContext) {
        let type_alias_decl = ctx.engines.de().get_type_alias(&self.decl_id);
        type_alias_decl.parse(ctx);
    }
}

impl Parse for ty::TyFunctionParameter {
    fn parse(&self, ctx: &ParseContext) {
        let typed_token = TypedAstToken::TypedFunctionParameter(self.clone());
        if let Some(mut token) = ctx
            .tokens
            .try_get_mut(&to_ident_key(&self.name))
            .try_unwrap()
        {
            token.typed = Some(typed_token);
            token.type_def = Some(TypeDefinition::TypeId(self.type_argument.type_id));
        }
        collect_type_argument(ctx, &self.type_argument);
    }
}

impl Parse for ty::TyTraitFn {
    fn parse(&self, ctx: &ParseContext) {
        if let Some(mut token) = ctx
            .tokens
            .try_get_mut(&to_ident_key(&self.name))
            .try_unwrap()
        {
            token.typed = Some(TypedAstToken::TypedTraitFn(self.clone()));
            token.type_def = Some(TypeDefinition::Ident(self.name.clone()));
        }
        self.parameters.iter().for_each(|param| param.parse(ctx));
        let return_ident = Ident::new(self.return_type_span.clone());
        if let Some(mut token) = ctx
            .tokens
            .try_get_mut(&to_ident_key(&return_ident))
            .try_unwrap()
        {
            token.typed = Some(TypedAstToken::TypedTraitFn(self.clone()));
            token.type_def = Some(TypeDefinition::TypeId(self.return_type));
        }
    }
}

impl Parse for ty::TyStructField {
    fn parse(&self, ctx: &ParseContext) {
        if let Some(mut token) = ctx
            .tokens
            .try_get_mut(&to_ident_key(&self.name))
            .try_unwrap()
        {
            token.typed = Some(TypedAstToken::TypedStructField(self.clone()));
            token.type_def = Some(TypeDefinition::TypeId(self.type_argument.type_id));
        }
        collect_type_argument(ctx, &self.type_argument);
    }
}

impl Parse for ty::TyEnumVariant {
    fn parse(&self, ctx: &ParseContext) {
        let typed_token = TypedAstToken::TypedEnumVariant(self.clone());
        if let Some(mut token) = ctx
            .tokens
            .try_get_mut(&to_ident_key(&self.name))
            .try_unwrap()
        {
            token.typed = Some(typed_token);
            token.type_def = Some(TypeDefinition::TypeId(self.type_argument.type_id));
        }
        collect_type_argument(ctx, &self.type_argument);
    }
}

impl Parse for ty::TyFunctionDecl {
    fn parse(&self, ctx: &ParseContext) {
        let typed_token = TypedAstToken::TypedFunctionDeclaration(self.clone());
        if let Some(mut token) = ctx
            .tokens
            .try_get_mut(&to_ident_key(&self.name))
            .try_unwrap()
        {
            token.typed = Some(typed_token.clone());
            token.type_def = Some(TypeDefinition::Ident(self.name.clone()));
        }
        self.body.contents.iter().for_each(|node| node.parse(ctx));
        self.parameters.iter().for_each(|param| param.parse(ctx));
        self.type_parameters.iter().for_each(|type_param| {
            collect_type_id(
                ctx,
                type_param.type_id,
                &typed_token,
                type_param.name_ident.span(),
            );
        });
        collect_type_argument(ctx, &self.return_type);
        for (ident, trait_constraints) in &self.where_clause {
            trait_constraints.iter().for_each(|constraint| {
                collect_trait_constraint(ctx, constraint);
            });
            if let Some(mut token) = ctx.tokens.try_get_mut(&to_ident_key(ident)).try_unwrap() {
                token.typed = Some(typed_token.clone());
                if let Some(param_decl_ident) = self
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
}

impl Parse for ty::TyTypeAliasDecl {
    fn parse(&self, ctx: &ParseContext) {
        if let Some(mut token) = ctx
            .tokens
            .try_get_mut(&to_ident_key(&self.name))
            .try_unwrap()
        {
            token.typed = Some(TypedAstToken::TypedTypeAliasDeclaration(self.clone()));
            token.type_def = Some(TypeDefinition::Ident(self.name.clone()));
        }
        collect_type_argument(ctx, &self.ty);
    }
}

impl Parse for ty::TyIntrinsicFunctionKind {
    fn parse(&self, ctx: &ParseContext) {
        self.type_arguments.iter().for_each(|type_arg| {
            collect_type_argument(ctx, type_arg);
        });
        self.arguments.iter().for_each(|arg| {
            arg.parse(ctx);
        });
    }
}

impl Parse for ty::TyScrutinee {
    fn parse(&self, ctx: &ParseContext) {
        use ty::TyScrutineeVariant::*;
        match &self.variant {
            CatchAll => {}
            Constant(name, _, const_decl) => {
                if let Some(mut token) = ctx.tokens.try_get_mut(&to_ident_key(name)).try_unwrap() {
                    token.typed = Some(TypedAstToken::TypedScrutinee(self.clone()));
                    token.type_def =
                        Some(TypeDefinition::Ident(const_decl.call_path.suffix.clone()));
                }
            }
            Literal(_) => {
                if let Some(mut token) = ctx
                    .tokens
                    .try_get_mut(&to_ident_key(&Ident::new(self.span.clone())))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedScrutinee(self.clone()));
                }
            }
            Variable(ident) => {
                if let Some(mut token) = ctx.tokens.try_get_mut(&to_ident_key(ident)).try_unwrap() {
                    token.typed = Some(TypedAstToken::TypedScrutinee(self.clone()));
                }
            }
            StructScrutinee {
                struct_ref,
                fields,
                instantiation_call_path,
            } => {
                if let Some(mut token) = ctx
                    .tokens
                    .try_get_mut(&to_ident_key(&instantiation_call_path.suffix))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedScrutinee(self.clone()));
                    token.type_def = Some(TypeDefinition::Ident(struct_ref.name().clone()));
                }
                fields.iter().for_each(|field| field.parse(ctx));
            }
            EnumScrutinee {
                enum_ref,
                variant,
                value,
                instantiation_call_path,
                call_path_decl: _,
            } => {
                let prefixes =
                    if let Some((last, prefixes)) = instantiation_call_path.prefixes.split_last() {
                        // the last prefix of the call path is not a module but a type
                        if let Some(mut token) =
                            ctx.tokens.try_get_mut(&to_ident_key(last)).try_unwrap()
                        {
                            token.typed = Some(TypedAstToken::TypedScrutinee(self.clone()));
                            token.type_def = Some(TypeDefinition::Ident(enum_ref.name().clone()));
                        }
                        prefixes
                    } else {
                        &instantiation_call_path.prefixes
                    };
                collect_call_path_prefixes(ctx, prefixes);
                if let Some(mut token) = ctx
                    .tokens
                    .try_get_mut(&to_ident_key(&instantiation_call_path.suffix))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedScrutinee(self.clone()));
                    token.type_def = Some(TypeDefinition::Ident(variant.name.clone()));
                }
                value.parse(ctx);
            }
            Tuple(scrutinees) | Or(scrutinees) => {
                scrutinees.iter().for_each(|s| s.parse(ctx));
            }
        }
    }
}

impl Parse for ty::TyStructScrutineeField {
    fn parse(&self, ctx: &ParseContext) {
        if let Some(mut token) = ctx
            .tokens
            .try_get_mut(&to_ident_key(&self.field))
            .try_unwrap()
        {
            token.typed = Some(TypedAstToken::TyStructScrutineeField(self.clone()));
            token.type_def = Some(TypeDefinition::Ident(self.field_def_name.clone()));
        }
        if let Some(scrutinee) = &self.scrutinee {
            scrutinee.parse(ctx);
        }
    }
}

impl Parse for ty::TyReassignment {
    fn parse(&self, ctx: &ParseContext) {
        self.rhs.parse(ctx);
        if let Some(mut token) = ctx
            .tokens
            .try_get_mut(&to_ident_key(&self.lhs_base_name))
            .try_unwrap()
        {
            token.typed = Some(TypedAstToken::TypedReassignment(self.clone()));
        }
        self.lhs_indices.iter().for_each(|proj_kind| {
            if let ty::ProjectionKind::StructField { name } = proj_kind {
                if let Some(mut token) = ctx.tokens.try_get_mut(&to_ident_key(name)).try_unwrap() {
                    token.typed = Some(TypedAstToken::TypedReassignment(self.clone()));
                    if let Some(struct_decl) = &ctx
                        .tokens
                        .struct_declaration_of_type_id(ctx.engines, &self.lhs_type)
                    {
                        struct_decl.fields.iter().for_each(|decl_field| {
                            if &decl_field.name == name {
                                token.type_def =
                                    Some(TypeDefinition::Ident(decl_field.name.clone()));
                            }
                        });
                    }
                }
            }
        });
    }
}

impl Parse for ty::TyStorageReassignment {
    fn parse(&self, ctx: &ParseContext) {
        // collect storage keyword
        if let Some(mut token) = ctx
            .tokens
            .try_get_mut(&to_ident_key(&Ident::new(
                self.storage_keyword_span.clone(),
            )))
            .try_unwrap()
        {
            token.typed = Some(TypedAstToken::TyStorageResassignment(Box::new(
                self.clone(),
            )));
            if let Some(storage) = ctx.namespace.get_declared_storage(ctx.engines.de()) {
                token.type_def = Some(TypeDefinition::Ident(storage.storage_keyword));
            }
        }
        if let Some((head_field, tail_fields)) = self.fields.split_first() {
            // collect the first ident as a field of the storage definition
            if let Some(mut token) = ctx
                .tokens
                .try_get_mut(&to_ident_key(&head_field.name))
                .try_unwrap()
            {
                token.typed = Some(TypedAstToken::TypeCheckedStorageReassignDescriptor(
                    head_field.clone(),
                ));
                if let Some(storage_field) = ctx
                    .namespace
                    .get_declared_storage(ctx.engines.de())
                    .and_then(|storage| {
                        // find the corresponding field in the storage declaration
                        storage
                            .fields
                            .into_iter()
                            .find(|f| f.name.as_str() == head_field.name.as_str())
                    })
                {
                    token.type_def = Some(TypeDefinition::Ident(storage_field.name));
                }
            }
            // collect the rest of the idents as fields of their respective types
            for (field, container_type_id) in tail_fields
                .iter()
                .zip(self.fields.iter().map(|f| f.type_id))
            {
                if let Some(mut token) = ctx
                    .tokens
                    .try_get_mut(&to_ident_key(&field.name))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypeCheckedStorageReassignDescriptor(
                        field.clone(),
                    ));
                    match ctx.engines.te().get(container_type_id) {
                        TypeInfo::Struct(decl_ref) => {
                            if let Some(field_name) = ctx
                                .engines
                                .de()
                                .get_struct(&decl_ref)
                                .fields
                                .iter()
                                .find(|struct_field| {
                                    // find the corresponding field in the containing type declaration
                                    struct_field.name.as_str() == field.name.as_str()
                                })
                                .map(|struct_field| struct_field.name.clone())
                            {
                                token.type_def = Some(TypeDefinition::Ident(field_name));
                            }
                        }
                        _ => {
                            token.type_def = Some(TypeDefinition::TypeId(field.type_id));
                        }
                    }
                }
            }
        }
        self.rhs.parse(ctx);
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

fn collect_call_path_tree(ctx: &ParseContext, tree: &CallPathTree, type_arg: &TypeArgument) {
    let type_info = ctx.engines.te().get(type_arg.type_id);
    collect_call_path_prefixes(ctx, &tree.call_path.prefixes);
    collect_type_id(
        ctx,
        type_arg.type_id,
        &TypedAstToken::TypedArgument(type_arg.clone()),
        tree.call_path.suffix.span(),
    );
    match &type_info {
        TypeInfo::Enum(decl_ref) => {
            let decl = ctx.engines.de().get_enum(decl_ref);
            let child_type_args = decl.type_parameters.iter().map(TypeArgument::from);
            for (child_tree, type_arg) in tree.children.iter().zip(child_type_args) {
                collect_call_path_tree(ctx, child_tree, &type_arg);
            }
        }
        TypeInfo::Struct(decl_ref) => {
            let decl = ctx.engines.de().get_struct(decl_ref);
            let child_type_args = decl.type_parameters.iter().map(TypeArgument::from);
            for (child_tree, type_arg) in tree.children.iter().zip(child_type_args) {
                collect_call_path_tree(ctx, child_tree, &type_arg);
            }
        }
        TypeInfo::Custom {
            type_arguments: Some(type_args),
            ..
        } => {
            for (child_tree, type_arg) in tree.children.iter().zip(type_args.iter()) {
                collect_call_path_tree(ctx, child_tree, type_arg);
            }
        }
        TypeInfo::ContractCaller { .. } => {
            // single generic argument to ContractCaller<_> has to be a single ABI
            // definition call path which we can collect without recursion
            if let Some(child_tree) = tree.children.first() {
                let abi_call_path = &child_tree.call_path;
                collect_call_path_prefixes(ctx, &abi_call_path.prefixes);
                if let Some(mut token) = ctx
                    .tokens
                    .try_get_mut(&to_ident_key(&abi_call_path.suffix))
                    .try_unwrap()
                {
                    token.typed = Some(TypedAstToken::TypedArgument(type_arg.clone()));
                    if let Some(abi_def_ident) = ctx
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

fn collect_call_path_prefixes(ctx: &ParseContext, prefixes: &[Ident]) {
    for (mod_path, ident) in iter_prefixes(prefixes).zip(prefixes) {
        if let Some(mut token) = ctx.tokens.try_get_mut(&to_ident_key(ident)).try_unwrap() {
            token.typed = Some(TypedAstToken::Ident(ident.clone()));
            if let Some(span) = ctx
                .namespace
                .submodule(mod_path)
                .and_then(|tgt_submod| tgt_submod.span.clone())
            {
                token.kind = SymbolKind::Module;
                token.type_def = Some(TypeDefinition::Ident(Ident::new(span)));
            }
        }
    }
}

fn collect_const_decl(ctx: &ParseContext, const_decl: &ty::TyConstantDecl, span: &Span) {
    if let Some(mut token) = ctx
        .tokens
        .try_get_mut(&to_ident_key(&Ident::new(span.clone())))
        .try_unwrap()
    {
        token.typed = Some(TypedAstToken::TypedConstantDeclaration(const_decl.clone()));
        token.type_def = Some(TypeDefinition::Ident(const_decl.call_path.suffix.clone()));
    }
    if let Some(call_path_tree) = &const_decl.type_ascription.call_path_tree {
        collect_call_path_tree(ctx, call_path_tree, &const_decl.type_ascription);
    }
    if let Some(value) = &const_decl.value {
        value.parse(ctx);
    }
}

fn collect_type_id(
    ctx: &ParseContext,
    type_id: TypeId,
    typed_token: &TypedAstToken,
    type_span: Span,
) {
    let type_info = ctx.engines.te().get(type_id);
    let symbol_kind = type_info_to_symbol_kind(ctx.engines.te(), &type_info, Some(&type_span));
    match &type_info {
        TypeInfo::Array(type_arg, ..) => {
            collect_type_argument(ctx, type_arg);
        }
        TypeInfo::Tuple(type_arguments) => {
            type_arguments.iter().for_each(|type_arg| {
                collect_type_argument(ctx, type_arg);
            });
        }
        TypeInfo::Enum(decl_ref) => {
            let decl = ctx.engines.de().get_enum(decl_ref);
            if let Some(token) = ctx
                .tokens
                .try_get_mut(&to_ident_key(&Ident::new(type_span)))
                .try_unwrap()
            {
                assign_type_to_token(token, symbol_kind, typed_token.clone(), type_id);
            }
            decl.type_parameters.iter().for_each(|param| {
                collect_type_id(
                    ctx,
                    param.type_id,
                    &TypedAstToken::TypedParameter(param.clone()),
                    param.name_ident.span(),
                );
            });
            decl.variants.iter().for_each(|variant| {
                variant.parse(ctx);
            });
        }
        TypeInfo::Struct(decl_ref) => {
            let decl = ctx.engines.de().get_struct(decl_ref);
            if let Some(token) = ctx
                .tokens
                .try_get_mut(&to_ident_key(&Ident::new(type_span)))
                .try_unwrap()
            {
                assign_type_to_token(token, symbol_kind, typed_token.clone(), type_id);
            }
            decl.type_parameters.iter().for_each(|param| {
                collect_type_id(
                    ctx,
                    param.type_id,
                    &TypedAstToken::TypedParameter(param.clone()),
                    param.name_ident.span(),
                );
            });
            decl.fields.iter().for_each(|field| {
                field.parse(ctx);
            });
        }
        TypeInfo::Custom {
            type_arguments,
            call_path: name,
        } => {
            if let Some(token) = ctx
                .tokens
                .try_get_mut(&to_ident_key(&Ident::new(name.span())))
                .try_unwrap()
            {
                assign_type_to_token(token, symbol_kind, typed_token.clone(), type_id);
            }
            if let Some(type_arguments) = type_arguments {
                for type_arg in type_arguments {
                    collect_type_argument(ctx, type_arg);
                }
            }
        }
        TypeInfo::Storage { fields } => {
            fields.iter().for_each(|field| {
                field.parse(ctx);
            });
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

fn collect_type_argument(ctx: &ParseContext, type_arg: &TypeArgument) {
    if let Some(call_path_tree) = &type_arg.call_path_tree {
        collect_call_path_tree(ctx, call_path_tree, type_arg);
    } else {
        collect_type_id(
            ctx,
            type_arg.type_id,
            &TypedAstToken::TypedArgument(type_arg.clone()),
            type_arg.span(),
        );
    }
}

fn collect_trait_constraint(
    ctx: &ParseContext,
    trait_constraint @ TraitConstraint {
        trait_name,
        type_arguments,
    }: &TraitConstraint,
) {
    collect_call_path_prefixes(ctx, &trait_name.prefixes);
    if let Some(mut token) = ctx
        .tokens
        .try_get_mut(&to_ident_key(&trait_name.suffix))
        .try_unwrap()
    {
        token.typed = Some(TypedAstToken::TypedTraitConstraint(
            trait_constraint.clone(),
        ));
        if let Some(trait_def_ident) = ctx
            .namespace
            .submodule(&trait_name.prefixes)
            .and_then(|module| module.symbols().get(&trait_name.suffix))
            .and_then(|decl| decl.get_decl_ident())
        {
            token.type_def = Some(TypeDefinition::Ident(trait_def_ident));
        }
    }
    type_arguments.iter().for_each(|type_arg| {
        collect_type_argument(ctx, type_arg);
    });
}

fn collect_supertrait(ctx: &ParseContext, supertrait: &Supertrait) {
    if let Some(mut token) = ctx
        .tokens
        .try_get_mut(&to_ident_key(&supertrait.name.suffix))
        .try_unwrap()
    {
        token.typed = Some(TypedAstToken::TypedSupertrait(supertrait.clone()));
        token.type_def = if let Some(decl_ref) = &supertrait.decl_ref {
            let trait_decl = ctx.engines.de().get_trait(decl_ref);
            Some(TypeDefinition::Ident(trait_decl.name))
        } else {
            Some(TypeDefinition::Ident(supertrait.name.suffix.clone()))
        }
    }
}

fn collect_enum(ctx: &ParseContext, decl_id: &DeclId<ty::TyEnumDecl>, declaration: &ty::TyDecl) {
    let enum_decl = ctx.engines.de().get_enum(decl_id);
    if let Some(mut token) = ctx
        .tokens
        .try_get_mut(&to_ident_key(&enum_decl.call_path.suffix))
        .try_unwrap()
    {
        token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
        token.type_def = Some(TypeDefinition::Ident(enum_decl.call_path.suffix.clone()));
    }
    enum_decl.type_parameters.iter().for_each(|type_param| {
        if let Some(mut token) = ctx
            .tokens
            .try_get_mut(&to_ident_key(&type_param.name_ident))
            .try_unwrap()
        {
            token.typed = Some(TypedAstToken::TypedParameter(type_param.clone()));
            token.type_def = Some(TypeDefinition::TypeId(type_param.type_id));
        }
    });
    enum_decl.variants.iter().for_each(|variant| {
        variant.parse(ctx);
    });
}

fn iter_prefixes<T>(slice: &[T]) -> impl Iterator<Item = &[T]> + DoubleEndedIterator {
    (1..=slice.len()).map(move |len| &slice[..len])
}

use sway_error::handler::Handler;

use crate::{
    ast_elements::binding::SymbolResolveTypeBinding,
    decl_engine::{parsed_engine::ParsedDeclEngineReplace, parsed_id::ParsedDeclId},
    language::{
        parsed::{
            AbiDeclaration, ArrayExpression, AstNode, AstNodeContent, CodeBlock,
            ConfigurableDeclaration, ConstantDeclaration, Declaration, EnumDeclaration,
            EnumVariant, Expression, ExpressionKind, FunctionDeclaration, FunctionParameter,
            ImplItem, ImplSelfOrTrait, ParseModule, ParseProgram, ReassignmentTarget, Scrutinee,
            StorageDeclaration, StorageEntry, StructDeclaration, StructExpressionField,
            StructField, StructScrutineeField, Supertrait, TraitDeclaration, TraitFn, TraitItem,
            TraitTypeDeclaration, TypeAliasDeclaration, VariableDeclaration,
        },
        CallPath, CallPathTree, ResolvedCallPath,
    },
    GenericArgument, TraitConstraint, TypeBinding, TypeParameter,
};

use super::symbol_resolve_context::SymbolResolveContext;

pub trait ResolveSymbols {
    fn resolve_symbols(&mut self, handler: &Handler, ctx: SymbolResolveContext);
}

impl ResolveSymbols for ParseProgram {
    fn resolve_symbols(&mut self, handler: &Handler, mut ctx: SymbolResolveContext) {
        let ParseProgram { root, .. } = self;
        root.resolve_symbols(handler, ctx.by_ref());
    }
}

impl ResolveSymbols for ParseModule {
    fn resolve_symbols(&mut self, handler: &Handler, mut ctx: SymbolResolveContext) {
        let ParseModule {
            submodules,
            tree,
            module_eval_order,
            attributes: _,
            span: _,
            hash: _,
            ..
        } = self;

        // Analyze submodules first in order of evaluation previously computed by the dependency graph.
        module_eval_order.iter().for_each(|eval_mod_name| {
            let (_name, submodule) = submodules
                .iter_mut()
                .find(|(submod_name, _submodule)| eval_mod_name == submod_name)
                .unwrap();
            submodule.module.resolve_symbols(handler, ctx.by_ref());
        });

        tree.root_nodes
            .iter_mut()
            .for_each(|node| node.resolve_symbols(handler, ctx.by_ref()))
    }
}

impl ResolveSymbols for AstNode {
    fn resolve_symbols(&mut self, handler: &Handler, ctx: SymbolResolveContext) {
        match &mut self.content {
            AstNodeContent::UseStatement(_) => {}
            AstNodeContent::Declaration(decl) => decl.resolve_symbols(handler, ctx),
            AstNodeContent::Expression(expr) => expr.resolve_symbols(handler, ctx),
            AstNodeContent::IncludeStatement(_) => {}
            AstNodeContent::Error(_, _) => {}
        }
    }
}

impl ResolveSymbols for Declaration {
    fn resolve_symbols(&mut self, handler: &Handler, ctx: SymbolResolveContext) {
        match self {
            Declaration::VariableDeclaration(decl_id) => decl_id.resolve_symbols(handler, ctx),
            Declaration::FunctionDeclaration(decl_id) => decl_id.resolve_symbols(handler, ctx),
            Declaration::TraitDeclaration(decl_id) => decl_id.resolve_symbols(handler, ctx),
            Declaration::StructDeclaration(decl_id) => decl_id.resolve_symbols(handler, ctx),
            Declaration::EnumDeclaration(decl_id) => decl_id.resolve_symbols(handler, ctx),
            Declaration::EnumVariantDeclaration(_decl) => unreachable!(),
            Declaration::ImplSelfOrTrait(decl_id) => decl_id.resolve_symbols(handler, ctx),
            Declaration::AbiDeclaration(decl_id) => decl_id.resolve_symbols(handler, ctx),
            Declaration::ConstantDeclaration(decl_id) => decl_id.resolve_symbols(handler, ctx),
            Declaration::StorageDeclaration(decl_id) => decl_id.resolve_symbols(handler, ctx),
            Declaration::TypeAliasDeclaration(decl_id) => decl_id.resolve_symbols(handler, ctx),
            Declaration::TraitTypeDeclaration(decl_id) => decl_id.resolve_symbols(handler, ctx),
            Declaration::TraitFnDeclaration(decl_id) => decl_id.resolve_symbols(handler, ctx),
            Declaration::ConfigurableDeclaration(decl_id) => decl_id.resolve_symbols(handler, ctx),
            Declaration::ConstGenericDeclaration(_) => {
                todo!("Will be implemented by https://github.com/FuelLabs/sway/issues/6860")
            }
        }
    }
}

impl ResolveSymbols for ParsedDeclId<VariableDeclaration> {
    fn resolve_symbols(&mut self, handler: &Handler, ctx: SymbolResolveContext) {
        let pe = ctx.engines().pe();
        let mut var_decl = pe.get_variable(self).as_ref().clone();
        var_decl.body.resolve_symbols(handler, ctx);
        pe.replace(*self, var_decl);
    }
}

impl ResolveSymbols for ParsedDeclId<FunctionDeclaration> {
    fn resolve_symbols(&mut self, handler: &Handler, ctx: SymbolResolveContext) {
        let pe = ctx.engines().pe();
        let mut fn_decl = pe.get_function(self).as_ref().clone();
        fn_decl.body.resolve_symbols(handler, ctx);
        pe.replace(*self, fn_decl);
    }
}

impl ResolveSymbols for ParsedDeclId<TraitDeclaration> {
    fn resolve_symbols(&mut self, handler: &Handler, ctx: SymbolResolveContext) {
        let pe = ctx.engines().pe();
        let mut trait_decl = ctx.engines().pe().get_trait(self).as_ref().clone();
        trait_decl.resolve_symbols(handler, ctx);
        pe.replace(*self, trait_decl);
    }
}

impl ResolveSymbols for ParsedDeclId<StructDeclaration> {
    fn resolve_symbols(&mut self, handler: &Handler, ctx: SymbolResolveContext) {
        let pe = ctx.engines().pe();
        let mut struct_decl = ctx.engines().pe().get_struct(self).as_ref().clone();
        struct_decl.resolve_symbols(handler, ctx);
        pe.replace(*self, struct_decl);
    }
}

impl ResolveSymbols for ParsedDeclId<EnumDeclaration> {
    fn resolve_symbols(&mut self, handler: &Handler, ctx: SymbolResolveContext) {
        let pe = ctx.engines().pe();
        let mut enum_decl = ctx.engines().pe().get_enum(self).as_ref().clone();
        enum_decl.resolve_symbols(handler, ctx);
        pe.replace(*self, enum_decl);
    }
}

impl ResolveSymbols for ParsedDeclId<ConfigurableDeclaration> {
    fn resolve_symbols(&mut self, handler: &Handler, ctx: SymbolResolveContext) {
        let pe = ctx.engines().pe();
        let mut configurable_decl = ctx.engines().pe().get_configurable(self).as_ref().clone();
        configurable_decl.resolve_symbols(handler, ctx);
        pe.replace(*self, configurable_decl);
    }
}

impl ResolveSymbols for ParsedDeclId<ConstantDeclaration> {
    fn resolve_symbols(&mut self, handler: &Handler, ctx: SymbolResolveContext) {
        let pe = ctx.engines().pe();
        let mut constant_decl = ctx.engines().pe().get_constant(self).as_ref().clone();
        constant_decl.resolve_symbols(handler, ctx);
        pe.replace(*self, constant_decl);
    }
}

impl ResolveSymbols for ParsedDeclId<TraitTypeDeclaration> {
    fn resolve_symbols(&mut self, handler: &Handler, ctx: SymbolResolveContext) {
        let pe = ctx.engines().pe();
        let mut trait_type_decl = ctx.engines().pe().get_trait_type(self).as_ref().clone();
        trait_type_decl.resolve_symbols(handler, ctx);
        pe.replace(*self, trait_type_decl);
    }
}

impl ResolveSymbols for ParsedDeclId<TraitFn> {
    fn resolve_symbols(&mut self, handler: &Handler, ctx: SymbolResolveContext) {
        let pe = ctx.engines().pe();
        let mut trait_fn_decl = ctx.engines().pe().get_trait_fn(self).as_ref().clone();
        trait_fn_decl.resolve_symbols(handler, ctx);
        pe.replace(*self, trait_fn_decl);
    }
}

impl ResolveSymbols for ParsedDeclId<ImplSelfOrTrait> {
    fn resolve_symbols(&mut self, handler: &Handler, ctx: SymbolResolveContext) {
        let pe = ctx.engines().pe();
        let mut impl_self_or_trait = ctx
            .engines()
            .pe()
            .get_impl_self_or_trait(self)
            .as_ref()
            .clone();
        impl_self_or_trait.resolve_symbols(handler, ctx);
        pe.replace(*self, impl_self_or_trait);
    }
}

impl ResolveSymbols for ParsedDeclId<AbiDeclaration> {
    fn resolve_symbols(&mut self, handler: &Handler, ctx: SymbolResolveContext) {
        let pe = ctx.engines().pe();
        let mut abi_decl = ctx.engines().pe().get_abi(self).as_ref().clone();
        abi_decl.resolve_symbols(handler, ctx);
        pe.replace(*self, abi_decl);
    }
}

impl ResolveSymbols for ParsedDeclId<StorageDeclaration> {
    fn resolve_symbols(&mut self, handler: &Handler, ctx: SymbolResolveContext) {
        let pe = ctx.engines().pe();
        let mut storage_decl = ctx.engines().pe().get_storage(self).as_ref().clone();
        storage_decl.resolve_symbols(handler, ctx);
        pe.replace(*self, storage_decl);
    }
}

impl ResolveSymbols for ParsedDeclId<TypeAliasDeclaration> {
    fn resolve_symbols(&mut self, handler: &Handler, ctx: SymbolResolveContext) {
        let pe = ctx.engines().pe();
        let mut type_alias = ctx.engines().pe().get_type_alias(self).as_ref().clone();
        type_alias.resolve_symbols(handler, ctx);
        pe.replace(*self, type_alias);
    }
}

impl ResolveSymbols for ConfigurableDeclaration {
    fn resolve_symbols(&mut self, handler: &Handler, mut ctx: SymbolResolveContext) {
        self.type_ascription.resolve_symbols(handler, ctx.by_ref());
        if let Some(value) = self.value.as_mut() {
            value.resolve_symbols(handler, ctx.by_ref())
        }
    }
}

impl ResolveSymbols for ConstantDeclaration {
    fn resolve_symbols(&mut self, handler: &Handler, mut ctx: SymbolResolveContext) {
        self.type_ascription.resolve_symbols(handler, ctx.by_ref());
        if let Some(value) = self.value.as_mut() {
            value.resolve_symbols(handler, ctx.by_ref())
        }
    }
}

impl ResolveSymbols for StructDeclaration {
    fn resolve_symbols(&mut self, handler: &Handler, mut ctx: SymbolResolveContext) {
        self.type_parameters
            .iter_mut()
            .for_each(|tp| tp.resolve_symbols(handler, ctx.by_ref()));
        self.fields
            .iter_mut()
            .for_each(|f| f.resolve_symbols(handler, ctx.by_ref()));
    }
}

impl ResolveSymbols for StructField {
    fn resolve_symbols(&mut self, handler: &Handler, ctx: SymbolResolveContext) {
        self.type_argument.resolve_symbols(handler, ctx);
    }
}

impl ResolveSymbols for EnumDeclaration {
    fn resolve_symbols(&mut self, handler: &Handler, mut ctx: SymbolResolveContext) {
        self.type_parameters
            .iter_mut()
            .for_each(|tp| tp.resolve_symbols(handler, ctx.by_ref()));
        self.variants
            .iter_mut()
            .for_each(|f| f.resolve_symbols(handler, ctx.by_ref()));
    }
}

impl ResolveSymbols for EnumVariant {
    fn resolve_symbols(&mut self, handler: &Handler, ctx: SymbolResolveContext) {
        self.type_argument.resolve_symbols(handler, ctx);
    }
}

impl ResolveSymbols for TraitDeclaration {
    fn resolve_symbols(&mut self, handler: &Handler, mut ctx: SymbolResolveContext) {
        self.supertraits
            .iter_mut()
            .for_each(|st| st.resolve_symbols(handler, ctx.by_ref()));
        self.interface_surface
            .iter_mut()
            .for_each(|item| item.resolve_symbols(handler, ctx.by_ref()));
        self.methods
            .iter_mut()
            .for_each(|m| m.resolve_symbols(handler, ctx.by_ref()));
    }
}

impl ResolveSymbols for AbiDeclaration {
    fn resolve_symbols(&mut self, handler: &Handler, mut ctx: SymbolResolveContext) {
        self.supertraits
            .iter_mut()
            .for_each(|st| st.resolve_symbols(handler, ctx.by_ref()));
        self.interface_surface
            .iter_mut()
            .for_each(|item| item.resolve_symbols(handler, ctx.by_ref()));
        self.methods
            .iter_mut()
            .for_each(|m| m.resolve_symbols(handler, ctx.by_ref()));
    }
}

impl ResolveSymbols for TraitItem {
    fn resolve_symbols(&mut self, handler: &Handler, mut ctx: SymbolResolveContext) {
        match self {
            TraitItem::TraitFn(ref mut id) => id.resolve_symbols(handler, ctx.by_ref()),
            TraitItem::Constant(ref mut id) => id.resolve_symbols(handler, ctx.by_ref()),
            TraitItem::Type(ref mut id) => id.resolve_symbols(handler, ctx.by_ref()),
            TraitItem::Error(_, _) => {}
        }
    }
}

impl ResolveSymbols for TraitFn {
    fn resolve_symbols(&mut self, handler: &Handler, mut ctx: SymbolResolveContext) {
        self.parameters
            .iter_mut()
            .for_each(|f| f.resolve_symbols(handler, ctx.by_ref()));
        self.return_type.resolve_symbols(handler, ctx.by_ref());
    }
}

impl ResolveSymbols for Supertrait {
    fn resolve_symbols(&mut self, handler: &Handler, mut ctx: SymbolResolveContext) {
        self.name.resolve_symbols(handler, ctx.by_ref());
    }
}

impl ResolveSymbols for FunctionParameter {
    fn resolve_symbols(&mut self, handler: &Handler, mut ctx: SymbolResolveContext) {
        self.type_argument.resolve_symbols(handler, ctx.by_ref());
    }
}

impl ResolveSymbols for ImplSelfOrTrait {
    fn resolve_symbols(&mut self, handler: &Handler, mut ctx: SymbolResolveContext) {
        self.impl_type_parameters
            .iter_mut()
            .for_each(|f| f.resolve_symbols(handler, ctx.by_ref()));
        self.trait_name.resolve_symbols(handler, ctx.by_ref());
        self.trait_type_arguments
            .iter_mut()
            .for_each(|tp| tp.resolve_symbols(handler, ctx.by_ref()));
        self.implementing_for.resolve_symbols(handler, ctx.by_ref());
        self.items
            .iter_mut()
            .for_each(|tp| tp.resolve_symbols(handler, ctx.by_ref()));
    }
}

impl ResolveSymbols for ImplItem {
    fn resolve_symbols(&mut self, handler: &Handler, ctx: SymbolResolveContext) {
        match self {
            ImplItem::Fn(decl_id) => decl_id.resolve_symbols(handler, ctx),
            ImplItem::Constant(decl_id) => decl_id.resolve_symbols(handler, ctx),
            ImplItem::Type(decl_id) => decl_id.resolve_symbols(handler, ctx),
        }
    }
}

impl ResolveSymbols for TraitTypeDeclaration {
    fn resolve_symbols(&mut self, handler: &Handler, ctx: SymbolResolveContext) {
        if let Some(ty) = self.ty_opt.as_mut() {
            ty.resolve_symbols(handler, ctx)
        }
    }
}

impl ResolveSymbols for StorageDeclaration {
    fn resolve_symbols(&mut self, handler: &Handler, mut ctx: SymbolResolveContext) {
        self.entries
            .iter_mut()
            .for_each(|e| e.resolve_symbols(handler, ctx.by_ref()));
    }
}

impl ResolveSymbols for StorageEntry {
    fn resolve_symbols(&mut self, handler: &Handler, mut ctx: SymbolResolveContext) {
        match self {
            StorageEntry::Namespace(ref mut ns) => {
                ns.entries
                    .iter_mut()
                    .for_each(|e| e.resolve_symbols(handler, ctx.by_ref()));
            }
            StorageEntry::Field(ref mut f) => {
                f.type_argument.resolve_symbols(handler, ctx.by_ref());
                f.initializer.resolve_symbols(handler, ctx.by_ref());
            }
        }
    }
}

impl ResolveSymbols for TypeAliasDeclaration {
    fn resolve_symbols(&mut self, handler: &Handler, ctx: SymbolResolveContext) {
        self.ty.resolve_symbols(handler, ctx)
    }
}

impl ResolveSymbols for GenericArgument {
    fn resolve_symbols(&mut self, handler: &Handler, ctx: SymbolResolveContext) {
        if let Some(call_path) = self.as_type_argument_mut().unwrap().call_path_tree.as_mut() {
            call_path.resolve_symbols(handler, ctx);
        }
    }
}

impl ResolveSymbols for TypeParameter {
    fn resolve_symbols(&mut self, handler: &Handler, mut ctx: SymbolResolveContext) {
        match self {
            TypeParameter::Type(p) => p
                .trait_constraints
                .iter_mut()
                .for_each(|tc| tc.resolve_symbols(handler, ctx.by_ref())),
            TypeParameter::Const(_) => todo!(),
        }
    }
}

impl ResolveSymbols for TraitConstraint {
    fn resolve_symbols(&mut self, handler: &Handler, mut ctx: SymbolResolveContext) {
        self.trait_name.resolve_symbols(handler, ctx.by_ref());
        self.type_arguments
            .iter_mut()
            .for_each(|tc| tc.resolve_symbols(handler, ctx.by_ref()));
    }
}

impl ResolveSymbols for CallPath {
    fn resolve_symbols(&mut self, _handler: &Handler, _ctx: SymbolResolveContext) {}
}

impl ResolveSymbols for CallPathTree {
    fn resolve_symbols(&mut self, _handler: &Handler, _ctx: SymbolResolveContext) {}
}

impl ResolveSymbols for CodeBlock {
    fn resolve_symbols(&mut self, handler: &Handler, mut ctx: SymbolResolveContext) {
        for expr in self.contents.iter_mut() {
            expr.resolve_symbols(handler, ctx.by_ref())
        }
    }
}

impl ResolveSymbols for StructExpressionField {
    fn resolve_symbols(&mut self, handler: &Handler, ctx: SymbolResolveContext) {
        self.value.resolve_symbols(handler, ctx);
    }
}

impl ResolveSymbols for Scrutinee {
    fn resolve_symbols(&mut self, handler: &Handler, mut ctx: SymbolResolveContext) {
        match self {
            Scrutinee::Or {
                ref mut elems,
                span: _,
            } => elems
                .iter_mut()
                .for_each(|e| e.resolve_symbols(handler, ctx.by_ref())),
            Scrutinee::CatchAll { .. } => {}
            Scrutinee::Literal { .. } => {}
            Scrutinee::Variable { .. } => {}
            Scrutinee::AmbiguousSingleIdent(_) => {}
            Scrutinee::StructScrutinee {
                struct_name,
                fields,
                span: _,
            } => {
                struct_name.resolve_symbols(handler, ctx.by_ref());
                fields
                    .iter_mut()
                    .for_each(|f| f.resolve_symbols(handler, ctx.by_ref()))
            }
            Scrutinee::EnumScrutinee {
                call_path,
                value,
                span: _,
            } => {
                call_path.resolve_symbols(handler, ctx.by_ref());
                value.resolve_symbols(handler, ctx.by_ref());
            }
            Scrutinee::Tuple { elems, span: _ } => {
                elems
                    .iter_mut()
                    .for_each(|s| s.resolve_symbols(handler, ctx.by_ref()));
            }
            Scrutinee::Error { .. } => {}
        }
    }
}

impl ResolveSymbols for StructScrutineeField {
    fn resolve_symbols(&mut self, handler: &Handler, mut ctx: SymbolResolveContext) {
        match self {
            StructScrutineeField::Rest { .. } => {}
            StructScrutineeField::Field {
                field: _,
                scrutinee,
                span: _,
            } => {
                if let Some(scrutinee) = scrutinee.as_mut() {
                    scrutinee.resolve_symbols(handler, ctx.by_ref());
                }
            }
        }
    }
}

impl ResolveSymbols for Expression {
    fn resolve_symbols(&mut self, handler: &Handler, ctx: SymbolResolveContext) {
        self.kind.resolve_symbols(handler, ctx);
    }
}

impl ResolveSymbols for ExpressionKind {
    fn resolve_symbols(&mut self, handler: &Handler, mut ctx: SymbolResolveContext) {
        match self {
            ExpressionKind::Error(_, _) => {}
            ExpressionKind::Literal(_) => {}
            ExpressionKind::AmbiguousPathExpression(_) => {}
            ExpressionKind::FunctionApplication(expr) => {
                let result = SymbolResolveTypeBinding::resolve_symbol(
                    &mut expr.call_path_binding,
                    &Handler::default(),
                    ctx.by_ref(),
                );
                if let Ok(result) = result {
                    expr.resolved_call_path_binding = Some(TypeBinding::<
                        ResolvedCallPath<ParsedDeclId<FunctionDeclaration>>,
                    > {
                        inner: ResolvedCallPath {
                            decl: result,
                            unresolved_call_path: expr.call_path_binding.inner.clone(),
                        },
                        span: expr.call_path_binding.span.clone(),
                        type_arguments: expr.call_path_binding.type_arguments.clone(),
                    });
                }
                expr.arguments
                    .iter_mut()
                    .for_each(|a| a.resolve_symbols(handler, ctx.by_ref()))
            }
            ExpressionKind::LazyOperator(expr) => {
                expr.lhs.resolve_symbols(handler, ctx.by_ref());
                expr.rhs.resolve_symbols(handler, ctx.by_ref());
            }
            ExpressionKind::AmbiguousVariableExpression(_) => {}
            ExpressionKind::Variable(_) => {}
            ExpressionKind::Tuple(exprs) => {
                exprs
                    .iter_mut()
                    .for_each(|expr| expr.resolve_symbols(handler, ctx.by_ref()));
            }
            ExpressionKind::TupleIndex(expr) => {
                expr.prefix.resolve_symbols(handler, ctx.by_ref());
            }
            ExpressionKind::Array(ArrayExpression::Explicit { contents, .. }) => contents
                .iter_mut()
                .for_each(|e| e.resolve_symbols(handler, ctx.by_ref())),
            ExpressionKind::Array(ArrayExpression::Repeat { value, length }) => {
                value.resolve_symbols(handler, ctx.by_ref());
                length.resolve_symbols(handler, ctx.by_ref());
            }
            ExpressionKind::Struct(expr) => {
                expr.call_path_binding
                    .resolve_symbols(handler, ctx.by_ref());
                let result = SymbolResolveTypeBinding::resolve_symbol(
                    &mut expr.call_path_binding,
                    &Handler::default(),
                    ctx.by_ref(),
                );
                if let Ok(result) = result {
                    expr.resolved_call_path_binding = Some(TypeBinding::<
                        ResolvedCallPath<ParsedDeclId<StructDeclaration>>,
                    > {
                        inner: ResolvedCallPath {
                            decl: result,
                            unresolved_call_path: expr.call_path_binding.inner.clone(),
                        },
                        span: expr.call_path_binding.span.clone(),
                        type_arguments: expr.call_path_binding.type_arguments.clone(),
                    });
                }
            }
            ExpressionKind::CodeBlock(block) => {
                block
                    .contents
                    .iter_mut()
                    .for_each(|node| node.resolve_symbols(handler, ctx.by_ref()));
            }
            ExpressionKind::If(expr) => {
                expr.condition.resolve_symbols(handler, ctx.by_ref());
                expr.then.resolve_symbols(handler, ctx.by_ref());
                if let Some(r#else) = expr.r#else.as_mut() {
                    r#else.resolve_symbols(handler, ctx.by_ref());
                }
            }
            ExpressionKind::Match(expr) => {
                expr.value.resolve_symbols(handler, ctx.by_ref());
                expr.branches.iter_mut().for_each(|branch| {
                    branch.scrutinee.resolve_symbols(handler, ctx.by_ref());
                    branch.result.resolve_symbols(handler, ctx.by_ref());
                });
            }
            ExpressionKind::Asm(asm_expr) => asm_expr.registers.iter_mut().for_each(|reg| {
                if let Some(initializer) = reg.initializer.as_mut() {
                    initializer.resolve_symbols(handler, ctx.by_ref());
                }
            }),
            ExpressionKind::MethodApplication(expr) => {
                expr.method_name_binding
                    .resolve_symbols(handler, ctx.by_ref());
                expr.contract_call_params
                    .iter_mut()
                    .for_each(|field| field.resolve_symbols(handler, ctx.by_ref()));
                expr.arguments
                    .iter_mut()
                    .for_each(|arg| arg.resolve_symbols(handler, ctx.by_ref()));
            }
            ExpressionKind::Subfield(expr) => expr.prefix.resolve_symbols(handler, ctx),
            ExpressionKind::DelineatedPath(expr) => {
                expr.call_path_binding.resolve_symbols(handler, ctx)
            }
            ExpressionKind::AbiCast(expr) => {
                expr.abi_name.resolve_symbols(handler, ctx.by_ref());
                expr.address.resolve_symbols(handler, ctx.by_ref());
            }
            ExpressionKind::ArrayIndex(expr) => {
                expr.index.resolve_symbols(handler, ctx.by_ref());
                expr.prefix.resolve_symbols(handler, ctx.by_ref());
            }
            ExpressionKind::StorageAccess(_expr) => {}
            ExpressionKind::IntrinsicFunction(expr) => {
                expr.arguments
                    .iter_mut()
                    .for_each(|arg| arg.resolve_symbols(handler, ctx.by_ref()));
                expr.kind_binding.resolve_symbols(handler, ctx);
            }
            ExpressionKind::WhileLoop(expr) => {
                expr.condition.resolve_symbols(handler, ctx.by_ref());
                expr.body.resolve_symbols(handler, ctx.by_ref());
            }
            ExpressionKind::ForLoop(expr) => expr.desugared.resolve_symbols(handler, ctx.by_ref()),
            ExpressionKind::Break => {}
            ExpressionKind::Continue => {}
            ExpressionKind::Reassignment(expr) => {
                match &mut expr.lhs {
                    ReassignmentTarget::ElementAccess(expr) => {
                        expr.resolve_symbols(handler, ctx.by_ref())
                    }
                    ReassignmentTarget::Deref(expr) => expr.resolve_symbols(handler, ctx.by_ref()),
                };
                expr.rhs.resolve_symbols(handler, ctx.by_ref());
            }
            ExpressionKind::ImplicitReturn(expr) => expr.resolve_symbols(handler, ctx),
            ExpressionKind::Return(expr) => expr.resolve_symbols(handler, ctx.by_ref()),
            ExpressionKind::Panic(expr) => expr.resolve_symbols(handler, ctx.by_ref()),
            ExpressionKind::Ref(expr) => expr.value.resolve_symbols(handler, ctx.by_ref()),
            ExpressionKind::Deref(expr) => expr.resolve_symbols(handler, ctx.by_ref()),
        }
    }
}

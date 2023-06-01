use crate::{monomorphize::priv_prelude::*, language::ty::*};

pub(crate) trait CollectFrom {
    fn collect_from(&self, ctx: CollectContext);
}

impl CollectFrom for TyModule {
    fn collect_from(&self, ctx: CollectContext) {
        for (_, submod) in self.submodules_recursive() {
            submod.module.collect_from(ctx);
        }
        for node in self.all_nodes.iter() {
            node.content.collect_from(ctx);
        }
    }
}

impl CollectFrom for TyAstNodeContent {
    fn collect_from(&self, ctx: CollectContext) {
        use TyAstNodeContent::*;
        match self {
            Declaration(decl) => {
                decl.collect_from(ctx);
            }
            Expression(exp) => {
                exp.collect_from(ctx);
            }
            ImplicitReturnExpression(exp) => {
                exp.collect_from(ctx);
            }
            SideEffect(_) => {}
        }
    }
}

impl CollectFrom for TyDecl {
    fn collect_from(&self, ctx: CollectContext) {
        use TyDecl::*;
        match self {
            VariableDecl(decl) => {
                decl.body.collect_from(ctx);
            }
            ConstantDecl { .. } => todo!(),
            FunctionDecl {
                decl_id,
                subst_list,
                ..
            } => {
                gather_from_fn_decl(ctx, decl_id, subst_list.inner());
            }
            TraitDecl {
                name: _,
                decl_id,
                subst_list,
                decl_span: _,
            } => {
                gather_from_trait_decl(ctx, decl_id, subst_list.inner());
            }
            StructDecl {
                name: _,
                decl_id,
                subst_list,
                decl_span: _,
            } => {
                gather_from_struct_decl(ctx, decl_id, subst_list.inner());
            }
            EnumDecl {
                name: _,
                decl_id,
                subst_list,
                decl_span: _,
            } => {
                gather_from_enum_decl(ctx, decl_id, subst_list.inner());
            }
            EnumVariantDecl { .. } => todo!(),
            ImplTrait { .. } => todo!(),
            AbiDecl { .. } => todo!(),
            GenericTypeForFunctionScope { .. } => todo!(),
            StorageDecl { .. } => todo!(),
            ErrorRecovery(_) => {}
            TypeAliasDecl { .. } => todo!(),
        }
    }
}

impl CollectFrom for TyFunctionDecl {
    fn collect_from(&self, ctx: CollectContext) {
        ctx.add_constraint(Constraint::mk_fn_decl(decl_id, subst_list));
        let fn_decl = ctx.decl_engine.get_function(decl_id);
        for param in fn_decl.parameters {
            gather_from_ty(ctx, param.type_argument.type_id);
        }
        gather_from_ty(ctx, fn_decl.return_type.type_id);
        gather_from_code_block(ctx, &fn_decl.body);
    }
}

impl CollectFrom for TyStructDecl {
    fn collect_from(&self, ctx: CollectContext) {
        ctx.add_constraint(Constraint::mk_struct_decl(decl_id, subst_list));
        let struct_decl = ctx.decl_engine.get_struct(decl_id);
        for field in struct_decl.fields {
            gather_from_ty(ctx, field.type_argument.type_id);
        }
    }
}

impl CollectFrom for TyEnumDecl {
    fn collect_from(&self, ctx: CollectContext) {
        ctx.add_constraint(Constraint::mk_enum_decl(decl_id, subst_list));
        let enum_decl = ctx.decl_engine.get_enum(decl_id);
        for variant in enum_decl.variants {
            gather_from_ty(ctx, variant.type_argument.type_id);
        }
    }
}

impl CollectFrom for TyTraitDecl {
    fn collect_from(&self, ctx: CollectContext) {
        ctx.add_constraint(Constraint::mk_trait_decl(decl_id, subst_list));
        let trait_decl = ctx.decl_engine.get_trait(decl_id);
        todo!();
    }
}

impl CollectFrom for TyExpression {
    fn collect_from(&self, ctx: CollectContext) {
        gather_from_ty(ctx, return_type);
        self.expression.collect_from(ctx);
    }
}

impl CollectFrom for TyExpressionVariant {
    fn collect_from(&self, ctx: CollectContext) {
        use TyExpressionVariant::*;
        match exp {
            FunctionApplication {
                arguments,
                contract_call_params,
                call_path,
                fn_ref,
                ..
            } => {
                arguments
                    .iter()
                    .for_each(|(_, arg)| {
                        arg.collect_from(ctx);
                    });
                contract_call_params
                    .iter()
                    .for_each(|(_, arg)| {
                        arg.collect_from(ctx);
                    });
                ctx.add_constraint(Constraint::mk_fn_call(call_path, arguments, fn_ref));
            }
            LazyOperator { lhs, rhs, .. } => {
                lhs.collect_from(ctx);
                rhs.collect_from(ctx);
            }
            VariableExpression { .. } => {
                // NOTE: may need to do something here later
            }
            Tuple { fields } => {
                fields.iter().for_each(|field| {
                    field.collect_from(ctx);
                });
            }
            Array {
                contents: _,
                elem_type: _,
            } => {
                todo!();
                // contents
                //     .iter()
                //     .for_each(|elem| gather_from_exp(ctx, elem));
            }
            ArrayIndex {
                prefix: _,
                index: _,
            } => {
                todo!();
                // gather_from_exp(ctx, prefix);
                // gather_from_exp(ctx, index);
            }
            StructExpression { .. } => todo!(),
            CodeBlock(block) => {
                gather_from_code_block(ctx, block);
            }
            IfExp { .. } => todo!(),
            MatchExp { .. } => todo!(),
            AsmExpression { .. } => todo!(),
            StructFieldAccess { .. } => todo!(),
            TupleElemAccess { prefix, .. } => {
                prefix.collect_from(ctx);
            }
            EnumInstantiation { .. } => todo!(),
            AbiCast { .. } => todo!(),
            StorageAccess(_) => todo!(),
            IntrinsicFunction(_) => todo!(),
            AbiName(_) => todo!(),
            EnumTag { exp } => {
                exp.collect_from(ctx);
            }
            UnsafeDowncast { .. } => todo!(),
            WhileLoop { .. } => todo!(),
            Reassignment(_) => todo!(),
            StorageReassignment(_) => todo!(),
            Return(exp) => {
                exp.collect_from(ctx);
            }
            Literal(_) => {}
            Break => {}
            Continue => {}
            FunctionParameter => {}
        }
    }
}

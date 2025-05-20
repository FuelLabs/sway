use crate::{
    core::token::{ParsedAstToken, SymbolKind, Token},
    traverse::{adaptive_iter, Parse, ParseContext},
};
use rayon::iter::{ParallelBridge, ParallelIterator};
use sway_ast::{
    assignable::ElementAccess, attribute::Annotated, expr::LoopControlFlow, ty::TyTupleDescriptor,
    Assignable, CodeBlockContents, ConfigurableField, Expr, ExprArrayDescriptor, ExprStructField,
    ExprTupleDescriptor, FnArg, FnArgs, FnSignature, IfCondition, IfExpr, ItemAbi,
    ItemConfigurable, ItemConst, ItemEnum, ItemFn, ItemImpl, ItemImplItem, ItemKind, ItemStorage,
    ItemStruct, ItemTrait, ItemTypeAlias, ItemUse, MatchBranchKind, ModuleKind, Pattern,
    PatternStructField, Statement, StatementLet, StorageEntry, StorageField, TraitType, Ty,
    TypeField, UseTree,
};
use sway_core::language::{lexed::LexedProgram, HasSubmodules};
use sway_types::{Ident, Span, Spanned};

pub struct LexedTree<'a> {
    ctx: &'a ParseContext<'a>,
}

impl<'a> LexedTree<'a> {
    pub fn new(ctx: &'a ParseContext<'a>) -> Self {
        Self { ctx }
    }

    pub fn traverse_node(&self, node: &Annotated<ItemKind>) {
        node.value.parse(self.ctx);
    }

    pub fn collect_module_kinds(&self, lexed_program: &LexedProgram) {
        insert_module_kind(self.ctx, &lexed_program.root.tree.value.kind);
        lexed_program
            .root
            .submodules_recursive()
            .for_each(|(_, dep)| {
                insert_module_kind(self.ctx, &dep.module.tree.value.kind);
            });
    }
}

fn insert_module_kind(ctx: &ParseContext, kind: &ModuleKind) {
    match kind {
        ModuleKind::Script { script_token } => {
            insert_program_type_keyword(ctx, script_token.span());
        }
        ModuleKind::Contract { contract_token } => {
            insert_program_type_keyword(ctx, contract_token.span());
        }
        ModuleKind::Predicate { predicate_token } => {
            insert_program_type_keyword(ctx, predicate_token.span());
        }
        ModuleKind::Library { library_token, .. } => {
            insert_program_type_keyword(ctx, library_token.span());
        }
    }
}

fn insert_program_type_keyword(ctx: &ParseContext, span: Span) {
    let ident = Ident::new(span);
    let token = Token::from_parsed(
        ParsedAstToken::Keyword(ident.clone()),
        SymbolKind::ProgramTypeKeyword,
    );
    ctx.tokens.insert(ctx.ident(&ident), token);
}

fn insert_keyword(ctx: &ParseContext, span: Span) {
    let ident = Ident::new(span);
    let token = Token::from_parsed(ParsedAstToken::Keyword(ident.clone()), SymbolKind::Keyword);
    ctx.tokens.insert(ctx.ident(&ident), token);
}

impl Parse for ItemKind {
    fn parse(&self, ctx: &ParseContext) {
        match self {
            ItemKind::Submodule(submod) => {
                insert_keyword(ctx, submod.mod_token.span());
            }
            ItemKind::Use(item_use) => {
                item_use.parse(ctx);
            }
            ItemKind::Struct(item_struct) => {
                item_struct.parse(ctx);
            }
            ItemKind::Enum(item_enum) => {
                item_enum.parse(ctx);
            }
            ItemKind::Fn(item_func) => {
                item_func.parse(ctx);
            }
            ItemKind::Trait(item_trait) => {
                item_trait.parse(ctx);
            }
            ItemKind::Impl(item_impl) => {
                item_impl.parse(ctx);
            }
            ItemKind::Abi(item_abi) => {
                item_abi.parse(ctx);
            }
            ItemKind::Const(item_const) => {
                item_const.parse(ctx);
            }
            ItemKind::Storage(item_storage) => {
                item_storage.parse(ctx);
            }
            ItemKind::Configurable(item_configurable) => {
                item_configurable.parse(ctx);
            }
            ItemKind::TypeAlias(item_type_alias) => {
                item_type_alias.parse(ctx);
            }
            ItemKind::Error(_, _) => {}
        }
    }
}

impl Parse for Expr {
    fn parse(&self, ctx: &ParseContext) {
        match self {
            Expr::AbiCast { abi_token, args } => {
                insert_keyword(ctx, abi_token.span());
                args.get().address.parse(ctx);
            }
            Expr::Struct { fields, .. } => {
                fields
                    .get()
                    .into_iter()
                    .par_bridge()
                    .for_each(|field| field.parse(ctx));
            }
            Expr::Tuple(tuple) => {
                tuple.get().parse(ctx);
            }
            Expr::Parens(parens) => {
                parens.get().parse(ctx);
            }
            Expr::Block(block) => {
                block.get().parse(ctx);
            }
            Expr::Array(array) => {
                array.get().parse(ctx);
            }
            Expr::Return {
                return_token,
                expr_opt,
            } => {
                insert_keyword(ctx, return_token.span());
                if let Some(expr) = expr_opt {
                    expr.parse(ctx);
                }
            }
            Expr::Panic {
                panic_token,
                expr_opt,
            } => {
                insert_keyword(ctx, panic_token.span());
                if let Some(expr) = expr_opt {
                    expr.parse(ctx);
                }
            }
            Expr::If(if_expr) => {
                if_expr.parse(ctx);
            }
            Expr::Match {
                match_token,
                value,
                branches,
            } => {
                insert_keyword(ctx, match_token.span());
                value.parse(ctx);
                adaptive_iter(branches.get(), |branch| {
                    branch.pattern.parse(ctx);
                    branch.kind.parse(ctx);
                });
            }
            Expr::While {
                while_token,
                condition,
                block,
            } => {
                insert_keyword(ctx, while_token.span());
                condition.parse(ctx);
                block.get().parse(ctx);
            }
            Expr::FuncApp { func, args } => {
                func.parse(ctx);
                args.get()
                    .into_iter()
                    .par_bridge()
                    .for_each(|expr| expr.parse(ctx));
            }
            Expr::Index { target, arg } => {
                target.parse(ctx);
                arg.get().parse(ctx);
            }
            Expr::MethodCall {
                target,
                contract_args_opt,
                args,
                ..
            } => {
                target.parse(ctx);
                if let Some(contract_args) = contract_args_opt {
                    contract_args
                        .get()
                        .into_iter()
                        .par_bridge()
                        .for_each(|expr| expr.parse(ctx));
                }
                args.get()
                    .into_iter()
                    .par_bridge()
                    .for_each(|expr| expr.parse(ctx));
            }
            Expr::FieldProjection { target, .. } => {
                target.parse(ctx);
            }
            Expr::TupleFieldProjection { target, .. } => {
                target.parse(ctx);
            }
            Expr::Ref {
                mut_token, expr, ..
            } => {
                if let Some(mut_token) = mut_token {
                    insert_keyword(ctx, mut_token.span());
                }
                expr.parse(ctx);
            }
            Expr::Deref { expr, .. } => {
                expr.parse(ctx);
            }
            Expr::Not { expr, .. } => {
                expr.parse(ctx);
            }
            Expr::Mul { lhs, rhs, .. }
            | Expr::Div { lhs, rhs, .. }
            | Expr::Pow { lhs, rhs, .. }
            | Expr::Modulo { lhs, rhs, .. }
            | Expr::Add { lhs, rhs, .. }
            | Expr::Sub { lhs, rhs, .. }
            | Expr::Shl { lhs, rhs, .. }
            | Expr::Shr { lhs, rhs, .. }
            | Expr::BitAnd { lhs, rhs, .. }
            | Expr::BitXor { lhs, rhs, .. }
            | Expr::BitOr { lhs, rhs, .. }
            | Expr::Equal { lhs, rhs, .. }
            | Expr::NotEqual { lhs, rhs, .. }
            | Expr::LessThan { lhs, rhs, .. }
            | Expr::GreaterThan { lhs, rhs, .. }
            | Expr::LessThanEq { lhs, rhs, .. }
            | Expr::GreaterThanEq { lhs, rhs, .. }
            | Expr::LogicalAnd { lhs, rhs, .. }
            | Expr::LogicalOr { lhs, rhs, .. } => {
                lhs.parse(ctx);
                rhs.parse(ctx);
            }
            Expr::Reassignment {
                assignable, expr, ..
            } => {
                assignable.parse(ctx);
                expr.parse(ctx);
            }
            Expr::Break { break_token } => {
                insert_keyword(ctx, break_token.span());
            }
            Expr::Continue { continue_token } => {
                insert_keyword(ctx, continue_token.span());
            }
            _ => {}
        }
    }
}

impl Parse for ItemUse {
    fn parse(&self, ctx: &ParseContext) {
        if let Some(visibility) = &self.visibility {
            insert_keyword(ctx, visibility.span());
        }
        insert_keyword(ctx, self.use_token.span());
        self.tree.parse(ctx);
    }
}

impl Parse for ItemStruct {
    fn parse(&self, ctx: &ParseContext) {
        if let Some(visibility) = &self.visibility {
            insert_keyword(ctx, visibility.span());
        }
        insert_keyword(ctx, self.struct_token.span());

        if let Some(where_clause_opt) = &self.where_clause_opt {
            insert_keyword(ctx, where_clause_opt.where_token.span());
        }

        self.fields
            .get()
            .into_iter()
            .par_bridge()
            .for_each(|field| field.value.parse(ctx));
    }
}

impl Parse for ItemEnum {
    fn parse(&self, ctx: &ParseContext) {
        if let Some(visibility) = &self.visibility {
            insert_keyword(ctx, visibility.span());
        }
        insert_keyword(ctx, self.enum_token.span());

        if let Some(where_clause_opt) = &self.where_clause_opt {
            insert_keyword(ctx, where_clause_opt.where_token.span());
        }

        self.fields
            .get()
            .into_iter()
            .par_bridge()
            .for_each(|field| field.value.parse(ctx));
    }
}

impl Parse for ItemFn {
    fn parse(&self, ctx: &ParseContext) {
        self.fn_signature.parse(ctx);
        self.body.get().parse(ctx);
    }
}

impl Parse for ItemTrait {
    fn parse(&self, ctx: &ParseContext) {
        if let Some(visibility) = &self.visibility {
            insert_keyword(ctx, visibility.span());
        }
        insert_keyword(ctx, self.trait_token.span());

        if let Some(where_clause_opt) = &self.where_clause_opt {
            insert_keyword(ctx, where_clause_opt.where_token.span());
        }

        adaptive_iter(self.trait_items.get(), |annotated| match &annotated.value {
            sway_ast::ItemTraitItem::Fn(fn_sig, _) => fn_sig.parse(ctx),
            sway_ast::ItemTraitItem::Const(item_const, _) => item_const.parse(ctx),
            sway_ast::ItemTraitItem::Type(item_type, _) => item_type.parse(ctx),
            sway_ast::ItemTraitItem::Error(_, _) => {}
        });

        if let Some(trait_defs_opt) = &self.trait_defs_opt {
            adaptive_iter(trait_defs_opt.get(), |item| item.value.parse(ctx));
        }
    }
}

impl Parse for ItemImpl {
    fn parse(&self, ctx: &ParseContext) {
        insert_keyword(ctx, self.impl_token.span());

        if let Some((.., for_token)) = &self.trait_opt {
            insert_keyword(ctx, for_token.span());
        }

        self.ty.parse(ctx);

        if let Some(where_clause_opt) = &self.where_clause_opt {
            insert_keyword(ctx, where_clause_opt.where_token.span());
        }

        adaptive_iter(self.contents.get(), |item| match &item.value {
            ItemImplItem::Fn(fn_decl) => fn_decl.parse(ctx),
            ItemImplItem::Const(const_decl) => const_decl.parse(ctx),
            ItemImplItem::Type(type_decl) => type_decl.parse(ctx),
        });
    }
}

impl Parse for ItemAbi {
    fn parse(&self, ctx: &ParseContext) {
        insert_keyword(ctx, self.abi_token.span());

        adaptive_iter(self.abi_items.get(), |annotated| match &annotated.value {
            sway_ast::ItemTraitItem::Fn(fn_sig, _) => fn_sig.parse(ctx),
            sway_ast::ItemTraitItem::Const(item_const, _) => item_const.parse(ctx),
            sway_ast::ItemTraitItem::Type(item_type, _) => item_type.parse(ctx),
            sway_ast::ItemTraitItem::Error(_, _) => {}
        });

        if let Some(abi_defs_opt) = self.abi_defs_opt.as_ref() {
            adaptive_iter(abi_defs_opt.get(), |item| item.value.parse(ctx));
        }
    }
}

impl Parse for ItemConst {
    fn parse(&self, ctx: &ParseContext) {
        if let Some(visibility) = &self.pub_token {
            insert_keyword(ctx, visibility.span());
        }
        insert_keyword(ctx, self.const_token.span());

        if let Some((.., ty)) = self.ty_opt.as_ref() {
            ty.parse(ctx);
        }

        if let Some(expr) = self.expr_opt.as_ref() {
            expr.parse(ctx);
        }
    }
}

impl Parse for TraitType {
    fn parse(&self, ctx: &ParseContext) {
        insert_keyword(ctx, self.type_token.span());

        if let Some(ty) = self.ty_opt.as_ref() {
            ty.parse(ctx);
        }
    }
}

impl Parse for ItemStorage {
    fn parse(&self, ctx: &ParseContext) {
        insert_keyword(ctx, self.storage_token.span());

        self.entries
            .get()
            .into_iter()
            .par_bridge()
            .for_each(|entry| entry.value.parse(ctx));
    }
}

impl Parse for StorageEntry {
    fn parse(&self, ctx: &ParseContext) {
        if let Some(namespace) = &self.namespace {
            namespace
                .clone()
                .into_inner()
                .into_iter()
                .par_bridge()
                .for_each(|entry| entry.value.parse(ctx));
        } else if let Some(field) = &self.field {
            field.parse(ctx);
        }
    }
}

impl Parse for StorageField {
    fn parse(&self, ctx: &ParseContext) {
        self.ty.parse(ctx);
        self.initializer.parse(ctx);
    }
}

impl Parse for ItemConfigurable {
    fn parse(&self, ctx: &ParseContext) {
        insert_keyword(ctx, self.configurable_token.span());

        self.fields
            .get()
            .into_iter()
            .par_bridge()
            .for_each(|field| field.value.parse(ctx));
    }
}

impl Parse for ConfigurableField {
    fn parse(&self, ctx: &ParseContext) {
        self.ty.parse(ctx);
        self.initializer.parse(ctx);
    }
}

impl Parse for ItemTypeAlias {
    fn parse(&self, ctx: &ParseContext) {
        if let Some(visibility) = &self.visibility {
            insert_keyword(ctx, visibility.span());
        }
        insert_keyword(ctx, self.type_token.span());

        self.ty.parse(ctx);
    }
}

impl Parse for UseTree {
    fn parse(&self, ctx: &ParseContext) {
        match self {
            UseTree::Group { imports } => {
                imports.get().into_iter().par_bridge().for_each(|use_tree| {
                    use_tree.parse(ctx);
                });
            }
            UseTree::Rename { as_token, .. } => {
                insert_keyword(ctx, as_token.span());
            }
            UseTree::Path { suffix, .. } => {
                suffix.parse(ctx);
            }
            _ => {}
        }
    }
}

impl Parse for TypeField {
    fn parse(&self, ctx: &ParseContext) {
        if let Some(visibility) = &self.visibility {
            insert_keyword(ctx, visibility.span());
        }
        self.ty.parse(ctx);
    }
}

impl Parse for Ty {
    fn parse(&self, ctx: &ParseContext) {
        match self {
            Ty::Tuple(tuple) => {
                tuple.get().parse(ctx);
            }
            Ty::Array(array) => {
                let inner = array.get();
                inner.ty.parse(ctx);
                inner.length.parse(ctx);
            }
            Ty::StringArray { str_token, length } => {
                insert_keyword(ctx, str_token.span());
                length.get().parse(ctx);
            }
            _ => {}
        }
    }
}

impl Parse for FnSignature {
    fn parse(&self, ctx: &ParseContext) {
        if let Some(visibility) = &self.visibility {
            insert_keyword(ctx, visibility.span());
        }
        insert_keyword(ctx, self.fn_token.span());

        self.arguments.get().parse(ctx);
        if let Some((.., ty)) = &self.return_type_opt {
            ty.parse(ctx);
        }
        if let Some(where_clause) = &self.where_clause_opt {
            insert_keyword(ctx, where_clause.where_token.span());
        }
    }
}

impl Parse for FnArgs {
    fn parse(&self, ctx: &ParseContext) {
        match self {
            FnArgs::Static(punct) => {
                punct
                    .into_iter()
                    .par_bridge()
                    .for_each(|fn_arg| fn_arg.parse(ctx));
            }
            FnArgs::NonStatic {
                self_token,
                ref_self,
                mutable_self,
                args_opt,
            } => {
                insert_keyword(ctx, self_token.span());
                if let Some(ref_token) = ref_self {
                    insert_keyword(ctx, ref_token.span());
                }
                if let Some(mut_token) = mutable_self {
                    insert_keyword(ctx, mut_token.span());
                }
                if let Some((.., punct)) = args_opt {
                    punct
                        .into_iter()
                        .par_bridge()
                        .for_each(|fn_arg| fn_arg.parse(ctx));
                }
            }
        }
    }
}

impl Parse for FnArg {
    fn parse(&self, ctx: &ParseContext) {
        self.pattern.parse(ctx);
        self.ty.parse(ctx);
    }
}

impl Parse for CodeBlockContents {
    fn parse(&self, ctx: &ParseContext) {
        adaptive_iter(&self.statements, |statement| {
            statement.parse(ctx);
        });
        if let Some(expr) = self.final_expr_opt.as_ref() {
            expr.parse(ctx);
        }
    }
}

impl Parse for Statement {
    fn parse(&self, ctx: &ParseContext) {
        match self {
            Statement::Let(let_stmt) => {
                let_stmt.parse(ctx);
            }
            Statement::Expr { expr, .. } => {
                expr.parse(ctx);
            }
            Statement::Item(item) => {
                item.value.parse(ctx);
            }
            Statement::Error(_, _) => {}
        }
    }
}

impl Parse for StatementLet {
    fn parse(&self, ctx: &ParseContext) {
        insert_keyword(ctx, self.let_token.span());
        self.pattern.parse(ctx);
        if let Some((.., ty)) = &self.ty_opt {
            ty.parse(ctx);
        }
        self.expr.parse(ctx);
    }
}

impl Parse for ExprArrayDescriptor {
    fn parse(&self, ctx: &ParseContext) {
        match self {
            ExprArrayDescriptor::Sequence(punct) => {
                punct
                    .into_iter()
                    .par_bridge()
                    .for_each(|expr| expr.parse(ctx));
            }
            ExprArrayDescriptor::Repeat { value, length, .. } => {
                value.parse(ctx);
                length.parse(ctx);
            }
        }
    }
}

impl Parse for IfExpr {
    fn parse(&self, ctx: &ParseContext) {
        insert_keyword(ctx, self.if_token.span());
        self.condition.parse(ctx);
        self.then_block.get().parse(ctx);
        if let Some((else_token, control_flow)) = &self.else_opt {
            insert_keyword(ctx, else_token.span());
            match control_flow {
                LoopControlFlow::Break(block) => {
                    block.get().parse(ctx);
                }
                LoopControlFlow::Continue(if_expr) => {
                    if_expr.parse(ctx);
                }
            }
        }
    }
}

impl Parse for IfCondition {
    fn parse(&self, ctx: &ParseContext) {
        match self {
            IfCondition::Expr(expr) => {
                expr.parse(ctx);
            }
            IfCondition::Let {
                let_token,
                lhs,
                rhs,
                ..
            } => {
                insert_keyword(ctx, let_token.span());
                lhs.parse(ctx);
                rhs.parse(ctx);
            }
        }
    }
}

impl Parse for Pattern {
    fn parse(&self, ctx: &ParseContext) {
        match self {
            Pattern::Var {
                reference, mutable, ..
            } => {
                if let Some(reference) = reference {
                    insert_keyword(ctx, reference.span());
                }
                if let Some(mutable) = mutable {
                    insert_keyword(ctx, mutable.span());
                }
            }
            Pattern::Constructor { args, .. } | Pattern::Tuple(args) => {
                args.get()
                    .into_iter()
                    .par_bridge()
                    .for_each(|pattern| pattern.parse(ctx));
            }
            Pattern::Struct { fields, .. } => {
                fields
                    .get()
                    .into_iter()
                    .par_bridge()
                    .for_each(|field| field.parse(ctx));
            }
            _ => {}
        }
    }
}

impl Parse for PatternStructField {
    fn parse(&self, ctx: &ParseContext) {
        if let PatternStructField::Field {
            pattern_opt: Some((.., pattern)),
            ..
        } = self
        {
            pattern.parse(ctx);
        }
    }
}

impl Parse for MatchBranchKind {
    fn parse(&self, ctx: &ParseContext) {
        match self {
            MatchBranchKind::Block { block, .. } => {
                block.get().parse(ctx);
            }
            MatchBranchKind::Expr { expr, .. } => {
                expr.parse(ctx);
            }
        }
    }
}

impl Parse for ExprStructField {
    fn parse(&self, ctx: &ParseContext) {
        if let Some((.., expr)) = &self.expr_opt {
            expr.parse(ctx);
        }
    }
}

impl Parse for ExprTupleDescriptor {
    fn parse(&self, ctx: &ParseContext) {
        if let ExprTupleDescriptor::Cons { head, tail, .. } = self {
            head.parse(ctx);
            tail.into_iter()
                .par_bridge()
                .for_each(|expr| expr.parse(ctx));
        }
    }
}

impl Parse for TyTupleDescriptor {
    fn parse(&self, ctx: &ParseContext) {
        if let TyTupleDescriptor::Cons { head, tail, .. } = self {
            head.parse(ctx);
            tail.into_iter()
                .par_bridge()
                .for_each(|expr| expr.parse(ctx));
        }
    }
}

impl Parse for ElementAccess {
    fn parse(&self, ctx: &ParseContext) {
        match self {
            ElementAccess::Index { target, arg } => {
                target.parse(ctx);
                arg.get().parse(ctx);
            }
            ElementAccess::FieldProjection { target, .. }
            | ElementAccess::TupleFieldProjection { target, .. } => {
                target.parse(ctx);
            }
            ElementAccess::Deref { target, .. } => target.parse(ctx),
            ElementAccess::Var(_) => {}
        }
    }
}

impl Parse for Assignable {
    fn parse(&self, ctx: &ParseContext) {
        match self {
            Assignable::ElementAccess(element_access) => element_access.parse(ctx),
            Assignable::Deref { expr, .. } => expr.parse(ctx),
        }
    }
}

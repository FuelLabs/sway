use crate::{
    core::{
        token::{to_ident_key, AstToken, SymbolKind, Token},
        token_map::TokenMap,
    },
    traverse::Parse,
};
use std::ops::ControlFlow;
use sway_ast::{
    ty::TyTupleDescriptor, Assignable, CodeBlockContents, ConfigurableField, Expr,
    ExprArrayDescriptor, ExprStructField, ExprTupleDescriptor, FnArg, FnArgs, FnSignature,
    IfCondition, IfExpr, ItemAbi, ItemConfigurable, ItemConst, ItemEnum, ItemFn, ItemImpl,
    ItemImplItem, ItemKind, ItemStorage, ItemStruct, ItemTrait, ItemUse, MatchBranchKind,
    ModuleKind, Pattern, PatternStructField, Statement, StatementLet, StorageField, Ty, TypeField,
    UseTree,
};
use sway_core::language::lexed::LexedProgram;
use sway_types::{Ident, Span, Spanned};

pub struct LexedTree<'a> {
    tokens: &'a TokenMap,
}

impl<'a> LexedTree<'a> {
    pub fn new(tokens: &'a TokenMap) -> Self {
        Self { tokens }
    }

    pub fn parse(&self, lexed_program: &LexedProgram) {
        insert_module_kind(self.tokens, &lexed_program.root.tree.kind);
        for item in &lexed_program.root.tree.items {
            item.value.parse(self.tokens);
        }

        for (.., dep) in &lexed_program.root.submodules {
            insert_module_kind(self.tokens, &dep.module.tree.kind);
            for item in &dep.module.tree.items {
                item.value.parse(self.tokens);
            }
        }
    }
}

fn insert_module_kind(tokens: &TokenMap, kind: &ModuleKind) {
    match kind {
        ModuleKind::Script { script_token } => {
            insert_keyword(tokens, script_token.span());
        }
        ModuleKind::Contract { contract_token } => {
            insert_keyword(tokens, contract_token.span());
        }
        ModuleKind::Predicate { predicate_token } => {
            insert_keyword(tokens, predicate_token.span());
        }
        ModuleKind::Library { library_token, .. } => {
            insert_keyword(tokens, library_token.span());
        }
    }
}

fn insert_keyword(tokens: &TokenMap, span: Span) {
    let ident = Ident::new(span);
    let token = Token::from_parsed(AstToken::Keyword(ident.clone()), SymbolKind::Keyword);
    tokens.insert(to_ident_key(&ident), token);
}

impl Parse for ItemKind {
    fn parse(&self, tokens: &TokenMap) {
        match self {
            ItemKind::Dependency(dependency) => {
                insert_keyword(tokens, dependency.dep_token.span());
            }
            ItemKind::Use(item_use) => {
                item_use.parse(tokens);
            }
            ItemKind::Struct(item_struct) => {
                item_struct.parse(tokens);
            }
            ItemKind::Enum(item_enum) => {
                item_enum.parse(tokens);
            }
            ItemKind::Fn(item_func) => {
                item_func.parse(tokens);
            }
            ItemKind::Trait(item_trait) => {
                item_trait.parse(tokens);
            }
            ItemKind::Impl(item_impl) => {
                item_impl.parse(tokens);
            }
            ItemKind::Abi(item_abi) => {
                item_abi.parse(tokens);
            }
            ItemKind::Const(item_const) => {
                item_const.parse(tokens);
            }
            ItemKind::Storage(item_storage) => {
                item_storage.parse(tokens);
            }
            ItemKind::Configurable(item_configurable) => {
                item_configurable.parse(tokens);
            }
        }
    }
}

impl Parse for Expr {
    fn parse(&self, tokens: &TokenMap) {
        match self {
            Expr::AbiCast { abi_token, args } => {
                insert_keyword(tokens, abi_token.span());
                args.get().address.parse(tokens);
            }
            Expr::Struct { fields, .. } => {
                for expr in fields.get() {
                    expr.parse(tokens);
                }
            }
            Expr::Tuple(tuple) => {
                tuple.get().parse(tokens);
            }
            Expr::Parens(parens) => {
                parens.get().parse(tokens);
            }
            Expr::Block(block) => {
                block.get().parse(tokens);
            }
            Expr::Array(array) => {
                array.get().parse(tokens);
            }
            Expr::Return {
                return_token,
                expr_opt,
            } => {
                insert_keyword(tokens, return_token.span());
                if let Some(expr) = expr_opt {
                    expr.parse(tokens);
                }
            }
            Expr::If(if_expr) => {
                if_expr.parse(tokens);
            }
            Expr::Match {
                match_token,
                value,
                branches,
            } => {
                insert_keyword(tokens, match_token.span());
                value.parse(tokens);
                for branch in branches.get() {
                    branch.pattern.parse(tokens);
                    branch.kind.parse(tokens);
                }
            }
            Expr::While {
                while_token,
                condition,
                block,
            } => {
                insert_keyword(tokens, while_token.span());
                condition.parse(tokens);
                block.get().parse(tokens);
            }
            Expr::FuncApp { func, args } => {
                func.parse(tokens);
                for expr in args.get().into_iter() {
                    expr.parse(tokens);
                }
            }
            Expr::Index { target, arg } => {
                target.parse(tokens);
                arg.get().parse(tokens);
            }
            Expr::MethodCall {
                target,
                contract_args_opt,
                args,
                ..
            } => {
                target.parse(tokens);
                if let Some(contract_args) = contract_args_opt {
                    for expr in contract_args.get().into_iter() {
                        expr.parse(tokens);
                    }
                }
                for expr in args.get().into_iter() {
                    expr.parse(tokens);
                }
            }
            Expr::FieldProjection { target, .. } => {
                target.parse(tokens);
            }
            Expr::TupleFieldProjection { target, .. } => {
                target.parse(tokens);
            }
            Expr::Ref { ref_token, expr } => {
                insert_keyword(tokens, ref_token.span());
                expr.parse(tokens);
            }
            Expr::Deref { deref_token, expr } => {
                insert_keyword(tokens, deref_token.span());
                expr.parse(tokens);
            }
            Expr::Not { expr, .. } => {
                expr.parse(tokens);
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
                lhs.parse(tokens);
                rhs.parse(tokens);
            }
            Expr::Reassignment {
                assignable, expr, ..
            } => {
                assignable.parse(tokens);
                expr.parse(tokens);
            }
            Expr::Break { break_token } => {
                insert_keyword(tokens, break_token.span());
            }
            Expr::Continue { continue_token } => {
                insert_keyword(tokens, continue_token.span());
            }
            _ => {}
        }
    }
}

impl Parse for ItemUse {
    fn parse(&self, tokens: &TokenMap) {
        if let Some(visibility) = &self.visibility {
            insert_keyword(tokens, visibility.span());
        }
        insert_keyword(tokens, self.use_token.span());
        self.tree.parse(tokens);
    }
}

impl Parse for ItemStruct {
    fn parse(&self, tokens: &TokenMap) {
        if let Some(visibility) = &self.visibility {
            insert_keyword(tokens, visibility.span());
        }
        insert_keyword(tokens, self.struct_token.span());

        if let Some(where_clause_opt) = &self.where_clause_opt {
            insert_keyword(tokens, where_clause_opt.where_token.span());
        }

        self.fields
            .get()
            .into_iter()
            .for_each(|field| field.value.parse(tokens));
    }
}

impl Parse for ItemEnum {
    fn parse(&self, tokens: &TokenMap) {
        if let Some(visibility) = &self.visibility {
            insert_keyword(tokens, visibility.span());
        }
        insert_keyword(tokens, self.enum_token.span());

        if let Some(where_clause_opt) = &self.where_clause_opt {
            insert_keyword(tokens, where_clause_opt.where_token.span());
        }

        self.fields
            .get()
            .into_iter()
            .for_each(|field| field.value.parse(tokens));
    }
}

impl Parse for ItemFn {
    fn parse(&self, tokens: &TokenMap) {
        self.fn_signature.parse(tokens);
        self.body.get().parse(tokens);
    }
}

impl Parse for ItemTrait {
    fn parse(&self, tokens: &TokenMap) {
        if let Some(visibility) = &self.visibility {
            insert_keyword(tokens, visibility.span());
        }
        insert_keyword(tokens, self.trait_token.span());

        if let Some(where_clause_opt) = &self.where_clause_opt {
            insert_keyword(tokens, where_clause_opt.where_token.span());
        }

        self.trait_items
            .get()
            .iter()
            .for_each(|(annotated, _)| match &annotated.value {
                sway_ast::ItemTraitItem::Fn(fn_sig) => fn_sig.parse(tokens),
            });

        if let Some(trait_defs_opt) = &self.trait_defs_opt {
            trait_defs_opt
                .get()
                .iter()
                .for_each(|item| item.value.parse(tokens));
        }
    }
}

impl Parse for ItemImpl {
    fn parse(&self, tokens: &TokenMap) {
        insert_keyword(tokens, self.impl_token.span());

        if let Some((.., for_token)) = &self.trait_opt {
            insert_keyword(tokens, for_token.span());
        }

        self.ty.parse(tokens);

        if let Some(where_clause_opt) = &self.where_clause_opt {
            insert_keyword(tokens, where_clause_opt.where_token.span());
        }

        self.contents
            .get()
            .iter()
            .for_each(|item| match &item.value {
                ItemImplItem::Fn(fn_decl) => fn_decl.parse(tokens),
            });
    }
}

impl Parse for ItemAbi {
    fn parse(&self, tokens: &TokenMap) {
        insert_keyword(tokens, self.abi_token.span());

        self.abi_items
            .get()
            .iter()
            .for_each(|(annotated, _)| match &annotated.value {
                sway_ast::ItemTraitItem::Fn(fn_sig) => fn_sig.parse(tokens),
            });

        if let Some(abi_defs_opt) = self.abi_defs_opt.as_ref() {
            abi_defs_opt
                .get()
                .iter()
                .for_each(|item| item.value.parse(tokens));
        }
    }
}

impl Parse for ItemConst {
    fn parse(&self, tokens: &TokenMap) {
        if let Some(visibility) = &self.visibility {
            insert_keyword(tokens, visibility.span());
        }
        insert_keyword(tokens, self.const_token.span());

        if let Some((.., ty)) = self.ty_opt.as_ref() {
            ty.parse(tokens);
        }

        if let Some(expr) = self.expr_opt.as_ref() {
            expr.parse(tokens);
        }
    }
}

impl Parse for ItemStorage {
    fn parse(&self, tokens: &TokenMap) {
        insert_keyword(tokens, self.storage_token.span());

        self.fields
            .get()
            .into_iter()
            .for_each(|field| field.value.parse(tokens));
    }
}

impl Parse for StorageField {
    fn parse(&self, tokens: &TokenMap) {
        self.ty.parse(tokens);
        self.initializer.parse(tokens);
    }
}

impl Parse for ItemConfigurable {
    fn parse(&self, tokens: &TokenMap) {
        insert_keyword(tokens, self.configurable_token.span());

        self.fields
            .get()
            .into_iter()
            .for_each(|field| field.value.parse(tokens));
    }
}

impl Parse for ConfigurableField {
    fn parse(&self, tokens: &TokenMap) {
        self.ty.parse(tokens);
        self.initializer.parse(tokens);
    }
}

impl Parse for UseTree {
    fn parse(&self, tokens: &TokenMap) {
        match self {
            UseTree::Group { imports } => {
                for use_tree in imports.get().into_iter() {
                    use_tree.parse(tokens);
                }
            }
            UseTree::Rename { as_token, .. } => {
                insert_keyword(tokens, as_token.span());
            }
            UseTree::Path { suffix, .. } => {
                suffix.parse(tokens);
            }
            _ => {}
        }
    }
}

impl Parse for TypeField {
    fn parse(&self, tokens: &TokenMap) {
        self.ty.parse(tokens);
    }
}

impl Parse for Ty {
    fn parse(&self, tokens: &TokenMap) {
        match self {
            Ty::Tuple(tuple) => {
                tuple.get().parse(tokens);
            }
            Ty::Array(array) => {
                let inner = array.get();
                inner.ty.parse(tokens);
                inner.length.parse(tokens);
            }
            Ty::Str { str_token, length } => {
                insert_keyword(tokens, str_token.span());
                length.get().parse(tokens);
            }
            _ => {}
        }
    }
}

impl Parse for FnSignature {
    fn parse(&self, tokens: &TokenMap) {
        if let Some(visibility) = &self.visibility {
            insert_keyword(tokens, visibility.span());
        }
        insert_keyword(tokens, self.fn_token.span());

        self.arguments.get().parse(tokens);
        if let Some((.., ty)) = &self.return_type_opt {
            ty.parse(tokens);
        }
        if let Some(where_clause) = &self.where_clause_opt {
            insert_keyword(tokens, where_clause.where_token.span());
        }
    }
}

impl Parse for FnArgs {
    fn parse(&self, tokens: &TokenMap) {
        match self {
            FnArgs::Static(punct) => {
                punct.into_iter().for_each(|fn_arg| fn_arg.parse(tokens));
            }
            FnArgs::NonStatic {
                self_token,
                ref_self,
                mutable_self,
                args_opt,
            } => {
                insert_keyword(tokens, self_token.span());
                if let Some(ref_token) = ref_self {
                    insert_keyword(tokens, ref_token.span());
                }
                if let Some(mut_token) = mutable_self {
                    insert_keyword(tokens, mut_token.span());
                }
                if let Some((.., punct)) = args_opt {
                    punct.into_iter().for_each(|fn_arg| fn_arg.parse(tokens));
                }
            }
        }
    }
}

impl Parse for FnArg {
    fn parse(&self, tokens: &TokenMap) {
        self.pattern.parse(tokens);
        self.ty.parse(tokens);
    }
}

impl Parse for CodeBlockContents {
    fn parse(&self, tokens: &TokenMap) {
        for statement in self.statements.iter() {
            statement.parse(tokens);
        }
        if let Some(expr) = self.final_expr_opt.as_ref() {
            expr.parse(tokens);
        }
    }
}

impl Parse for Statement {
    fn parse(&self, tokens: &TokenMap) {
        match self {
            Statement::Let(let_stmt) => {
                let_stmt.parse(tokens);
            }
            Statement::Expr { expr, .. } => {
                expr.parse(tokens);
            }
            Statement::Item(item) => {
                item.value.parse(tokens);
            }
        }
    }
}

impl Parse for StatementLet {
    fn parse(&self, tokens: &TokenMap) {
        insert_keyword(tokens, self.let_token.span());
        self.pattern.parse(tokens);
        if let Some((.., ty)) = &self.ty_opt {
            ty.parse(tokens);
        }
        self.expr.parse(tokens);
    }
}

impl Parse for ExprArrayDescriptor {
    fn parse(&self, tokens: &TokenMap) {
        match self {
            ExprArrayDescriptor::Sequence(punct) => {
                for expr in punct.into_iter() {
                    expr.parse(tokens);
                }
            }
            ExprArrayDescriptor::Repeat { value, length, .. } => {
                value.parse(tokens);
                length.parse(tokens);
            }
        }
    }
}

impl Parse for IfExpr {
    fn parse(&self, tokens: &TokenMap) {
        insert_keyword(tokens, self.if_token.span());
        self.condition.parse(tokens);
        self.then_block.get().parse(tokens);
        if let Some((else_token, control_flow)) = &self.else_opt {
            insert_keyword(tokens, else_token.span());
            match control_flow {
                ControlFlow::Break(block) => {
                    block.get().parse(tokens);
                }
                ControlFlow::Continue(if_expr) => {
                    if_expr.parse(tokens);
                }
            }
        }
    }
}

impl Parse for IfCondition {
    fn parse(&self, tokens: &TokenMap) {
        match self {
            IfCondition::Expr(expr) => {
                expr.parse(tokens);
            }
            IfCondition::Let {
                let_token,
                lhs,
                rhs,
                ..
            } => {
                insert_keyword(tokens, let_token.span());
                lhs.parse(tokens);
                rhs.parse(tokens);
            }
        }
    }
}

impl Parse for Pattern {
    fn parse(&self, tokens: &TokenMap) {
        match self {
            Pattern::Var {
                reference, mutable, ..
            } => {
                if let Some(reference) = reference {
                    insert_keyword(tokens, reference.span());
                }
                if let Some(mutable) = mutable {
                    insert_keyword(tokens, mutable.span());
                }
            }
            Pattern::Constructor { args, .. } | Pattern::Tuple(args) => {
                for pattern in args.get().into_iter() {
                    pattern.parse(tokens);
                }
            }
            Pattern::Struct { fields, .. } => {
                for field in fields.get().into_iter() {
                    field.parse(tokens);
                }
            }
            _ => {}
        }
    }
}

impl Parse for PatternStructField {
    fn parse(&self, tokens: &TokenMap) {
        if let PatternStructField::Field {
            pattern_opt: Some((.., pattern)),
            ..
        } = self
        {
            pattern.parse(tokens);
        }
    }
}

impl Parse for MatchBranchKind {
    fn parse(&self, tokens: &TokenMap) {
        match self {
            MatchBranchKind::Block { block, .. } => {
                block.get().parse(tokens);
            }
            MatchBranchKind::Expr { expr, .. } => {
                expr.parse(tokens);
            }
        }
    }
}

impl Parse for ExprStructField {
    fn parse(&self, tokens: &TokenMap) {
        if let Some((.., expr)) = &self.expr_opt {
            expr.parse(tokens);
        }
    }
}

impl Parse for ExprTupleDescriptor {
    fn parse(&self, tokens: &TokenMap) {
        if let ExprTupleDescriptor::Cons { head, tail, .. } = self {
            head.parse(tokens);
            for expr in tail.into_iter() {
                expr.parse(tokens);
            }
        }
    }
}

impl Parse for TyTupleDescriptor {
    fn parse(&self, tokens: &TokenMap) {
        if let TyTupleDescriptor::Cons { head, tail, .. } = self {
            head.parse(tokens);
            for expr in tail.into_iter() {
                expr.parse(tokens);
            }
        }
    }
}

impl Parse for Assignable {
    fn parse(&self, tokens: &TokenMap) {
        match self {
            Assignable::Index { target, arg } => {
                target.parse(tokens);
                arg.get().parse(tokens)
            }
            Assignable::FieldProjection { target, .. }
            | Assignable::TupleFieldProjection { target, .. } => {
                target.parse(tokens);
            }
            _ => {}
        }
    }
}

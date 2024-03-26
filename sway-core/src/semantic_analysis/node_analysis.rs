//! This module handles the process of iterating through the parsed AST and doing an analysis.
//! At the moment we compute an dependency graph between parsed nodes.

use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::fs;

use petgraph::stable_graph::NodeIndex;
use petgraph::Graph;
use sway_error::handler::{ErrorEmitted, Handler};

use crate::decl_engine::parsed_id::ParsedDeclId;
use crate::decl_engine::DeclUniqueId;
use crate::engine_threading::DebugWithEngines;
use crate::language::parsed::{
    AbiDeclaration, AstNode, AstNodeContent, CodeBlock, ConstantDeclaration, Declaration,
    EnumDeclaration, Expression, ExpressionKind, FunctionDeclaration, ImplItem, ImplSelf,
    ImplTrait, ParseModule, ParseProgram, Scrutinee, StorageDeclaration, StructDeclaration,
    TraitDeclaration, TraitTypeDeclaration, TypeAliasDeclaration, VariableDeclaration,
};
use crate::Engines;

use super::collection_context::SymbolCollectionContext;

pub type ParsedNodeDepGraphNodeId = petgraph::graph::NodeIndex;

#[derive(Clone, Debug)]
pub enum ParsedNodeDepGraphEdgeInfo {
    FnApp,
}

#[derive(Clone, Debug)]
pub struct ParsedNodeDepGraphEdge(pub ParsedNodeDepGraphEdgeInfo);

impl Display for ParsedNodeDepGraphEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            ParsedNodeDepGraphEdgeInfo::FnApp => write!(f, "fn app"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ParsedNodeDepGraphNode {
    ImplSelf {
        decl_id: ParsedDeclId<ImplSelf>,
    },
    ImplTrait {
        decl_id: ParsedDeclId<ImplTrait>,
    },
    ImplTraitItem {
        item: ImplItem,
    },
    Fn {
        decl_id: ParsedDeclId<FunctionDeclaration>,
    },
    Variable {
        decl_id: ParsedDeclId<VariableDeclaration>,
    },
    Struct {
        decl_id: ParsedDeclId<StructDeclaration>,
    },
}

// Represents an ordered graph between declaration id indexes.
pub type ParsedNodeDepGraph =
    petgraph::graph::DiGraph<ParsedNodeDepGraphNode, ParsedNodeDepGraphEdge>;

// A simple context that is used to pass context necessary for parsed AST analysis.
pub struct NodeAnalysisContext<'cx> {
    pub(crate) engines: &'cx Engines,
    pub(crate) symbol_ctx: &'cx SymbolCollectionContext,
    pub(crate) dep_graph: ParsedNodeDepGraph,
    pub(crate) nodes: HashMap<DeclUniqueId, ParsedNodeDepGraphNodeId>,
    pub(crate) items_node_stack: Vec<ParsedNodeDepGraphNodeId>,
    pub(crate) node_stack: Vec<ParsedNodeDepGraphNodeId>,
}

impl NodeAnalysisContext<'_> {
    pub fn add_node(&mut self, node: ParsedNodeDepGraphNode) -> ParsedNodeDepGraphNodeId {
        self.dep_graph.add_node(node)
    }

    pub fn add_edge_from_current(
        &mut self,
        to: ParsedNodeDepGraphNodeId,
        edge: ParsedNodeDepGraphEdge,
    ) {
        let from = *self.node_stack.last().unwrap();
        if !self.dep_graph.contains_edge(from, to) {
            self.dep_graph.add_edge(from, to, edge);
        }
    }

    #[allow(clippy::map_entry)]
    pub fn get_or_create_node_for_impl_item(
        &mut self,
        item: &ImplItem,
    ) -> ParsedNodeDepGraphNodeId {
        let id = match item {
            ImplItem::Fn(decl_id) => decl_id.unique_id(),
            ImplItem::Constant(decl_id) => decl_id.unique_id(),
            ImplItem::Type(decl_id) => decl_id.unique_id(),
        };
        if self.nodes.contains_key(&id) {
            *self.nodes.get(&id).unwrap()
        } else {
            let item_node =
                self.add_node(ParsedNodeDepGraphNode::ImplTraitItem { item: item.clone() });

            self.nodes.insert(id, item_node);
            item_node
        }
    }

    /// This functions either gets an existing node in the graph, or creates a new
    /// node corresponding to the passed function declaration node.
    /// The function will try to find a non-monomorphized declaration node id so that
    /// future acesses always normalize to the same node id.
    #[allow(clippy::map_entry)]
    pub fn get_or_create_node_for_fn_decl(
        &mut self,
        fn_decl_id: &ParsedDeclId<FunctionDeclaration>,
    ) -> ParsedNodeDepGraphNodeId {
        let id = fn_decl_id.unique_id();
        if self.nodes.contains_key(&id) {
            *self.nodes.get(&id).unwrap()
        } else {
            let item_node = self.add_node(ParsedNodeDepGraphNode::Fn {
                decl_id: *fn_decl_id,
            });

            self.nodes.insert(id, item_node);
            item_node
        }
    }

    /// This function will process an impl self declaration, pushing graph nodes
    /// corresponding to each item in the trait impl.
    #[allow(clippy::map_entry)]
    pub(crate) fn push_nodes_for_impl_self(
        &mut self,
        impl_self: &ParsedDeclId<ImplSelf>,
    ) -> ParsedNodeDepGraphNodeId {
        if self.nodes.contains_key(&impl_self.unique_id()) {
            *self.nodes.get(&impl_self.unique_id()).unwrap()
        } else {
            let node = self.add_node(ParsedNodeDepGraphNode::ImplSelf {
                decl_id: *impl_self,
            });
            self.nodes.insert(impl_self.unique_id(), node);

            let decl_engine = self.engines.pe();
            let impl_self = decl_engine.get_impl_self(impl_self);

            for item in impl_self.items.iter() {
                let item_node = self.get_or_create_node_for_impl_item(item);

                // Connect the item node to the impl trait node.
                self.dep_graph.add_edge(
                    node,
                    item_node,
                    ParsedNodeDepGraphEdge(ParsedNodeDepGraphEdgeInfo::FnApp),
                );

                self.items_node_stack.push(item_node);
            }

            node
        }
    }

    /// This function will process an impl trait declaration, pushing graph nodes
    /// corresponding to each item in the trait impl.
    #[allow(clippy::map_entry)]
    pub(crate) fn push_nodes_for_impl_trait(
        &mut self,
        impl_trait: &ParsedDeclId<ImplTrait>,
    ) -> ParsedNodeDepGraphNodeId {
        if self.nodes.contains_key(&impl_trait.unique_id()) {
            *self.nodes.get(&impl_trait.unique_id()).unwrap()
        } else {
            let node = self.add_node(ParsedNodeDepGraphNode::ImplTrait {
                decl_id: *impl_trait,
            });
            self.nodes.insert(impl_trait.unique_id(), node);

            let decl_engine = self.engines.pe();
            let impl_trait = decl_engine.get_impl_trait(impl_trait);

            for item in impl_trait.items.iter() {
                let item_node = self.get_or_create_node_for_impl_item(item);

                // Connect the item node to the impl trait node.
                self.dep_graph.add_edge(
                    node,
                    item_node,
                    ParsedNodeDepGraphEdge(ParsedNodeDepGraphEdgeInfo::FnApp),
                );

                self.items_node_stack.push(item_node);
            }

            node
        }
    }

    /// This function will return an option to the node that represents the
    /// function being referenced by a function application.
    pub(crate) fn get_node_for_fn_decl(
        &mut self,
        fn_decl_id: &ParsedDeclId<FunctionDeclaration>,
    ) -> Option<ParsedNodeDepGraphNodeId> {
        if let Some(found) = self.nodes.get(&fn_decl_id.unique_id()) {
            return Some(*found);
        }

        for index in self.items_node_stack.iter().rev() {
            let node = self
                .dep_graph
                .node_weight(*index)
                .expect("expecting valid node id");

            let item_fn_decl_id = match node {
                ParsedNodeDepGraphNode::ImplTrait { decl_id: _ } => unreachable!(),
                ParsedNodeDepGraphNode::ImplTraitItem {
                    item: ImplItem::Fn(decl_id),
                } => *decl_id,
                ParsedNodeDepGraphNode::Fn {
                    decl_id: fn_decl_id,
                } => *fn_decl_id,
                _ => continue,
            };

            if item_fn_decl_id.unique_id() == fn_decl_id.unique_id() {
                return Some(*index);
            }
        }

        // If no node has been found yet, create it.
        let node = self.get_or_create_node_for_fn_decl(fn_decl_id);
        Some(node)
    }

    /// Prints out GraphViz DOT format for the dependency graph.
    #[allow(dead_code)]
    pub(crate) fn visualize(&self, engines: &Engines, print_graph: Option<String>) {
        if let Some(graph_path) = print_graph {
            use petgraph::dot::{Config, Dot};
            let string_graph = self.dep_graph.filter_map(
                |_idx, node| Some(format!("{:?}", engines.help_out(node))),
                |_idx, edge| Some(format!("{}", edge)),
            );

            let output = format!(
                "{:?}",
                Dot::with_attr_getters(
                    &string_graph,
                    &[Config::NodeNoLabel, Config::EdgeNoLabel],
                    &|_, er| format!("label = {:?}", er.weight()),
                    &|_, nr| {
                        let _node = &self.dep_graph[nr.0];
                        let shape = "";
                        let url = "".to_string();
                        format!("{shape} label = {:?} {url}", nr.1)
                    },
                )
            );

            if graph_path.is_empty() {
                tracing::info!("{output}");
            } else {
                let result = fs::write(graph_path.clone(), output);
                if let Some(error) = result.err() {
                    tracing::error!(
                        "There was an issue while outputing type check analysis graph to path {graph_path:?}\n{error}"
                    );
                }
            }
        }
    }

    pub(crate) fn get_sub_graph(
        &self,
        node_index: NodeIndex,
    ) -> Graph<&ParsedNodeDepGraphNode, &ParsedNodeDepGraphEdge> {
        let neighbors: Vec<_> = self
            .dep_graph
            .neighbors_directed(node_index, petgraph::Direction::Outgoing)
            .collect();
        let neighbors_set: HashSet<&NodeIndex> = HashSet::from_iter(neighbors.iter());
        self.dep_graph.filter_map(
            |node_index, node| {
                if neighbors_set.contains(&node_index) {
                    Some(node)
                } else {
                    None
                }
            },
            |_edge_index, edge| Some(edge),
        )
    }
}

impl DebugWithEngines for ParsedNodeDepGraphNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, engines: &Engines) -> std::fmt::Result {
        match self {
            ParsedNodeDepGraphNode::ImplSelf { decl_id } => {
                let node = engines.pe().get_impl_self(decl_id);
                node.fmt(f, engines)
            }
            ParsedNodeDepGraphNode::ImplTrait { decl_id } => {
                let node = engines.pe().get_impl_trait(decl_id);
                node.fmt(f, engines)
            }
            ParsedNodeDepGraphNode::ImplTraitItem { item: node } => node.fmt(f, engines),
            ParsedNodeDepGraphNode::Fn { decl_id } => {
                let node = engines.pe().get_function(decl_id);
                node.fmt(f, engines)
            }
            ParsedNodeDepGraphNode::Variable { decl_id } => {
                let node = engines.pe().get_variable(decl_id);
                node.fmt(f, engines)
            }
            ParsedNodeDepGraphNode::Struct { decl_id } => {
                let node = engines.pe().get_struct(decl_id);
                node.fmt(f, engines)
            }
        }
    }
}

impl<'cx> NodeAnalysisContext<'cx> {
    pub fn new(engines: &'cx Engines, symbol_ctx: &'cx SymbolCollectionContext) -> Self {
        Self {
            engines,
            symbol_ctx,
            dep_graph: Default::default(),
            nodes: Default::default(),
            items_node_stack: Default::default(),
            node_stack: Default::default(),
        }
    }
}

pub(crate) trait NodeAnalysis {
    fn analyze(&self, handler: &Handler, ctx: &mut NodeAnalysisContext)
        -> Result<(), ErrorEmitted>;
}

impl NodeAnalysis for ParseProgram {
    fn analyze(
        &self,
        handler: &Handler,
        ctx: &mut NodeAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        self.root.analyze(handler, ctx)
    }
}

impl NodeAnalysis for ParseModule {
    fn analyze(
        &self,
        handler: &Handler,
        ctx: &mut NodeAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        for (_name, submodule) in &self.submodules {
            let _ = submodule.module.analyze(handler, ctx);
        }
        for node in &self.tree.root_nodes {
            let _ = node.analyze(handler, ctx);
        }
        Ok(())
    }
}

impl NodeAnalysis for AstNode {
    fn analyze(
        &self,
        handler: &Handler,
        ctx: &mut NodeAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        self.content.analyze(handler, ctx)
    }
}

impl NodeAnalysis for AstNodeContent {
    fn analyze(
        &self,
        handler: &Handler,
        ctx: &mut NodeAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        match self {
            AstNodeContent::UseStatement(_) | AstNodeContent::IncludeStatement(_) => {
                // Already handled by [`ModuleDepGraph`]
            }
            AstNodeContent::Declaration(node) => node.analyze(handler, ctx)?,
            AstNodeContent::Expression(node) => node.analyze(handler, ctx)?,
            AstNodeContent::Error(_, _) => {}
        }
        Ok(())
    }
}

impl NodeAnalysis for Declaration {
    fn analyze(
        &self,
        handler: &Handler,
        ctx: &mut NodeAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        match self {
            Declaration::VariableDeclaration(decl_id) => decl_id.analyze(handler, ctx),
            Declaration::FunctionDeclaration(decl_id) => decl_id.analyze(handler, ctx),
            Declaration::TraitDeclaration(decl_id) => decl_id.analyze(handler, ctx),
            Declaration::StructDeclaration(decl_id) => decl_id.analyze(handler, ctx),
            Declaration::EnumDeclaration(decl_id) => decl_id.analyze(handler, ctx),
            Declaration::ImplTrait(decl_id) => decl_id.analyze(handler, ctx),
            Declaration::ImplSelf(decl_id) => decl_id.analyze(handler, ctx),
            Declaration::AbiDeclaration(decl_id) => decl_id.analyze(handler, ctx),
            Declaration::ConstantDeclaration(decl_id) => decl_id.analyze(handler, ctx),
            Declaration::StorageDeclaration(decl_id) => decl_id.analyze(handler, ctx),
            Declaration::TypeAliasDeclaration(decl_id) => decl_id.analyze(handler, ctx),
            Declaration::TraitTypeDeclaration(decl_id) => decl_id.analyze(handler, ctx),
        }
    }
}

impl NodeAnalysis for Expression {
    fn analyze(
        &self,
        handler: &Handler,
        ctx: &mut NodeAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        self.kind.analyze(handler, ctx)
    }
}

impl NodeAnalysis for ExpressionKind {
    fn analyze(
        &self,
        handler: &Handler,
        ctx: &mut NodeAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        match self {
            ExpressionKind::Error(_, _) => todo!(),
            ExpressionKind::Literal(_lit) => Ok(()),
            ExpressionKind::AmbiguousPathExpression(_) => Ok(()),
            ExpressionKind::FunctionApplication(_) => todo!(),
            ExpressionKind::LazyOperator(expr) => {
                let _ = expr.lhs.analyze(handler, ctx);
                let _ = expr.rhs.analyze(handler, ctx);
                Ok(())
            }
            ExpressionKind::AmbiguousVariableExpression(_) => Ok(()),
            ExpressionKind::Variable(_) => Ok(()),
            ExpressionKind::Tuple(_) => todo!(),
            ExpressionKind::TupleIndex(_) => Ok(()),
            ExpressionKind::Array(_) => todo!(),
            ExpressionKind::Struct(_expr) => {
                // TODO: Connect to the resolved symbol path struct node
                Ok(())
            }
            ExpressionKind::CodeBlock(block) => block.analyze(handler, ctx),
            ExpressionKind::If(expr) => {
                let _ = expr.condition.analyze(handler, ctx);
                let _ = expr.then.analyze(handler, ctx);
                if let Some(expr) = &expr.r#else {
                    let _ = expr.analyze(handler, ctx);
                }
                Ok(())
            }
            ExpressionKind::Match(expr) => {
                let _ = expr.value.analyze(handler, ctx);
                for branch in expr.branches.iter() {
                    let _ = branch.result.analyze(handler, ctx);
                    match &branch.scrutinee {
                        Scrutinee::Or { elems: _, span: _ } => todo!(),
                        Scrutinee::CatchAll { span: _ } => todo!(),
                        Scrutinee::Literal { value: _, span: _ } => todo!(),
                        Scrutinee::Variable { name: _, span: _ } => todo!(),
                        Scrutinee::AmbiguousSingleIdent(_) => todo!(),
                        Scrutinee::StructScrutinee {
                            struct_name: _,
                            fields: _,
                            span: _,
                        } => todo!(),
                        Scrutinee::EnumScrutinee {
                            call_path: _,
                            value: _,
                            span: _,
                        } => todo!(),
                        Scrutinee::Tuple { elems: _, span: _ } => todo!(),
                        Scrutinee::Error { spans: _, err: _ } => todo!(),
                    }
                }
                Ok(())
            }
            ExpressionKind::Asm(_expr) => Ok(()),
            ExpressionKind::MethodApplication(expr) => {
                for arg in expr.arguments.iter() {
                    let _ = arg.analyze(handler, ctx);
                }
                Ok(())
            }
            ExpressionKind::Subfield(_) => todo!(),
            ExpressionKind::DelineatedPath(_) => todo!(),
            ExpressionKind::AbiCast(_) => todo!(),
            ExpressionKind::ArrayIndex(_) => todo!(),
            ExpressionKind::StorageAccess(_) => todo!(),
            ExpressionKind::IntrinsicFunction(_) => Ok(()),
            ExpressionKind::WhileLoop(_) => todo!(),
            ExpressionKind::ForLoop(_) => todo!(),
            ExpressionKind::Break => todo!(),
            ExpressionKind::Continue => todo!(),
            ExpressionKind::Reassignment(_) => todo!(),
            ExpressionKind::ImplicitReturn(expr) => expr.analyze(handler, ctx),
            ExpressionKind::Return(_) => todo!(),
            ExpressionKind::Ref(_) => todo!(),
            ExpressionKind::Deref(_) => todo!(),
        }
    }
}

impl NodeAnalysis for ParsedDeclId<VariableDeclaration> {
    #[allow(clippy::map_entry)]
    fn analyze(
        &self,
        _handler: &Handler,
        ctx: &mut NodeAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        let id = self.unique_id();
        if !ctx.nodes.contains_key(&id) {
            let item_node = ctx.add_node(ParsedNodeDepGraphNode::Variable { decl_id: *self });
            ctx.nodes.insert(id, item_node);
        }
        Ok(())
    }
}

impl NodeAnalysis for ParsedDeclId<FunctionDeclaration> {
    fn analyze(
        &self,
        handler: &Handler,
        ctx: &mut NodeAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        handler.scope(|handler| {
            let node = ctx.get_node_for_fn_decl(self);
            if let Some(node) = node {
                ctx.node_stack.push(node);

                let fn_decl = ctx.engines.pe().get_function(self);
                let _ = fn_decl.analyze(handler, ctx);

                ctx.node_stack.pop();
            }
            Ok(())
        })
    }
}

impl NodeAnalysis for FunctionDeclaration {
    fn analyze(
        &self,
        handler: &Handler,
        ctx: &mut NodeAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        handler.scope(|handler| self.body.analyze(handler, ctx))
    }
}

impl NodeAnalysis for ParsedDeclId<TraitDeclaration> {
    fn analyze(
        &self,
        _handler: &Handler,
        _ctx: &mut NodeAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        Ok(())
    }
}

impl NodeAnalysis for ParsedDeclId<StructDeclaration> {
    #[allow(clippy::map_entry)]
    fn analyze(
        &self,
        _handler: &Handler,
        ctx: &mut NodeAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        let id = self.unique_id();
        if !ctx.nodes.contains_key(&id) {
            let item_node = ctx.add_node(ParsedNodeDepGraphNode::Struct { decl_id: *self });
            ctx.nodes.insert(id, item_node);
        }
        Ok(())
    }
}

impl NodeAnalysis for ParsedDeclId<EnumDeclaration> {
    fn analyze(
        &self,
        _handler: &Handler,
        _ctx: &mut NodeAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        todo!()
    }
}

impl NodeAnalysis for ParsedDeclId<ImplTrait> {
    fn analyze(
        &self,
        handler: &Handler,
        ctx: &mut NodeAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        let parsed_decl_engine = ctx.engines.pe();
        let impl_trait = parsed_decl_engine.get_impl_trait(self);

        // Lets create a graph node for the impl trait and for every item in the trait.
        ctx.push_nodes_for_impl_trait(self);

        // Now lets analyze each impl trait item.
        for (i, item) in impl_trait.items.iter().enumerate() {
            let _node = ctx.items_node_stack[i];
            item.analyze(handler, ctx)?;
        }

        // Clear the work-in-progress node stack.
        ctx.items_node_stack.clear();

        Ok(())
    }
}

impl NodeAnalysis for ParsedDeclId<ImplSelf> {
    fn analyze(
        &self,
        handler: &Handler,
        ctx: &mut NodeAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        let parsed_decl_engine = ctx.engines.pe();
        let impl_trait = parsed_decl_engine.get_impl_self(self);

        // Lets create a graph node for the impl self and for every item in the trait.
        ctx.push_nodes_for_impl_self(self);

        // Now lets analyze each impl trait item.
        for (i, item) in impl_trait.items.iter().enumerate() {
            let _node = ctx.items_node_stack[i];
            item.analyze(handler, ctx)?;
        }

        // Clear the work-in-progress node stack.
        ctx.items_node_stack.clear();

        Ok(())
    }
}

impl NodeAnalysis for ImplItem {
    fn analyze(
        &self,
        handler: &Handler,
        ctx: &mut NodeAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        match self {
            ImplItem::Fn(node) => {
                node.analyze(handler, ctx)?;
            }
            ImplItem::Constant(node) => {
                node.analyze(handler, ctx)?;
            }
            ImplItem::Type(node) => {
                node.analyze(handler, ctx)?;
            }
        }

        Ok(())
    }
}

impl NodeAnalysis for ParsedDeclId<AbiDeclaration> {
    fn analyze(
        &self,
        _handler: &Handler,
        _ctx: &mut NodeAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        todo!()
    }
}

impl NodeAnalysis for ParsedDeclId<ConstantDeclaration> {
    fn analyze(
        &self,
        _handler: &Handler,
        _ctx: &mut NodeAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        todo!()
    }
}

impl NodeAnalysis for ParsedDeclId<StorageDeclaration> {
    fn analyze(
        &self,
        _handler: &Handler,
        _ctx: &mut NodeAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        todo!()
    }
}

impl NodeAnalysis for ParsedDeclId<TypeAliasDeclaration> {
    fn analyze(
        &self,
        _handler: &Handler,
        _ctx: &mut NodeAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        todo!()
    }
}

impl NodeAnalysis for ParsedDeclId<TraitTypeDeclaration> {
    fn analyze(
        &self,
        _handler: &Handler,
        _ctx: &mut NodeAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        todo!()
    }
}

impl NodeAnalysis for CodeBlock {
    fn analyze(
        &self,
        handler: &Handler,
        ctx: &mut NodeAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        for node in self.contents.iter() {
            node.analyze(handler, ctx)?;
        }
        Ok(())
    }
}

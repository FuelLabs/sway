//! This is the flow graph, a graph which contains edges that represent possible steps of program
//! execution.

use crate::semantic_analysis::{ast_node::TypedStructField, TypedAstNode};
use crate::span::Span;
use crate::{semantic_analysis::ast_node::TypedEnumVariant, Ident};

use petgraph::{graph::EdgeIndex, prelude::NodeIndex};

mod namespace;
use namespace::ControlFlowNamespace;
pub(crate) use namespace::FunctionNamespaceEntry;

pub type EntryPoint = NodeIndex;
pub type ExitPoint = NodeIndex;

#[derive(Clone, Default)]
/// A graph that can be used to model the control flow of a fuel HLL program.
/// This graph is used as the basis for all of the algorithms in the control flow analysis portion
/// of the compiler.
pub struct ControlFlowGraph<'sc> {
    pub(crate) graph: Graph<'sc>,
    pub(crate) entry_points: Vec<NodeIndex>,
    pub(crate) namespace: ControlFlowNamespace<'sc>,
}

pub type Graph<'sc> = petgraph::Graph<ControlFlowGraphNode<'sc>, ControlFlowGraphEdge>;

#[derive(Clone)]
pub struct ControlFlowGraphEdge(String);

impl std::fmt::Debug for ControlFlowGraphEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::convert::From<&str> for ControlFlowGraphEdge {
    fn from(o: &str) -> Self {
        ControlFlowGraphEdge(o.to_string())
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum ControlFlowGraphNode<'sc> {
    OrganizationalDominator(String),
    #[allow(clippy::large_enum_variant)]
    ProgramNode(TypedAstNode<'sc>),
    EnumVariant {
        span: Span<'sc>,
        variant_name: String,
    },
    MethodDeclaration {
        span: Span<'sc>,
        method_name: Ident<'sc>,
    },
    StructField {
        struct_field_name: Ident<'sc>,
        span: Span<'sc>,
    },
}

impl<'sc> std::fmt::Debug for ControlFlowGraphNode<'sc> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            ControlFlowGraphNode::OrganizationalDominator(s) => s.to_string(),
            ControlFlowGraphNode::ProgramNode(node) => format!("{:?}", node),
            ControlFlowGraphNode::EnumVariant { variant_name, .. } => {
                format!("Enum variant {}", variant_name.to_string())
            }
            ControlFlowGraphNode::MethodDeclaration { method_name, .. } => {
                format!("Method {}", method_name.primary_name().to_string())
            }
            ControlFlowGraphNode::StructField {
                struct_field_name, ..
            } => {
                format!(
                    "Struct field {}",
                    struct_field_name.primary_name().to_string()
                )
            }
        };
        f.write_str(&text)
    }
}

impl<'sc> std::convert::From<&TypedAstNode<'sc>> for ControlFlowGraphNode<'sc> {
    fn from(other: &TypedAstNode<'sc>) -> Self {
        ControlFlowGraphNode::ProgramNode(other.clone())
    }
}

impl<'sc> std::convert::From<&TypedEnumVariant<'sc>> for ControlFlowGraphNode<'sc> {
    fn from(other: &TypedEnumVariant<'sc>) -> Self {
        ControlFlowGraphNode::EnumVariant {
            variant_name: other.name.primary_name().to_string(),
            span: other.span.clone(),
        }
    }
}

impl<'sc> std::convert::From<&TypedStructField<'sc>> for ControlFlowGraphNode<'sc> {
    fn from(other: &TypedStructField<'sc>) -> Self {
        ControlFlowGraphNode::StructField {
            struct_field_name: other.name.clone(),
            span: other.span.clone(),
        }
    }
}
impl std::convert::From<String> for ControlFlowGraphNode<'_> {
    fn from(other: String) -> Self {
        ControlFlowGraphNode::OrganizationalDominator(other)
    }
}

impl std::convert::From<&str> for ControlFlowGraphNode<'_> {
    fn from(other: &str) -> Self {
        ControlFlowGraphNode::OrganizationalDominator(other.to_string())
    }
}

impl<'sc> ControlFlowGraph<'sc> {
    pub(crate) fn add_edge_from_entry(&mut self, to: NodeIndex, label: ControlFlowGraphEdge) {
        for entry in &self.entry_points {
            self.graph.add_edge(*entry, to, label.clone());
        }
    }
    pub(crate) fn add_node(&mut self, node: ControlFlowGraphNode<'sc>) -> NodeIndex {
        self.graph.add_node(node)
    }
    pub(crate) fn add_edge(
        &mut self,
        from: NodeIndex,
        to: NodeIndex,
        edge: ControlFlowGraphEdge,
    ) -> EdgeIndex {
        self.graph.add_edge(from, to, edge)
    }

    #[allow(dead_code)]
    /// Prints out graphviz for this graph
    pub(crate) fn visualize(&self) {
        use petgraph::dot::Dot;
        println!("{:?}", Dot::with_config(&self.graph, &[]));
    }
}

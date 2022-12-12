//! This is the flow graph, a graph which contains edges that represent possible steps of program
//! execution.

use std::collections::HashMap;

use crate::{
    declaration_engine::{de_get_function, DeclarationId},
    language::ty::{self, GetDeclIdent},
    Ident,
};

use sway_types::{span::Span, IdentUnique};

use petgraph::{graph::EdgeIndex, prelude::NodeIndex};

mod namespace;
use namespace::ControlFlowNamespace;
pub(crate) use namespace::FunctionNamespaceEntry;

pub type EntryPoint = NodeIndex;
pub type ExitPoint = NodeIndex;

#[derive(Clone, Default)]
/// A graph that can be used to model the control flow of a Sway program.
/// This graph is used as the basis for all of the algorithms in the control flow analysis portion
/// of the compiler.
pub struct ControlFlowGraph {
    pub(crate) graph: Graph,
    pub(crate) entry_points: Vec<NodeIndex>,
    pub(crate) pending_entry_points_edges: Vec<(NodeIndex, ControlFlowGraphEdge)>,
    pub(crate) namespace: ControlFlowNamespace,
    pub(crate) decls: HashMap<IdentUnique, NodeIndex>,
}

pub type Graph = petgraph::Graph<ControlFlowGraphNode, ControlFlowGraphEdge>;

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
pub enum ControlFlowGraphNode {
    OrganizationalDominator(String),
    #[allow(clippy::large_enum_variant)]
    ProgramNode(ty::TyAstNode),
    EnumVariant {
        variant_name: Ident,
        is_public: bool,
    },
    MethodDeclaration {
        span: Span,
        method_name: Ident,
        method_decl_id: DeclarationId,
    },
    StructField {
        struct_field_name: Ident,
        span: Span,
    },
    StorageField {
        field_name: Ident,
    },
}

impl GetDeclIdent for ControlFlowGraphNode {
    fn get_decl_ident(&self) -> Option<Ident> {
        match self {
            ControlFlowGraphNode::OrganizationalDominator(_) => None,
            ControlFlowGraphNode::ProgramNode(node) => node.get_decl_ident(),
            ControlFlowGraphNode::EnumVariant { variant_name, .. } => Some(variant_name.clone()),
            ControlFlowGraphNode::MethodDeclaration { method_name, .. } => {
                Some(method_name.clone())
            }
            ControlFlowGraphNode::StructField {
                struct_field_name, ..
            } => Some(struct_field_name.clone()),
            ControlFlowGraphNode::StorageField { field_name, .. } => Some(field_name.clone()),
        }
    }
}

impl std::fmt::Debug for ControlFlowGraphNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            ControlFlowGraphNode::OrganizationalDominator(s) => s.to_string(),
            ControlFlowGraphNode::ProgramNode(node) => format!("{:?}", node),
            ControlFlowGraphNode::EnumVariant { variant_name, .. } => {
                format!("Enum variant {}", variant_name)
            }
            ControlFlowGraphNode::MethodDeclaration {
                method_name,
                method_decl_id,
                ..
            } => {
                let method = de_get_function(method_decl_id.clone(), &Span::dummy()).unwrap();
                if let Some(implementing_type) = method.implementing_type {
                    format!(
                        "Method {}.{}",
                        implementing_type
                            .get_decl_ident()
                            .map_or(String::from(""), |f| f.as_str().to_string()),
                        method_name.as_str()
                    )
                } else {
                    format!("Method {}", method_name.as_str())
                }
            }
            ControlFlowGraphNode::StructField {
                struct_field_name, ..
            } => {
                format!("Struct field {}", struct_field_name.as_str())
            }
            ControlFlowGraphNode::StorageField { field_name } => {
                format!("Storage field {}", field_name.as_str())
            }
        };
        f.write_str(&text)
    }
}
impl std::convert::From<&ty::TyStorageField> for ControlFlowGraphNode {
    fn from(other: &ty::TyStorageField) -> Self {
        ControlFlowGraphNode::StorageField {
            field_name: other.name.clone(),
        }
    }
}

impl std::convert::From<&ty::TyAstNode> for ControlFlowGraphNode {
    fn from(other: &ty::TyAstNode) -> Self {
        ControlFlowGraphNode::ProgramNode(other.clone())
    }
}

impl std::convert::From<&ty::TyStructField> for ControlFlowGraphNode {
    fn from(other: &ty::TyStructField) -> Self {
        ControlFlowGraphNode::StructField {
            struct_field_name: other.name.clone(),
            span: other.span.clone(),
        }
    }
}
impl std::convert::From<String> for ControlFlowGraphNode {
    fn from(other: String) -> Self {
        ControlFlowGraphNode::OrganizationalDominator(other)
    }
}

impl std::convert::From<&str> for ControlFlowGraphNode {
    fn from(other: &str) -> Self {
        other.to_string().into()
    }
}

impl ControlFlowGraph {
    pub(crate) fn add_edge_from_entry(&mut self, to: NodeIndex, label: ControlFlowGraphEdge) {
        self.pending_entry_points_edges.push((to, label));
    }
    pub(crate) fn add_node(&mut self, node: ControlFlowGraphNode) -> NodeIndex {
        let ident_opt = node.get_decl_ident();
        let node_index = self.graph.add_node(node);
        if let Some(ident) = ident_opt {
            self.decls.insert(ident.into(), node_index);
        }
        node_index
    }
    pub(crate) fn add_edge(
        &mut self,
        from: NodeIndex,
        to: NodeIndex,
        edge: ControlFlowGraphEdge,
    ) -> EdgeIndex {
        self.graph.add_edge(from, to, edge)
    }

    pub(crate) fn connect_pending_entry_edges(&mut self) {
        for entry in &self.entry_points {
            for (to, label) in &self.pending_entry_points_edges {
                self.graph.add_edge(*entry, *to, label.clone());
            }
        }
        self.pending_entry_points_edges.clear();
    }

    pub(crate) fn get_node_from_decl(&self, cfg_node: &ControlFlowGraphNode) -> Option<NodeIndex> {
        if let Some(ident) = cfg_node.get_decl_ident() {
            self.decls.get(&ident.into()).cloned()
        } else {
            None
        }
    }

    /// Prints out GraphViz DOT format for this graph.
    pub(crate) fn visualize(&self) {
        use petgraph::dot::{Config, Dot};
        tracing::info!(
            "{:?}",
            Dot::with_config(&self.graph, &[Config::EdgeNoLabel])
        );
    }
}

impl ControlFlowGraphNode {
    pub(crate) fn from_enum_variant(
        other: &ty::TyEnumVariant,
        is_public: bool,
    ) -> ControlFlowGraphNode {
        ControlFlowGraphNode::EnumVariant {
            variant_name: other.name.clone(),
            is_public,
        }
    }
}

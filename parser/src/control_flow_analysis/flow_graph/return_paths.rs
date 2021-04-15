//! This is the flow graph, a graph which contains edges that represent possible steps of program
//! execution.

use crate::error::*;
use crate::{
    parse_tree::Visibility,
    semantics::ast_node::{TypedEnumVariant, TypedExpressionVariant, TypedTraitDeclaration},
    Ident, TreeType,
};

use crate::{
    semantics::{
        ast_node::{
            TypedCodeBlock, TypedDeclaration, TypedEnumDeclaration, TypedExpression,
            TypedFunctionDeclaration, TypedReassignment, TypedVariableDeclaration, TypedWhileLoop,
        },
        TypedAstNode, TypedAstNodeContent, TypedParseTree,
    },
    CompileWarning, Warning,
};

use pest::Span;
use petgraph::algo::has_path_connecting;
use petgraph::{graph::EdgeIndex, prelude::NodeIndex};

use super::{ControlFlowGraph, EntryPoint, ExitPoint};

impl<'sc> ControlFlowGraph<'sc> {
    /// This function  looks through the control flow graph and ensures that all paths that are
    /// required to return a value do, indeed, return a value of the correct type.
    /// It does this by checking every function declaration in both the methods namespace
    /// and the functions namespace and validating that all paths leading to the function exit node
    /// return the same type. Additionally, if a function has a return type, all paths must indeed
    /// lead to the function exit node.
    pub(crate) fn analyze_return_paths(&self) -> Vec<CompileError<'sc>> {
        let mut errors = vec![];
        for (name, (entry_point, exit_point)) in &self.namespace.function_namespace {
            // For every node connected to the entry point
            errors.append(&mut self.ensure_all_paths_reach_exit(*entry_point, *exit_point));
        }
        errors
    }
    fn ensure_all_paths_reach_exit(
        &self,
        entry_point: EntryPoint,
        exit_point: ExitPoint,
    ) -> Vec<CompileError<'sc>> {
        let mut rovers = vec![entry_point];
        let mut errors = vec![];
        let mut max_iterations = 50;
        while rovers.len() >= 1 && rovers[0] != exit_point && max_iterations > 0 {
            max_iterations -= 1;
            dbg!(&rovers);
            rovers = rovers
                .into_iter()
                .filter(|idx| *idx != exit_point)
                .collect();
            let mut next_rovers = vec![];
            for rover in rovers {
                let mut neighbors = self
                    .graph
                    .neighbors_directed(rover, petgraph::Direction::Outgoing)
                    .collect::<Vec<_>>();
                if neighbors.is_empty() {
                    //j                    errors.push(todo!("Path does not return error"));
                }
                next_rovers.append(&mut neighbors);
            }
            rovers = next_rovers;
        }

        errors
    }
}

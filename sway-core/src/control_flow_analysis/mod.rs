//!
//! This module contains all of the logic related to control flow analysis.
//!
//! # Synopsis of Dead-Code Analysis Algorithm
//! The dead code analysis algorithm constructs a node for every declaration, expression, and
//! statement. Then, from the entry points of the AST, we begin drawing edges along the control
//! flow path. If a declaration is instantiated, we draw an edge to it. If an expression or
//! statement is executed, an edge is drawn to it. Finally, we trace the edges from the entry
//! points of the AST. If there are no paths from any entry point to a node, then it is either a
//! dead declaration or an unreachable expression or statement.
//!
//! See the Terms section for details on how entry points are determined.
//!
//! # Synopsis of Return-Path Analysis Algorithm
//! The graph constructed for this algorithm does not go into the details of the contents of any
//! declaration except for function declarations. Inside of every function, it traces the execution
//! path along to ensure that all reachable paths do indeed return a value. We don't need to type
//! check the value that is returned, since type checking of return statements happens in the type
//! checking stage. Here, we know all present return statements have the right type, and we just
//! need to verify that all paths do indeed contain a return statement.
//!
//!
//! # # Terms
//! # # # Node
//! A node is any [crate::semantic_analysis::TyAstNode], with some
//! [crate::semantic_analysis::TyAstNodeContent]. # # # Dominating nodes
//! A dominating node is a node which all previous nodes pass through. These are what we are
//! concerned about in control flow analysis. More formally,
//! A node _M_ dominates a node _N_ if every path from the entry that reaches node _N_ has to pass
//! through node _M_.
//!
//! # # # Reachability
//! A node _N_ is reachable if there is a path to it from any one of the tree's entry points.
//!
//! # # # Entry Points
//! The entry points to an AST depend on what type of AST it is. If it is a predicate or script,
//! then the main function is the sole entry point. If it is a library or contract, then public
//! functions or declarations are entry points.
mod analyze_return_paths;
mod dead_code_analysis;
mod flow_graph;
pub use analyze_return_paths::*;
pub use dead_code_analysis::*;
pub use flow_graph::*;

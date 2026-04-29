//! Shared traversal context and execution helpers for AST-to-token passes.

use crate::core::{token::TokenIdent, token_map::TokenMap};
use rayon::{ThreadPool, ThreadPoolBuilder};
use rayon_cond::CondIterator;
use std::sync::OnceLock;
use sway_core::{namespace::Package, Engines};

pub(crate) mod dependency;
pub(crate) mod lexed_tree;
pub(crate) mod parsed_tree;
pub(crate) mod typed_tree;

pub struct ParseContext<'a> {
    tokens: &'a TokenMap,
    pub engines: &'a Engines,
    namespace: &'a Package,
}

impl<'a> ParseContext<'a> {
    pub fn new(tokens: &'a TokenMap, engines: &'a Engines, namespace: &'a Package) -> Self {
        Self {
            tokens,
            engines,
            namespace,
        }
    }

    pub fn ident(&self, ident: &sway_types::Ident) -> TokenIdent {
        TokenIdent::new(ident, self.engines.se())
    }
}

/// The `Parse` trait is used to parse tokens from an AST during traversal.
pub trait Parse {
    fn parse(&self, ctx: &ParseContext);
}

/// Determines the threshold a collection must meet to be processed in parallel.
const PARALLEL_THRESHOLD: usize = 8;
// Typed-token traversal can recurse through large generated std ASTs on Rayon workers.
const TRAVERSAL_THREAD_STACK_SIZE: usize = 32 * 1024 * 1024;

static TRAVERSAL_THREAD_POOL: OnceLock<ThreadPool> = OnceLock::new();

fn traversal_thread_pool() -> &'static ThreadPool {
    TRAVERSAL_THREAD_POOL.get_or_init(|| {
        ThreadPoolBuilder::new()
            .thread_name(|index| format!("sway-lsp-traverse-{index}"))
            // Typed-token traversal can recurse deeply through large generated std ASTs.
            .stack_size(TRAVERSAL_THREAD_STACK_SIZE)
            .build()
            .expect("failed to build sway-lsp traversal thread pool")
    })
}

/// Runs traversal work on a dedicated Rayon pool with a larger stack than the global pool.
///
/// This keeps semantic-token and dependency traversal from aborting the process when
/// deeply nested generated std ASTs overflow the default worker stack.
pub fn with_traversal_thread_pool<T: Send>(action: impl FnOnce() -> T + Send) -> T {
    traversal_thread_pool().install(action)
}

/// Iterates over items, choosing parallel or sequential execution based on size.
pub fn adaptive_iter<T, F>(items: &[T], action: F)
where
    T: Sync + Send,          // Required for parallel processing
    F: Fn(&T) + Sync + Send, // Action to be applied to each item
{
    // Determine if the length meets the parallel threshold
    let use_parallel = items.len() >= PARALLEL_THRESHOLD;

    // Create a conditional iterator based on the use_parallel flag
    CondIterator::new(items, use_parallel).for_each(action);
}

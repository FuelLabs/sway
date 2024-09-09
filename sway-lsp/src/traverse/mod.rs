use crate::core::{token::TokenIdent, token_map::TokenMap};
use rayon_cond::CondIterator;
use sway_core::{namespace::Module, Engines};

pub(crate) mod dependency;
pub(crate) mod lexed_tree;
pub(crate) mod parsed_tree;
pub(crate) mod typed_tree;

pub struct ParseContext<'a> {
    tokens: &'a TokenMap,
    pub engines: &'a Engines,
    namespace: &'a Module,
}

impl<'a> ParseContext<'a> {
    pub fn new(tokens: &'a TokenMap, engines: &'a Engines, namespace: &'a Module) -> Self {
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

use petgraph::{Directed, stable_graph::{DefaultIx, IndexType, NodeIndex}, visit, EdgeType};

use crate::engine_threading::*;

#[derive(Clone)]
pub(crate) struct Graph<'a, N, E, Ty = Directed, Ix = DefaultIx> {
    graph: petgraph::Graph<N, E, Ty, Ix>,
    engines: Engines<'a>,
}

impl<'a, N, E> Graph<'a, N, E> {
    pub(super) fn new(engines: Engines<'a>) -> Graph<'a, N, E> {
        Graph {
            graph: Default::default(),
            engines,
        }
    }
}

impl<'a, N, E, Ty, Ix> visit::NodeIndexable for Graph<'a, N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    #[inline]
    fn node_bound(&self) -> usize {
        self.node_count()
    }
    #[inline]
    fn to_index(&self, ix: NodeIndex<Ix>) -> usize {
        ix.index()
    }
    #[inline]
    fn from_index(&self, ix: usize) -> Self::NodeId {
        NodeIndex::new(ix)
    }
}

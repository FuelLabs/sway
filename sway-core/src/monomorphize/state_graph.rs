use petgraph::Graph;

use crate::monomorphize::priv_prelude::*;

pub(crate) struct StateGraph {
    graph: Graph<MonoItem, ()>
}

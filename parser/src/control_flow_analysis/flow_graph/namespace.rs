use super::{EntryPoint, ExitPoint};
use crate::Ident;
use petgraph::prelude::NodeIndex;
use std::collections::HashMap;

#[derive(Default)]
/// This namespace holds mappings from various declarations to their indexes in the graph. This is
/// used for connecting those vertices when the declarations are instantiated.
///
/// Since control flow happens after type checking, we are not concerned about things being out
/// of scope at this point, as that would have been caught earlier and aborted the compilation
/// process.
pub struct ControlFlowNamespace<'sc> {
    function_namespace: HashMap<Ident<'sc>, (EntryPoint, ExitPoint)>,
    enum_namespace: HashMap<Ident<'sc>, (NodeIndex, HashMap<Ident<'sc>, NodeIndex>)>,
}

impl<'sc> ControlFlowNamespace<'sc> {
    pub(crate) fn get_function(&self, ident: &Ident<'sc>) -> Option<&(EntryPoint, ExitPoint)> {
        self.function_namespace.get(ident)
    }
    pub(crate) fn insert_function(&mut self, ident: Ident<'sc>, points: (EntryPoint, ExitPoint)) {
        self.function_namespace.insert(ident, points);
    }
    pub(crate) fn insert_enum(
        &mut self,
        enum_name: Ident<'sc>,
        enum_decl_index: NodeIndex,
        variant_name: Ident<'sc>,
        variant_index: NodeIndex,
    ) {
        match self.enum_namespace.get_mut(&enum_name) {
            Some((_ix, variants)) => {
                variants.insert(variant_name, variant_index);
            }
            None => {
                let variant_space = {
                    let mut map = HashMap::new();
                    map.insert(variant_name, variant_index);
                    map
                };
                self.enum_namespace
                    .insert(enum_name, (enum_decl_index, variant_space));
            }
        }
    }
    pub(crate) fn find_enum_variant_index(
        &self,
        enum_name: &Ident<'sc>,
        variant_name: &Ident<'sc>,
    ) -> Option<(NodeIndex, NodeIndex)> {
        let (enum_ix, enum_decl) = self.enum_namespace.get(enum_name)?;
        Some((enum_ix.clone(), enum_decl.get(variant_name)?.clone()))
    }
}

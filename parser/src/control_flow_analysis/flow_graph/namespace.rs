use super::{EntryPoint, ExitPoint};
use crate::{types::ResolvedType, Ident};
use petgraph::prelude::NodeIndex;
use std::collections::HashMap;

#[derive(Default, Clone)]
/// Represents a single entry in the [ControlFlowNamespace]'s function namespace. Contains various
/// metadata about a function including its node indexes in the graph, its return type, and more.
/// Used to both perform control flow analysis on functions as well as produce good error messages.
pub(crate) struct FunctionNamespaceEntry<'sc> {
    pub(crate) entry_point: EntryPoint,
    pub(crate) exit_point: ExitPoint,
    pub(crate) return_type: ResolvedType<'sc>,
}

#[derive(Default, Clone)]
pub(crate) struct StructNamespaceEntry<'sc> {
    pub(crate) struct_decl_ix: NodeIndex,
    pub(crate) fields: HashMap<Ident<'sc>, NodeIndex>,
}

#[derive(Default, Clone)]
/// This namespace holds mappings from various declarations to their indexes in the graph. This is
/// used for connecting those vertices when the declarations are instantiated.
///
/// Since control flow happens after type checking, we are not concerned about things being out
/// of scope at this point, as that would have been caught earlier and aborted the compilation
/// process.
pub struct ControlFlowNamespace<'sc> {
    pub(crate) function_namespace: HashMap<Ident<'sc>, FunctionNamespaceEntry<'sc>>,
    pub(crate) enum_namespace: HashMap<Ident<'sc>, (NodeIndex, HashMap<Ident<'sc>, NodeIndex>)>,
    pub(crate) trait_namespace: HashMap<Ident<'sc>, NodeIndex>,
    /// This is a mapping from trait name to method names and their node indexes
    pub(crate) trait_method_namespace: HashMap<Ident<'sc>, HashMap<Ident<'sc>, NodeIndex>>,
    /// This is a mapping from struct name to field names and their node indexes
    pub(crate) struct_namespace: HashMap<Ident<'sc>, StructNamespaceEntry<'sc>>,
}

impl<'sc> ControlFlowNamespace<'sc> {
    pub(crate) fn get_function(&self, ident: &Ident<'sc>) -> Option<&FunctionNamespaceEntry<'sc>> {
        self.function_namespace.get(ident)
    }
    pub(crate) fn insert_function(
        &mut self,
        ident: Ident<'sc>,
        entry: FunctionNamespaceEntry<'sc>,
    ) {
        self.function_namespace.insert(ident, entry);
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

    pub(crate) fn add_trait(&mut self, trait_name: Ident<'sc>, trait_idx: NodeIndex) {
        self.trait_namespace.insert(trait_name, trait_idx);
    }

    pub(crate) fn find_trait(&self, name: &Ident<'sc>) -> Option<&NodeIndex> {
        self.trait_namespace.get(name)
    }

    pub(crate) fn insert_trait_methods(
        &mut self,
        trait_name: Ident<'sc>,
        methods: Vec<(Ident<'sc>, NodeIndex)>,
    ) {
        match self.trait_method_namespace.get_mut(&trait_name) {
            Some(methods_ns) => {
                for (name, ix) in methods {
                    methods_ns.insert(name, ix);
                }
            }
            None => {
                let mut ns = HashMap::default();
                for (name, ix) in methods {
                    ns.insert(name, ix);
                }
                self.trait_method_namespace.insert(trait_name, ns);
            }
        }
    }

    pub(crate) fn insert_struct(
        &mut self,
        struct_name: Ident<'sc>,
        declaration_node: NodeIndex,
        field_nodes: Vec<(Ident<'sc>, NodeIndex)>,
    ) {
        let entry = StructNamespaceEntry {
            struct_decl_ix: declaration_node,
            fields: field_nodes.into_iter().collect(),
        };
        self.struct_namespace.insert(struct_name, entry);
    }
    pub(crate) fn find_struct_decl(&self, struct_name: &Ident<'sc>) -> Option<&NodeIndex> {
        self.struct_namespace
            .get(struct_name)
            .map(|StructNamespaceEntry { struct_decl_ix, .. }| struct_decl_ix)
    }
    pub(crate) fn find_struct_field_idx(
        &self,
        struct_name: &Ident<'sc>,
        field_name: &Ident<'sc>,
    ) -> Option<&NodeIndex> {
        self.struct_namespace
            .get(struct_name)?
            .fields
            .get(field_name)
    }
}

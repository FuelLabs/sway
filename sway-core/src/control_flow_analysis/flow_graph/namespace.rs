use super::{EntryPoint, ExitPoint};
use crate::{
    language::{ty, CallPath},
    type_system::TypeInfo,
    Ident,
};
use petgraph::prelude::NodeIndex;
use std::collections::HashMap;

#[derive(Default, Clone)]
/// Represents a single entry in the [ControlFlowNamespace]'s function namespace. Contains various
/// metadata about a function including its node indexes in the graph, its return type, and more.
/// Used to both perform control flow analysis on functions as well as produce good error messages.
pub(crate) struct FunctionNamespaceEntry {
    pub(crate) entry_point: EntryPoint,
    pub(crate) exit_point: ExitPoint,
    pub(crate) return_type: TypeInfo,
}

#[derive(Default, Clone)]
pub(crate) struct StructNamespaceEntry {
    pub(crate) struct_decl_ix: NodeIndex,
    pub(crate) fields: HashMap<String, NodeIndex>,
}

#[derive(Default, Clone)]
/// This namespace holds mappings from various declarations to their indexes in the graph. This is
/// used for connecting those vertices when the declarations are instantiated.
///
/// Since control flow happens after type checking, we are not concerned about things being out
/// of scope at this point, as that would have been caught earlier and aborted the compilation
/// process.
pub struct ControlFlowNamespace {
    pub(crate) function_namespace: HashMap<Ident, FunctionNamespaceEntry>,
    pub(crate) enum_namespace: HashMap<Ident, (NodeIndex, HashMap<Ident, NodeIndex>)>,
    pub(crate) trait_namespace: HashMap<CallPath, NodeIndex>,
    /// This is a mapping from trait name to method names and their node indexes
    pub(crate) trait_method_namespace: HashMap<CallPath, HashMap<Ident, NodeIndex>>,
    /// This is a mapping from struct name to field names and their node indexes
    /// TODO this should be an Ident and not a String, switch when static spans are implemented
    pub(crate) struct_namespace: HashMap<String, StructNamespaceEntry>,
    pub(crate) const_namespace: HashMap<Ident, NodeIndex>,
    pub(crate) storage: HashMap<Ident, NodeIndex>,
}

impl ControlFlowNamespace {
    pub(crate) fn get_function(&self, ident: &Ident) -> Option<&FunctionNamespaceEntry> {
        self.function_namespace.get(ident)
    }
    pub(crate) fn insert_function(&mut self, ident: Ident, entry: FunctionNamespaceEntry) {
        self.function_namespace.insert(ident, entry);
    }
    pub(crate) fn get_constant(&self, ident: &Ident) -> Option<&NodeIndex> {
        self.const_namespace.get(ident)
    }
    pub(crate) fn insert_constant(&mut self, const_name: Ident, declaration_node: NodeIndex) {
        self.const_namespace.insert(const_name, declaration_node);
    }
    pub(crate) fn insert_enum(&mut self, enum_name: Ident, enum_decl_index: NodeIndex) {
        self.enum_namespace
            .insert(enum_name, (enum_decl_index, HashMap::new()));
    }
    pub(crate) fn insert_enum_variant(
        &mut self,
        enum_name: Ident,
        enum_decl_index: NodeIndex,
        variant_name: Ident,
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
    pub(crate) fn find_enum(&self, enum_name: &Ident) -> Option<&NodeIndex> {
        self.enum_namespace.get(enum_name).map(|f| &f.0)
    }
    pub(crate) fn find_enum_variant_index(
        &self,
        enum_name: &Ident,
        variant_name: &Ident,
    ) -> Option<(NodeIndex, NodeIndex)> {
        let (enum_ix, enum_decl) = self.enum_namespace.get(enum_name)?;
        Some((*enum_ix, *enum_decl.get(variant_name)?))
    }

    pub(crate) fn add_trait(&mut self, trait_name: CallPath, trait_idx: NodeIndex) {
        self.trait_namespace.insert(trait_name, trait_idx);
    }

    pub(crate) fn find_trait(&self, name: &CallPath) -> Option<&NodeIndex> {
        self.trait_namespace.get(name)
    }

    pub(crate) fn insert_trait_methods(
        &mut self,
        trait_name: CallPath,
        methods: Vec<(Ident, NodeIndex)>,
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

    pub(crate) fn insert_storage(&mut self, field_nodes: Vec<(ty::TyStorageField, NodeIndex)>) {
        for (field, node) in field_nodes {
            self.storage.insert(field.name, node);
        }
    }

    pub(crate) fn insert_struct(
        &mut self,
        struct_name: String,
        declaration_node: NodeIndex,
        field_nodes: Vec<(Ident, NodeIndex)>,
    ) {
        let entry = StructNamespaceEntry {
            struct_decl_ix: declaration_node,
            fields: field_nodes
                .into_iter()
                .map(|(ident, ix)| (ident.as_str().to_string(), ix))
                .collect(),
        };
        self.struct_namespace.insert(struct_name, entry);
    }
    pub(crate) fn find_struct_decl(&self, struct_name: &str) -> Option<&NodeIndex> {
        self.struct_namespace
            .get(struct_name)
            .map(|StructNamespaceEntry { struct_decl_ix, .. }| struct_decl_ix)
    }
    pub(crate) fn find_struct_field_idx(
        &self,
        struct_name: &str,
        field_name: &str,
    ) -> Option<&NodeIndex> {
        self.struct_namespace
            .get(struct_name)?
            .fields
            .get(field_name)
    }
}

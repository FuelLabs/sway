use crate::{
    language::CallPath,
    type_system::{look_up_type_id, CopyTypes, TypeId},
    TyFunctionDeclaration, TypeInfo, TypeMapping,
};

type TraitName = CallPath;

// This cannot be a HashMap because of how TypeInfo's are handled.
//
// In Rust, in general, a custom type should uphold the invariant
// that PartialEq and Hash produce consistent results. i.e. for
// two objects, their hash value is equal if and only if they are
// equal under the PartialEq trait.
//
// For TypeInfo, this means that if you have:
//
// ```ignore
// 1: u64
// 2: u64
// 3: Ref(1)
// 4: Ref(2)
// ```
//
// 1, 2, 3, 4 are equal under PartialEq and their hashes are the same
// value.
//
// However, we need this structure to be able to maintain the
// difference between 3 and 4, as in practice, 1 and 2 might not yet
// be resolved.
type TraitMapInner = im::Vector<((TraitName, TypeId), TraitMethods)>;
type TraitMethods = im::HashMap<String, TyFunctionDeclaration>;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct TraitMap {
    trait_map: TraitMapInner,
}

impl TraitMap {
    pub(crate) fn insert(
        &mut self,
        trait_name: TraitName,
        incoming_type_id: TypeId,
        methods: Vec<TyFunctionDeclaration>,
    ) {
        let mut methods_map = im::HashMap::new();
        for method in methods.into_iter() {
            methods_map.insert(method.name.as_str().to_string(), method);
        }
        self.trait_map
            .push_back(((trait_name, incoming_type_id), methods_map));
    }

    pub(crate) fn extend(&mut self, other: TraitMap) {
        for ((trait_name, type_implementing_for), methods) in other.trait_map.into_iter() {
            self.insert(
                trait_name,
                type_implementing_for,
                methods.values().cloned().collect(),
            );
        }
    }

    pub(crate) fn get_call_path_and_type_info(
        &self,
        incoming_type_id: TypeId,
    ) -> Vec<((TraitName, TypeId), Vec<TyFunctionDeclaration>)> {
        let mut ret = vec![];
        for ((call_path, map_type_id), methods) in self.trait_map.iter() {
            if look_up_type_id(incoming_type_id).is_subset_of(&look_up_type_id(*map_type_id)) {
                ret.push((
                    (call_path.clone(), *map_type_id),
                    methods.values().cloned().collect(),
                ));
            }
        }
        ret
    }

    pub(crate) fn get_methods_for_type(
        &self,
        incoming_type_id: TypeId,
    ) -> Vec<TyFunctionDeclaration> {
        let mut methods = vec![];
        // small performance gain in bad case
        if look_up_type_id(incoming_type_id) == TypeInfo::ErrorRecovery {
            return methods;
        }
        for ((_, map_type_id), trait_methods) in self.trait_map.iter() {
            if look_up_type_id(incoming_type_id).is_subset_of(&look_up_type_id(*map_type_id)) {
                let type_mapping =
                    TypeMapping::from_superset_and_subset(*map_type_id, incoming_type_id);
                let mut trait_methods = trait_methods.values().cloned().collect::<Vec<_>>();
                trait_methods
                    .iter_mut()
                    .for_each(|x| x.copy_types(&type_mapping));
                methods.append(&mut trait_methods);
            }
        }
        methods
    }
}

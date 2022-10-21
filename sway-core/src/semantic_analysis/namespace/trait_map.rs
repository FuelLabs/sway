use crate::{
    language::{ty, CallPath},
    type_system::{look_up_type_id, CopyTypes, TypeId},
    TypeInfo, TypeMapping,
};

type TraitName = CallPath;
/// Map of function name to [TyFunctionDeclaration](ty::TyFunctionDeclaration)
type TraitMethods = im::HashMap<String, ty::TyFunctionDeclaration>;
/// Map of trait name and type to [TraitMethods].
type TraitImpls = im::HashMap<(TraitName, TypeId), TraitMethods>;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct TraitMap {
    trait_impls: TraitImpls,
}

impl TraitMap {
    /// Given a [TraitName] `trait_name`, [TypeId] `type_id`, and list of
    /// [TyFunctionDeclaration](ty::TyFunctionDeclaration) `methods`, inserts
    /// `methods` into the [TraitMap] with the key `(trait_name, type_id)`.
    ///
    /// This method is as conscious as possible of existing entries in the
    /// [TraitMap], and tries to append `methods` to an existing list of
    /// [TyFunctionDeclaration](ty::TyFunctionDeclaration) for the key
    /// `(trait_name, type_id)` whenever possible.
    pub(crate) fn insert(
        &mut self,
        trait_name: TraitName,
        type_id: TypeId,
        methods: Vec<ty::TyFunctionDeclaration>,
    ) {
        let trait_methods: TraitMethods = methods
            .into_iter()
            .map(|method| (method.name.as_str().to_string(), method))
            .collect();
        let trait_impls: TraitImpls = vec![((trait_name, type_id), trait_methods)]
            .into_iter()
            .collect();
        let trait_map = TraitMap { trait_impls };
        self.extend(trait_map);
    }

    pub(crate) fn insert_for_type(&mut self, type_id: TypeId) {
        self.extend(self.filter_by_type(type_id));
    }

    pub(crate) fn extend(&mut self, other: TraitMap) {
        for (key, other_trait_methods) in other.trait_impls.into_iter() {
            self.trait_impls
                .entry(key)
                .or_insert(other_trait_methods.clone())
                .extend(other_trait_methods.into_iter());
        }
    }

    pub(crate) fn filter_by_type(&self, type_id: TypeId) -> TraitMap {
        let mut trait_map = TraitMap {
            trait_impls: Default::default(),
        };
        for ((map_trait_name, map_type_id), map_trait_methods) in self.trait_impls.iter() {
            if look_up_type_id(type_id).is_subset_of(&look_up_type_id(*map_type_id)) {
                let type_mapping = TypeMapping::from_superset_and_subset(*map_type_id, type_id);
                let mut trait_methods = map_trait_methods
                    .values()
                    .cloned()
                    .into_iter()
                    .collect::<Vec<_>>();
                trait_methods.iter_mut().for_each(|trait_method| {
                    trait_method.copy_types(&type_mapping);
                });
                trait_map.insert(map_trait_name.clone(), type_id, trait_methods);
            }
        }
        trait_map
    }

    pub(crate) fn get_methods_for_type(&self, type_id: TypeId) -> Vec<ty::TyFunctionDeclaration> {
        let mut methods = vec![];
        // small performance gain in bad case
        if look_up_type_id(type_id) == TypeInfo::ErrorRecovery {
            return methods;
        }
        for ((_, map_type_id), map_trait_methods) in self.trait_impls.iter() {
            if look_up_type_id(type_id) == look_up_type_id(*map_type_id) {
                let mut trait_methods = map_trait_methods
                    .values()
                    .cloned()
                    .into_iter()
                    .collect::<Vec<_>>();
                methods.append(&mut trait_methods);
            }
        }
        methods
    }
}

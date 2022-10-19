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

    pub(crate) fn extend(&mut self, other: TraitMap) {
        for (key, other_trait_methods) in other.trait_impls.into_iter() {
            match self.trait_impls.get_mut(&key) {
                Some(trait_methods) => {
                    trait_methods.extend(other_trait_methods.into_iter());
                }
                None => {
                    self.trait_impls.insert(key, other_trait_methods);
                }
            }
        }
    }

    pub(crate) fn filter_by_type(&self, type_id: TypeId) -> TraitMap {
        let mut trait_impls: TraitImpls = im::HashMap::new();
        for ((map_trait_name, map_type_id), map_trait_impls) in self.trait_impls.iter() {
            if look_up_type_id(type_id).is_subset_of(&look_up_type_id(*map_type_id)) {
                trait_impls.insert((map_trait_name.clone(), type_id), map_trait_impls.clone());
            }
        }
        TraitMap { trait_impls }
    }

    pub(crate) fn get_methods_for_type(&self, type_id: TypeId) -> Vec<ty::TyFunctionDeclaration> {
        let mut methods = vec![];
        // small performance gain in bad case
        if look_up_type_id(type_id) == TypeInfo::ErrorRecovery {
            return methods;
        }
        for ((_, map_type_id), map_trait_methods) in self.trait_impls.iter() {
            if look_up_type_id(type_id).is_subset_of(&look_up_type_id(*map_type_id)) {
                let type_mapping = TypeMapping::from_superset_and_subset(*map_type_id, type_id);
                let mut trait_methods = map_trait_methods.values().cloned().collect::<Vec<_>>();
                trait_methods
                    .iter_mut()
                    .for_each(|x| x.copy_types(&type_mapping));
                methods.append(&mut trait_methods);
            }
        }
        methods
    }
}

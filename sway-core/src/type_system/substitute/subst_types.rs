use crate::{engine_threading::*, type_system::priv_prelude::*};

pub trait SubstTypes {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines);

    fn subst(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) {
        if !type_mapping.is_empty() {
            self.subst_inner(type_mapping, engines);
        }
    }
}

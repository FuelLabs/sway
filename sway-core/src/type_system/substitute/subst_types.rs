use crate::{engine_threading::*, type_system::priv_prelude::*};

pub trait SubstTypes {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) -> bool;

    fn subst(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) -> bool {
        if !type_mapping.is_empty() {
            self.subst_inner(type_mapping, engines)
        } else {
            false
        }
    }
}

impl<A, B: SubstTypes> SubstTypes for (A, B) {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) -> bool {
        self.1.subst(type_mapping, engines)
    }
}

impl<T: SubstTypes> SubstTypes for Box<T> {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) -> bool {
        self.as_mut().subst(type_mapping, engines)
    }
}

impl<T: SubstTypes> SubstTypes for Option<T> {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) -> bool {
        self.as_mut()
            .map(|x| x.subst(type_mapping, engines))
            .unwrap_or_default()
    }
}

impl<T: SubstTypes> SubstTypes for Vec<T> {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) -> bool {
        self.iter_mut().fold(false, |has_change, x| {
            x.subst(type_mapping, engines) || has_change
        })
    }
}

#[macro_export]
macro_rules! has_changes {
    ($($stmts:expr);* ;) => {{
        let mut has_change = false;
        $(
            has_change |= $stmts;
        )*
        has_change
    }};
}

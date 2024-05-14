use crate::{engine_threading::Engines, type_system::priv_prelude::*};

#[derive(Default)]
pub enum HasChanges {
    Yes,
    #[default]
    No,
}

impl HasChanges {
    pub fn has_changes(&self) -> bool {
        matches!(self, HasChanges::Yes)
    }
}

impl std::ops::BitOr for HasChanges {
    type Output = HasChanges;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (HasChanges::No, HasChanges::No) => HasChanges::No,
            _ => HasChanges::Yes,
        }
    }
}

pub trait SubstTypes {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) -> HasChanges;

    fn subst(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) -> HasChanges {
        if type_mapping.is_empty() {
            HasChanges::No
        } else {
            self.subst_inner(type_mapping, engines)
        }
    }
}

impl<A, B: SubstTypes> SubstTypes for (A, B) {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) -> HasChanges {
        self.1.subst(type_mapping, engines)
    }
}

impl<T: SubstTypes> SubstTypes for Box<T> {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) -> HasChanges {
        self.as_mut().subst(type_mapping, engines)
    }
}

impl<T: SubstTypes> SubstTypes for Option<T> {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) -> HasChanges {
        self.as_mut()
            .map(|x| x.subst(type_mapping, engines))
            .unwrap_or_default()
    }
}

impl<T: SubstTypes> SubstTypes for Vec<T> {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) -> HasChanges {
        self.iter_mut().fold(HasChanges::No, |has_change, x| {
            x.subst(type_mapping, engines) | has_change
        })
    }
}

#[macro_export]
macro_rules! has_changes {
    ($($stmt:expr);* ;) => {{
        let mut has_changes = $crate::type_system::HasChanges::No;
        $(
            has_changes = $stmt | has_changes;
        )*
        has_changes
    }};
}

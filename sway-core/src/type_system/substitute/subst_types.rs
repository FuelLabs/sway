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

pub struct SubstTypesContext<'a> {
    pub engines: &'a Engines,
    pub subst_function_body: bool,
}

impl<'a> SubstTypesContext<'a> {
    pub fn new(engines: &Engines, subst_function_body: bool) -> SubstTypesContext {
        SubstTypesContext {
            engines,
            subst_function_body,
        }
    }
}

pub trait SubstTypes {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, ctx: &SubstTypesContext) -> HasChanges;

    fn subst(&mut self, type_mapping: &TypeSubstMap, ctx: &SubstTypesContext) -> HasChanges {
        if type_mapping.is_empty() {
            HasChanges::No
        } else {
            self.subst_inner(type_mapping, ctx)
        }
    }
}

impl<A, B: SubstTypes> SubstTypes for (A, B) {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, ctx: &SubstTypesContext) -> HasChanges {
        self.1.subst(type_mapping, ctx)
    }
}

impl<T: SubstTypes> SubstTypes for Box<T> {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, ctx: &SubstTypesContext) -> HasChanges {
        self.as_mut().subst(type_mapping, ctx)
    }
}

impl<T: SubstTypes> SubstTypes for Option<T> {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, ctx: &SubstTypesContext) -> HasChanges {
        self.as_mut()
            .map(|x| x.subst(type_mapping, ctx))
            .unwrap_or_default()
    }
}

impl<T: SubstTypes> SubstTypes for Vec<T> {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, ctx: &SubstTypesContext) -> HasChanges {
        self.iter_mut().fold(HasChanges::No, |has_change, x| {
            x.subst(type_mapping, ctx) | has_change
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

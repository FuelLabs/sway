pub(crate) mod create_copy;
pub(crate) mod subst_list;
pub(crate) mod subst_types;
pub(crate) mod substituted;

// TODO: Change this abstraction to passing closures.
#[derive(Clone, Copy)]
pub enum SubstitutionKind {
    Subst,
    Fold,
    Flatten,
}

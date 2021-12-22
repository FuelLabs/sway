/// The purity of a function is related to its access of contract storage. If a function accesses
/// or could potentially access contract storage, it is [Purity::Impure]. If a function does not utilize any
/// any accesses (reads _or_ writes) of storage, then it is [Purity::Pure].
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum Purity {
    Pure,
    Impure,
}

impl Default for Purity {
    fn default() -> Self {
        Purity::Pure
    }
}

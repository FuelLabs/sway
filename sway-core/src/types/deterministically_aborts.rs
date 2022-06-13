/// If this expression deterministically_aborts 100% of the time, this function returns
/// `true`. Used in dead-code and control-flow analysis.
pub(crate) trait DeterministicallyAborts {
    fn deterministically_aborts(&self) -> bool;
}

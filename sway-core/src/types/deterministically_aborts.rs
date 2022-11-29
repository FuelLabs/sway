/// If this expression deterministically_aborts 100% of the time, this function returns
/// `true`. Used in dead-code and control-flow analysis.
/// if `fn_appl_inlined` is set, the function calls are considered
/// to be inlined, in which case its body is inspected.
pub(crate) trait DeterministicallyAborts {
    fn deterministically_aborts(&self, fn_appl_inlined: bool) -> bool;
}

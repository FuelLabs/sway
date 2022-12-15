/// If this expression deterministically_aborts 100% of the time, this function returns
/// `true`. Used in dead-code and control-flow analysis.
/// if `check_call_body` is set, body of the callee is inspected at call sites.
pub(crate) trait DeterministicallyAborts {
    fn deterministically_aborts(&self, check_call_body: bool) -> bool;
}

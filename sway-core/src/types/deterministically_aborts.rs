/// If this expression deterministically_aborts 100% of the time, this function returns
/// `true`. Used in dead-code and control-flow analysis.
/// `look_inside_callee` determines whether the property is checked inside
/// the body of a called function.
pub trait DeterministicallyAborts {
    fn deterministically_aborts(&self, look_inside_callee: bool) -> bool;
}

use crate::monomorphize::priv_prelude::*;

pub(crate) struct IterationReport {
    pub(super) new_constraints: ConstraintPQ,
    pub(super) instructions: Vec<Instruction>,
}

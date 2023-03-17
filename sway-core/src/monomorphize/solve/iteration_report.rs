use crate::monomorphize::priv_prelude::*;

pub(crate) struct IterationReport<'a> {
    pub(super) new_constraints: ConstraintPQ<'a>,
    pub(super) instructions: Vec<Instruction>,
}

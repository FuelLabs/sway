//! This module takes a list of [Constraint]s, solves them (if they are
//! solvable), and outputs a list of [Instruction]s to apply to a typed AST.

pub(crate) mod instruction_result;
pub(crate) mod iteration_report;
pub(crate) mod solver;

use std::collections::BinaryHeap;

use crate::{engine_threading::*, monomorphize::priv_prelude::*};

pub(crate) struct ConstraintTick {
    constraint: Constraint,
    num_times: usize,
}

impl ConstraintTick {
    pub(super) fn new(constraint: Constraint, num_times: usize) -> ConstraintTick {
        ConstraintTick {
            constraint,
            num_times,
        }
    }
}

impl OrdWithEngines for ConstraintTick {
    fn cmp(&self, other: &Self, engines: Engines<'_>) -> std::cmp::Ordering {
        let ConstraintTick {
            constraint: lc,
            num_times: lnt,
        } = self;
        let ConstraintTick {
            constraint: rc,
            num_times: rnt,
        } = other;
        lc.cmp(rc, engines).then_with(|| lnt.cmp(rnt))
    }
}

impl EqWithEngines for ConstraintTick {}
impl PartialEqWithEngines for ConstraintTick {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let ConstraintTick {
            constraint: lc,
            num_times: lnt,
        } = self;
        let ConstraintTick {
            constraint: rc,
            num_times: rnt,
        } = other;
        lc.eq(rc, engines) && lnt == rnt
    }
}

/// Priority queue sorting the constraints.
// https://dev.to/timclicks/creating-a-priority-queue-with-a-custom-sort-order-using-a-binary-heap-in-rust-3oab
pub(crate) type ConstraintWrapper<'a> = WithEngines<'a, ConstraintTick>;
pub(crate) type ConstraintPQ<'a> = BinaryHeap<ConstraintWrapper<'a>>;

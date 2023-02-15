pub(crate) mod instruction_result;
pub(crate) mod iteration_report;
pub(crate) mod solver;

use std::collections::BinaryHeap;

use crate::{engine_threading::*, monomorphize::priv_prelude::*};

/// Priority queue sorting the constraints.
// https://dev.to/timclicks/creating-a-priority-queue-with-a-custom-sort-order-using-a-binary-heap-in-rust-3oab
pub(crate) type ConstraintWrapper<'a> = WithEngines<'a, Constraint>;
pub(crate) type ConstraintPQ<'a> = BinaryHeap<ConstraintWrapper<'a>>;

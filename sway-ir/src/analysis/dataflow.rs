//! An analysis for solving dataflow equations, currently for live variable analysis.

pub mod live_variables;

use rustc_hash::FxHashMap;
use std::{collections::VecDeque, ops::Sub};

use crate::{AnalysisResultT, Block, Context, Function};

pub struct DataFlowBlock {
    input: BitField,
    output: BitField,
    block: Block,
}

#[derive(Clone, PartialEq, Eq)]
// TODO: actually implement this.
pub struct BitField {}

impl BitField {
    // TODO: actually implement this.
    fn empty() -> BitField {
        BitField {}
    }

    // TODO: actually implement this.
    fn union(&self, other: BitField) -> BitField {
        BitField {}
    }

    // TODO: actually implement this.
    fn substract(&self, other: BitField) -> BitField {
        BitField {}
    }
}

impl Sub for BitField {
    type Output = BitField;

    fn sub(self, rhs: Self) -> Self::Output {
        self.substract(rhs)
    }
}

impl AnalysisResultT for Vec<DataFlowBlock> {}

/// Initial approximation for sets.
pub enum InitialApproximation {
    Over,
    Under,
}

/// Flow of the data for a specific analysis.
pub enum DataFlowDirection {
    Forward,
    Backward,
}

pub enum MeetOperator {
    Union,
    Intersect,
}

trait Gen {
    fn gen(context: &Context, function: &Block) -> BitField;
}

trait Kill {
    fn kill(context: &Context, function: &Block) -> BitField;
}

struct TransferFunction {
    gen: fn(&Context, &Block) -> BitField,
    kill: fn(&Context, &Block) -> BitField,
}

/// A basic worklist based data flow equation solver algorithm.
fn solve_dataflow_equations(
    direction: DataFlowDirection,
    meet_operator: MeetOperator,
    function: Function,
    context: &Context,
    transfer_fn: TransferFunction,
) -> Vec<DataFlowBlock> {
    let block_e = match direction {
        DataFlowDirection::Forward => function.get_entry_block(context),
        DataFlowDirection::Backward => function.get_last_block(context),
    };

    let gen_fn = transfer_fn.gen;
    let kill_fn = transfer_fn.kill;

    let mut inputs = FxHashMap::default();
    let mut outputs = FxHashMap::default();

    match (&direction, &meet_operator) {
        (DataFlowDirection::Forward, MeetOperator::Intersect) => {
            inputs.insert(block_e, BitField::empty());
            let gen_be = gen_fn(context, &block_e);
            outputs.insert(block_e, gen_be);
        }
        (DataFlowDirection::Backward, MeetOperator::Intersect) => {
            outputs.insert(block_e, BitField::empty());
            let gen_be = gen_fn(context, &block_e);
            inputs.insert(block_e, gen_be);
        }
        (DataFlowDirection::Forward, MeetOperator::Union) => {
            for block in function.block_iter(context) {
                let gen = gen_fn(context, &block);
                inputs.insert(block, BitField::empty());
                outputs.insert(block, gen);
            }
        }
        (DataFlowDirection::Backward, MeetOperator::Union) => {
            for block in function.block_iter(context) {
                let gen = gen_fn(context, &block);
                inputs.insert(block, gen);
                outputs.insert(block, BitField::empty());
            }
        }
    }

    let mut worklist = function
        .block_iter(context)
        .filter(|block| *block != block_e)
        .collect::<VecDeque<_>>();
    while let Some(curr_block) = worklist.pop_front() {
        match (&direction, &meet_operator) {
            (DataFlowDirection::Forward, MeetOperator::Union) => todo!(),
            (DataFlowDirection::Forward, MeetOperator::Intersect) => todo!(),
            (DataFlowDirection::Backward, MeetOperator::Union) => {
                let old_in = inputs[&curr_block].clone();
                // Live variable analysis.
                let curr_block_out = curr_block
                    .successors(context)
                    .iter()
                    .map(|succ_block_with_args| succ_block_with_args.block)
                    .map(|succ_block| &inputs[&succ_block])
                    .fold(BitField::empty(), |acc, curr_in| acc.union(curr_in.clone()));

                let curr_block_in = gen_fn(context, &curr_block)
                    .union(curr_block_out - kill_fn(context, &curr_block));

                if curr_block_in != old_in {
                    worklist.extend(curr_block.pred_iter(context))
                }
            }
            (DataFlowDirection::Backward, MeetOperator::Intersect) => todo!(),
        }
    }

    vec![]
}

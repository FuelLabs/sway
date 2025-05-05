mod const_indexed_aggregates;
mod constant_propagate;
mod misc;
mod reachability;
mod verify;

use std::cmp::Ordering;

use super::abstract_instruction_set::AbstractInstructionSet;

use crate::OptLevel;

use super::data_section::DataSection;

/// Maximum number of optimization rounds to perform in release build.
const MAX_OPT_ROUNDS: usize = 10;

impl AbstractInstructionSet {
    pub(crate) fn optimize(
        mut self,
        data_section: &DataSection,
        level: OptLevel,
    ) -> AbstractInstructionSet {
        match level {
            // On debug builds do a single pass through the simple optimizations
            OptLevel::Opt0 => self
                .const_indexing_aggregates_function(data_section)
                .constant_propagate()
                .dce()
                .simplify_cfg()
                .remove_sequential_jumps()
                .remove_redundant_moves()
                .remove_redundant_ops(),
            // On release builds we can do more iterations
            OptLevel::Opt1 => {
                for _ in 0..MAX_OPT_ROUNDS {
                    let old = self.clone();
                    // run two rounds, so that if an optimization depends on another
                    // it will be applied at least once
                    self = self.optimize(data_section, OptLevel::Opt0);
                    self = self.optimize(data_section, OptLevel::Opt0);
                    match self.ops.len().cmp(&old.ops.len()) {
                        // Not able to optimize anything, stop here
                        Ordering::Equal => break,
                        // Never accept worse results
                        Ordering::Greater => return old,
                        // We reduced the number of ops, so continue
                        Ordering::Less => {}
                    }
                }
                self
            }
        }
    }
}

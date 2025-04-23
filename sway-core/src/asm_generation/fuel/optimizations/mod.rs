mod const_indexed_aggregates;
mod misc;
mod reachability;
mod symbolic_interpretation;
mod verify;

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
                .constant_register_propagation()
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
                    if self.ops.len() == old.ops.len() {
                        // Not changed at all, we're done
                        break;
                    } else if old.ops.len() < self.ops.len() {
                        // Never accept worse results
                        return old;
                    }
                }
                self
            }
        }
    }
}

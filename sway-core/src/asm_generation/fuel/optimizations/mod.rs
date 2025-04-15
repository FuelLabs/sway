mod misc;
mod optimizations;
mod verify;

use super::abstract_instruction_set::AbstractInstructionSet;

use crate::OptLevel;

use super::data_section::DataSection;

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
                .dce()
                .simplify_cfg()
                .remove_sequential_jumps()
                .remove_redundant_moves()
                .remove_redundant_ops(),
            // On release builds we can do more iterations
            OptLevel::Opt1 => {
                for _ in 0..10 {
                    // limit the number of iterations
                    let old = self.clone();
                    self = self.optimize(data_section, OptLevel::Opt0);
                    if self.ops.len() == old.ops.len() {
                        // No improvement made, we're done here
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

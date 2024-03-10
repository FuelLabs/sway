use crate::{build_config::ExperimentalFlags, language::parsed::TreeType, BuildTarget};

#[derive(Default)]
pub struct Context {
    pub experimental: ExperimentalFlags,

    /// Indicates whether the module being parsed has a `configurable` block.
    module_has_configurable_block: bool,

    /// Unique suffix used to generate unique names for destructured structs.
    destructured_struct_unique_suffix: usize,

    /// Unique suffix used to generate unique names for destructured tuples.
    destructured_tuple_unique_suffix: usize,

    /// Unique suffix used to generate unique names for variables
    /// that store values matched in match expressions.
    match_expression_matched_value_unique_suffix: usize,

    /// The build target.
    build_target: BuildTarget,

    /// The program type.
    program_type: Option<TreeType>,
}

impl Context {
    /// Create a new context.
    pub fn new(build_target: BuildTarget, experimental: ExperimentalFlags) -> Self {
        Self {
            build_target,
            experimental,
            ..Default::default()
        }
    }

    /// Updates the value of `module_has_configurable_block`.
    pub fn set_module_has_configurable_block(&mut self, val: bool) {
        self.module_has_configurable_block = val;
    }

    /// Returns whether the module being parsed has a `configurable` block.
    pub fn module_has_configurable_block(&self) -> bool {
        self.module_has_configurable_block
    }

    /// Returns a unique suffix used to generate a unique name for a destructured struct.
    pub fn next_destructured_struct_unique_suffix(&mut self) -> usize {
        self.destructured_struct_unique_suffix += 1;
        self.destructured_struct_unique_suffix
    }

    /// Returns a unique suffix used to generate a unique name for a destructured tuple
    pub fn next_destructured_tuple_unique_suffix(&mut self) -> usize {
        self.destructured_tuple_unique_suffix += 1;
        self.destructured_tuple_unique_suffix
    }

    /// Returns a unique suffix used to generate a unique name for a variable
    /// that stores the value matched in a match expression.
    pub fn next_match_expression_matched_value_unique_suffix(&mut self) -> usize {
        self.match_expression_matched_value_unique_suffix += 1;
        self.match_expression_matched_value_unique_suffix
    }

    /// Returns the build target.
    pub fn build_target(&self) -> BuildTarget {
        self.build_target
    }

    /// Returns the program type.
    pub fn program_type(&self) -> Option<TreeType> {
        self.program_type
    }

    /// Updates the value of `program_type`.
    pub fn set_program_type(&mut self, program_type: TreeType) {
        self.program_type = Some(program_type);
    }
}

use crate::BuildTarget;

#[derive(Default)]
pub struct Context {
    /// Indicates whether the module being parsed has a `configurable` block
    module_has_configurable_block: bool,

    /// Unique suffix used to generate unique names for destructured structs
    destructured_struct_unique_suffix: usize,

    /// Unique suffix used to generate unique names for destructured tuples
    destructured_tuple_unique_suffix: usize,

    /// Unique suffix used to generate unique names for vars returned from `match` expressions
    match_expression_return_var_unique_suffix: usize,

    /// The build target for this compilation.
    target: BuildTarget,
}

impl Context {
    /// Update the value of `module_has_configurable_block`
    pub fn set_module_has_configurable_block(&mut self, val: bool) {
        self.module_has_configurable_block = val;
    }

    /// Returns whether the module being parsed has a `configurable` block
    pub fn module_has_configurable_block(&self) -> bool {
        self.module_has_configurable_block
    }

    /// Returns a suffix used to generate a unique name for a destructured struct
    pub fn next_destructured_struct_unique_suffix(&mut self) -> usize {
        self.destructured_struct_unique_suffix += 1;
        self.destructured_struct_unique_suffix
    }

    /// Returns a unique suffix used to generate a unique name for a destructured tuple
    pub fn next_destructured_tuple_unique_suffix(&mut self) -> usize {
        self.destructured_tuple_unique_suffix += 1;
        self.destructured_tuple_unique_suffix
    }

    /// Returns a unique suffix used to generate a unique name for a var returned from a `match`
    /// expressions
    pub fn next_match_expression_return_var_unique_suffix(&mut self) -> usize {
        self.match_expression_return_var_unique_suffix += 1;
        self.match_expression_return_var_unique_suffix
    }

    pub fn target(&self) -> BuildTarget {
        self.target
    }
}

use core::cell::RefCell;

/// A context containing global information used during `convert_parse_tree"
#[derive(Default)]
pub struct Context {
    /// The inner context.
    /// This construction is used to avoid `&mut` all over `convert_parse_tree`.
    inner: RefCell<ContextInner>,
}

/// Contains the actual data for `Context`.
/// Modelled this way to afford an API using interior mutability.
#[derive(Default)]
struct ContextInner {
    /// Indicates whether the module being parsed has a `configurable` block
    module_has_configurable_block: bool,

    /// Unique suffix used to generate unique names for destructured structs
    destructured_struct_unique_suffix: usize,

    /// Unique suffix used to generate unique names for destructured tuples
    destructured_tuple_unique_suffix: usize,

    /// Unique suffix used to generate unique names for vars returned from `match` expressions
    match_expression_return_var_unique_suffix: usize,
}

impl Context {
    /// Update the value of `module_has_configurable_block`
    pub fn set_module_has_configurable_block(&self, val: bool) {
        self.inner.borrow_mut().module_has_configurable_block = val;
    }

    /// Returns whether the module being parsed has a `configurable` block
    pub fn module_has_configurable_block(&self) -> bool {
        self.inner.borrow().module_has_configurable_block
    }

    /// Returns a suffix used to generate a unique name for a destructured struct
    pub fn next_destructured_struct_unique_suffix(&self) -> usize {
        let new_suffix = self.inner.borrow().destructured_struct_unique_suffix + 1;
        self.inner.borrow_mut().destructured_struct_unique_suffix = new_suffix;
        new_suffix
    }

    /// Returns a unique suffix used to generate a unique name for a destructured tuple
    pub fn next_destructured_tuple_unique_suffix(&self) -> usize {
        let new_suffix = self.inner.borrow().destructured_tuple_unique_suffix + 1;
        self.inner.borrow_mut().destructured_tuple_unique_suffix = new_suffix;
        new_suffix
    }

    /// Returns a unique suffix used to generate a unique name for a var returned from a `match`
    /// expressions
    pub fn next_match_expression_return_var_unique_suffix(&self) -> usize {
        let new_suffix = self
            .inner
            .borrow()
            .match_expression_return_var_unique_suffix
            + 1;
        self.inner
            .borrow_mut()
            .match_expression_return_var_unique_suffix = new_suffix;
        new_suffix
    }
}

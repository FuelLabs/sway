mod source_code_module;
use crate::*;
pub use source_code_module::*;

/// A documentation parser and generator.
pub struct Documenter {
    modules: Vec<SourceCodeModule>,
}

impl Documenter {
    /// Given input sway source code, generates [Documention] for it.
    pub fn generate_documentation(input: &str) -> Result<Documentation, DocumentationError> {
        todo!("{}", input)
    }

    /// Create a new [Documenter] from a mapping of module names to their source code.
    pub fn new(_raw: impl Into<Vec<SourceCodeModule>>) -> Self {
        todo!()
    }
}

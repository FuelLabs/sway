/// Represents a Sway module and its contents as a string.
pub struct SourceCodeModule {
    /// The name of a module.
    /// e.g. if module `root` contains a submodule `foo`, this would be
    /// vec!["root", "foo"]
    name: Vec<String>,
    /// The raw source code contained in the module.
    source: String,
}

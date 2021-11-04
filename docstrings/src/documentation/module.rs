use super::DocumentedItem;
/// A sway module is a name, where the strings represent the name of the module
/// and parent modules.
/// e.g. if module `root` contains a submodule `foo`, this would be
/// vec!["root", "foo"]
///
/// and the documented items associated with it
pub struct Module {
    /// The name of a module.
    /// e.g. if module `root` contains a submodule `foo`, this would be
    /// vec!["root", "foo"]
    name: Vec<String>,
    /// The documented items contained in this module.
    documented_items: Vec<DocumentedItem>,
}

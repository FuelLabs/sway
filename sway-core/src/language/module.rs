use sway_types::Ident;

/// The name used within a module to refer to one of its submodules.
///
/// If an alias was given to the `dep`, this will be the alias. If not, this is the submodule's
/// library name.
pub type DepName = Ident;

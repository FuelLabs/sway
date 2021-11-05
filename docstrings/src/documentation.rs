mod documented_item;
mod item_type;
mod module;
pub use documented_item::*;
pub use item_type::*;
pub use module::*;

/// Represents a compiled project's entire documentation.
pub struct Documentation {
    modules: Vec<Module>,
}

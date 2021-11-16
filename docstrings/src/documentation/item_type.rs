/// The type of the item being documented.
pub enum ItemType {
    /// A Sway function, starting with `fn`, that is _not_ a method, abi method, or interface surface item.
    Function,
    /// A Sway method, starting with `fn`, that is _not_ a top-level function or ABI method.
    Method,
    /// A Sway struct, denoted with `struct`.
    Struct,
}

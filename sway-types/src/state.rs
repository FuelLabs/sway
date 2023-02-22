/// Represents the position in a storage statement that a field was declared.
/// For example, in the following storage declaration, `foo` has [StateIndex] 0 and `bar` has
/// [StateIndex] 1.
/// ```
//// storage {
////   foo: u32 = 0,
////   bar: u32 = 0,
//// }
/// ```
/// The actual [StorageSlot] is calculated as the sha256 hash of the domain separator
/// [sway_utils::constants::STORAGE_DOMAIN_SEPARATOR] concatenated with the index.
///
/// Here, `foo`'s [StorageSlot] is `sha256(format!("{}{}", STORAGE_DOMAIN_SEPARATOR, 0))` or
/// `F383B0CE51358BE57DAA3B725FE44ACDB2D880604E367199080B4379C41BB6ED`.
///
/// `bar`'s [StorageSlot] is `sha256(format!("{}{}", STORAGE_DOMAIN_SEPARATOR, 1))` or
/// `DE9090CB50E71C2588C773487D1DA7066D0C719849A7E58DC8B6397A25C567C0`.
#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub struct StateIndex(usize);

impl StateIndex {
    pub fn new(raw: usize) -> Self {
        StateIndex(raw)
    }
    pub fn to_usize(&self) -> usize {
        self.0
    }
}

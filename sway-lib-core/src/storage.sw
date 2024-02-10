library;

/// Describes a location in storage.
///
/// # Additional Information
///
/// The location in storage is specified by the `b256` key of a particular storage slot and an
/// offset, in words, from the start of the storage slot at `key`. 
pub struct StorageKey {
    /// The assigned location in storage.
    pub slot: b256,
    /// The assigned offset
    pub offset: u64,
}

pub trait Storage {
    fn new(id: StorageKey) -> Self;
}

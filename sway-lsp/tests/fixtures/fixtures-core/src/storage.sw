library;

/// Describes a location in storage.
///
/// # Additional Information
/// 
/// The location in storage is specified by the `b256` key of a particular storage slot and an
/// offset, in words, from the start of the storage slot at `key`. The parameter `T` is the type of 
/// the data to be read from or written to at `offset`.
/// `field_id` is a unique identifier for the storage field being referred to, it is different even
/// for multiple zero sized fields that might live at the same location but
/// represent different storage constructs.
pub struct StorageKey<T> {
    /// The assigned location in storage.
    slot: b256,
    /// The assigned offset based on the data structure `T`.
    offset: u64,
    /// A unique identifier.
    field_id: b256,
}

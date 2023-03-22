library;

/// Describes a location in storage specified by the `b256` key of a particular storage slot and an
/// offset, in words, from the start of the storage slot at `key`. The parameter `T` is the type of 
/// the data to be read from or written to at `offset`.
pub struct StorageHandle<T> {
    key: b256,
    offset: u64,
}

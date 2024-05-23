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

impl<T> StorageKey<T> {
    /// Create a new `StorageKey`.
    ///
    /// # Arguments
    ///
    /// * `slot`: [b256] - The assigned location in storage for the new `StorageKey`.
    /// * `offset`: [u64] - The assigned offset based on the data structure `T` for the new `StorageKey`.
    /// * `field_id`: [b256] - A unique identifier for the new `StorageKey`.
    ///
    /// # Returns
    ///
    /// * [StorageKey] - The newly create `StorageKey`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::hash::sha256;
    ///
    /// fn foo() {
    ///     let my_key = StorageKey::<u64>::new(b256::zero(), 0, sha256(b256::zero()));
    ///     assert(my_key.slot() == b256::zero());
    /// }
    /// ```
    pub fn new(slot: b256, offset: u64, field_id: b256) -> Self {
        Self {
            slot,
            offset,
            field_id,
        }
    }

    /// Returns the storage slot address.
    ///
    /// # Returns
    ///
    /// * [b256] - The address in storage that this storage slot points to.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::hash::sha256;
    ///
    /// fn foo() {
    ///     let my_key = StorageKey::<u64>::new(b256::zero(), 0, sha256(b256::zero()));
    ///     assert(my_key.slot() == b256::zero());
    /// }
    /// ```
    pub fn slot(self) -> b256 {
        self.slot
    }

    /// Returns the offset on the storage slot.
    ///
    /// # Returns
    ///
    /// * [u64] - The offset in storage that this storage slot points to.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::hash::sha256;
    ///
    /// fn foo() {
    ///     let my_key = StorageKey::<u64>::new(b256::zero(), 0, sha256(b256::zero()));
    ///     assert(my_key.offset() == 0);
    /// }
    /// ```
    pub fn offset(self) -> u64 {
        self.offset
    }

    /// Returns the storage slot field id.
    ///
    /// # Additional Information
    ///
    /// The field id is a unique identifier for the storage field being referred to, it is different even
    /// for multiple zero sized fields that might live at the same location but
    /// represent different storage constructs.
    ///
    /// # Returns
    ///
    /// * [b256] - The field id for this storage slot.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::hash::sha256;
    ///
    /// fn foo() {
    ///     let my_key = StorageKey::<u64>::new(b256::zero(), 0, sha256(b256::zero()));
    ///     assert(my_key.field_id() == sha256(b256::zero()));
    /// }
    /// ```
    pub fn field_id(self) -> b256 {
        self.field_id
    }
}

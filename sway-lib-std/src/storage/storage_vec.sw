library;

use ::alloc::{alloc_bytes, realloc_bytes};
use ::assert::assert;
use ::hash::*;
use ::option::Option::{self, *};
use ::storage::storage_api::*;
use ::storage::storage_key::*;
use ::vec::Vec;
use ::iterator::Iterator;
use ::codec::*;
use ::debug::*;
use ::marker::*;

/// Describes an out-of-bounds `StorageVec` access.
pub struct OutOfBounds {
    /// The length of the `StorageVec`.
    pub length: u64,
    /// The index that was out-of-bounds.
    /// `index` is always greater than or equal to `length`.
    pub index: u64,
}

/// The error type used by `StorageVec<T>` methods.
#[error_type]
pub enum StorageVecError {
    /// Index was out of bounds.
    #[error(m = "Index was out of bounds.")]
    IndexOutOfBounds: OutOfBounds,
    /// Called `StorageVec<T>` method does not support `T` being a nested storage type (a zero-sized type in general).
    #[error(m = "Called `StorageVec<T>` method does not support `T` being a nested storage type (a zero-sized type in general).")]
    MethodDoesNotSupportNestedStorageTypes: (),
}

/// A storage type for storing a vector of elements of type `V`.
///
/// `StorageVec` can contain storage types and can be contained
///  in storage types. E.g.:
///  - `StorageVec<StorageVec<u64>>`
///  - `StorageVec<StorageMap<u64, b256>>`
///  - `StorageMap<u64, StorageVec<b256>>`
///
/// **Some `StorageVec` methods are not supported for
/// nested storage types and will revert if used with nested
/// storage types.** E.g., `StorageVec<StorageString>::remove`
/// will revert.
///
/// **Some `StorageVec` methods can have limitations and exceptional
/// behaviors when used with nested storage types.
/// E.g., `StorageVec<StorageString>::pop` will pop the last element,
/// but it will always return `None` and the `StorageString` will not
/// be removed from the storage.
///
/// To see if some `StorageVec` method is supported for nested storage types
/// at all, or if it has any limitations or exceptional behavior,
/// refer to method documentation.
#[cfg(experimental_dynamic_storage = false)]
pub struct StorageVec<V> {}

/// A storage type for storing a vector of elements of type `V`.
///
/// `StorageVec` can contain storage types and can be contained
///  in storage types. E.g.:
///  - `StorageVec<StorageVec<u64>>`
///  - `StorageVec<StorageMap<u64, b256>>`
///  - `StorageMap<u64, StorageVec<b256>>`
///
/// **The size of `V` must be less than or equal to 1024 bytes.**
/// Having size of `V` be larger than 1024 bytes is considered an
/// undefined behavior and can lead to unexpected results, most
/// likely to run-time reverts.
///
/// **Some `StorageVec` methods are not supported for
/// nested storage types and will panic if used with nested
/// storage types.** E.g., `StorageVec<StorageString>::remove`
/// will panic with `StorageVecError::MethodDoesNotSupportNestedStorageTypes`.
///
/// **Some `StorageVec` methods can have limitations and exceptional
/// behaviors when used with nested storage types.**
/// E.g., `StorageVec<StorageString>::pop` will pop the last element,
/// but it will always return `None` and the `StorageString` will not
/// be removed from the storage.
///
/// To see if some `StorageVec` method is supported for nested storage types
/// at all, or if it has any limitations or exceptional behaviors,
/// refer to method documentation.
#[cfg(experimental_dynamic_storage = true)]
pub struct StorageVec<V> {}

// Note: `StorageVec` is a zero-sized storage type that can be nested
//       within other storage types. For example, a `StorageMap<K, StorageVec>`.
//       That's why we **always use the `self.field_id`** as a storage slot
//       for all of the methods of `StorageVec`, and **never the `self.slot`**.

// In the quads-based `StorageVec` the `self.field_id`
// stores the vector's length, while the actual content is stored
// at the `sha256(self.field_id)`.
#[cfg(experimental_dynamic_storage = false)]
impl<V> StorageKey<StorageVec<V>> {
    /// Appends `value` to the end of the vector.
    ///
    /// # Arguments
    ///
    /// * `value`: [V] - The item being added to the end of the vector.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `3`
    /// * Writes: `2`
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::storage::storage_vec::*;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    ///     vec_of_vec: StorageVec<StorageVec<u64>> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///     let five = 5_u64;
    ///     storage.vec.push(five);
    ///     assert_eq(five, storage.vec.get(0).unwrap().read());
    ///
    ///     storage.vec_of_vec.push(StorageVec {});
    ///     assert_eq(0, storage.vec_of_vec.get(0).unwrap().len());
    /// }
    /// ```
    #[storage(read, write)]
    pub fn push(self, value: V) {
        let len = read_quads::<u64>(self.field_id(), 0).unwrap_or(0);

        // Storing the value at the current length index
        // (if this is the first item, starts off at 0).
        let key = sha256(self.field_id());
        let offset = offset_calculator::<V>(len);
        write_quads::<V>(key, offset, value);

        // Incrementing the length.
        write_quads(self.field_id(), 0, len + 1);
    }

    /// Removes the last element of the vector and returns it, or `None` if the vector is empty,
    /// or it contains a nested storage type.
    ///
    /// # Additional Information
    ///
    /// **If `V` is a nested storage type, `pop` always returns `None`, even if the vector is not
    /// empty and the last element got removed.**
    ///
    /// > **_WARNING:_** **If `V` is a nested storage type, `pop` will reduce the vector length and
    /// remove the last element _from the vector_, but the nested content of the removed element
    /// will remain in the storage.**
    ///
    /// # Returns
    ///
    /// * [Option<V>] - The last element of the vector, or `None` if the vector is empty, or it contains a nested storage type.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `3`
    /// * Writes: `1`
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::storage::storage_vec::*;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///     let five = 5_u64;
    ///     storage.vec.push(five);
    ///     let popped_value = storage.vec.pop().unwrap();
    ///     assert_eq(five, popped_value);
    ///     let none_value = storage.vec.pop();
    ///     assert(none_value.is_none());
    /// }
    /// ```
    #[storage(read, write)]
    pub fn pop(self) -> Option<V> {
        let len = read_quads::<u64>(self.field_id(), 0).unwrap_or(0);

        if len == 0 {
            return None;
        }

        // Reducing len by 1, effectively removing the last item in the vec.
        write_quads(self.field_id(), 0, len - 1);

        let key = sha256(self.field_id());
        let offset = offset_calculator::<V>(len - 1);
        read_quads::<V>(key, offset)
    }

    /// Gets the value at the given `index` or `None` if `index` is out of bounds.
    ///
    /// # Arguments
    ///
    /// * `index`: [u64] - The index to retrieve the item from.
    ///
    /// # Returns
    ///
    /// * [Option<StorageKey<V>>] - Location in storage of the value stored at `index` or `None` if `index` is out of bounds.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1`
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::storage::storage_vec::*;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     let five = 5_u64;
    ///     storage.vec.push(five);
    ///     assert_eq(five, storage.vec.get(0).unwrap().read());
    ///     assert(storage.vec.get(1).is_none());
    /// }
    /// ```
    #[storage(read)]
    pub fn get(self, index: u64) -> Option<StorageKey<V>> {
        let len = read_quads::<u64>(self.field_id(), 0).unwrap_or(0);

        if len <= index {
            return None;
        }

        let key = sha256(self.field_id());
        let offset = offset_calculator::<V>(index);

        // This `StorageKey` can be read by the standard storage API.
        // Note that it has a unique `field_id` set to `sha256((index, key))`,
        // to support nested storage types that will use that value.
        Some(StorageKey::<V>::new(key, offset, sha256((index, key))))
    }

    /// Removes the element at the given `index` and moves all the elements at the following indices
    /// down one index. Returns the removed element.
    ///
    /// # Additional Information
    ///
    /// **This method is not supported for nested storage types and will revert if `V` is a nested storage type.**
    ///
    /// > **_WARNING:_** Gas consuming for larger vectors.
    ///
    /// # Arguments
    ///
    /// * `index`: [u64] - The index to remove the element from.
    ///
    /// # Returns
    ///
    /// * [V] - The removed element.
    ///
    /// # Reverts
    ///
    /// * If `index` is out of bounds.
    /// * If `V` is a nested storage type, i.e. if it is a zero-sized type.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `3 + (2 * (self.len() - index))`
    /// * Writes: `self.len() - index`
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::storage::storage_vec::*;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///     storage.vec.push(15);
    ///     let removed_value = storage.vec.remove(1);
    ///     assert_eq(10, removed_value);
    ///     assert_eq(storage.vec.len(), 2);
    /// }
    /// ```
    #[storage(read, write)]
    pub fn remove(self, index: u64) -> V {
        assert(__size_of::<V>() > 0);

        let len = read_quads::<u64>(self.field_id(), 0).unwrap_or(0);

        assert(index < len);

        // gets the element before removing it, so it can be returned
        let key = sha256(self.field_id());
        let removed_offset = offset_calculator::<V>(index);
        let removed_element = read_quads::<V>(key, removed_offset).unwrap();

        // for every element in the vec with an index greater than the input index,
        // shifts the index for that element down one
        let mut count = index + 1;
        while count < len {
            // gets the storage location for the previous index and
            // moves the element of the current index into the previous index
            let write_offset = offset_calculator::<V>(count - 1);
            let read_offset = offset_calculator::<V>(count);
            write_quads::<V>(
                key,
                write_offset,
                read_quads::<V>(key, read_offset)
                    .unwrap(),
            );

            count += 1;
        }

        // decrements len by 1
        write_quads(self.field_id(), 0, len - 1);

        removed_element
    }

    /// Replaces the element at the given `index` with the last element, and removes the last element.
    /// Returns the replaced element.
    ///
    /// # Additional Information
    ///
    /// **This method is not supported for nested storage types and will revert if `V` is a nested storage type.**
    ///
    /// If `index` is the index of the last element, the last element will be returned and removed from the vector.
    ///
    /// # Arguments
    ///
    /// * `index`: [u64] - The index at which to replace the element with the last element.
    ///
    /// # Returns
    ///
    /// * [V] - The replaced element.
    ///
    /// # Reverts
    ///
    /// * If `index` is out of bounds.
    /// * If `V` is a nested storage type, i.e. if it is a zero-sized type.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `5`
    /// * Writes: `2`
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::storage::storage_vec::*;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///     storage.vec.push(15);
    ///     let removed_value = storage.vec.swap_remove(0);
    ///     assert_eq(5, removed_value);
    ///     let swapped_value = storage.vec.get(0).unwrap().read();
    ///     assert_eq(15, swapped_value); // The first element is replaced by the last element.
    ///     assert_eq(2, vec.len()); // The last element is removed.
    /// }
    /// ```
    #[storage(read, write)]
    pub fn swap_remove(self, index: u64) -> V {
        assert(__size_of::<V>() > 0);

        let len = read_quads::<u64>(self.field_id(), 0).unwrap_or(0);

        assert(index < len);

        let key = sha256(self.field_id());
        // gets the element before removing it, so it can be returned
        let element_offset = offset_calculator::<V>(index);
        let element_to_be_removed = read_quads::<V>(key, element_offset).unwrap();

        let last_offset = offset_calculator::<V>(len - 1);
        let last_element = read_quads::<V>(key, last_offset).unwrap();

        write_quads::<V>(key, element_offset, last_element);

        // decrements len by 1
        write_quads(self.field_id(), 0, len - 1);

        element_to_be_removed
    }

    /// Sets the `value` at the given `index`.
    ///
    /// # Additional Information
    ///
    /// **This method is not supported for nested storage types and will revert if `V` is a nested storage type.**
    ///
    /// # Arguments
    ///
    /// * `index`: [u64] - The index to set the value at.
    /// * `value`: [V] - The value to be set.
    ///
    /// # Reverts
    ///
    /// * If `index` is out of bounds.
    /// * If `V` is a nested storage type, i.e. if it is a zero-sized type.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1` or `2` (`1` to get vector's length, and `0` if the `value` occupies full slots, or `1` otherwise to read the existing data that will be partially overwritten)
    /// * Writes: `1`
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::storage::storage_vec::*;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///
    ///     storage.vec.set(0, 15);
    ///     let set_value = storage.vec.get(0).unwrap().read();
    ///     assert_eq(15, set_value);
    /// }
    /// ```
    #[storage(read, write)]
    pub fn set(self, index: u64, value: V) {
        assert(__size_of::<V>() > 0);

        let len = read_quads::<u64>(self.field_id(), 0).unwrap_or(0);

        assert(index < len);

        let key = sha256(self.field_id());
        let offset = offset_calculator::<V>(index);
        write_quads::<V>(key, offset, value);
    }

    /// Inserts the `value` at the given `index`, moving the current value at `index`
    /// as well as the following indices up by one index.
    ///
    /// # Additional Information
    ///
    /// **This method is not supported for nested storage types and will revert if `V` is a nested storage type.**
    ///
    /// If `index` is equal to vector's length, appends the `value` at the end of the
    /// vector, effectively acting the same as `push`.
    ///
    /// > **_WARNING:_** Gas consuming for larger vectors.
    ///
    /// # Arguments
    ///
    /// * `index`: [u64] - The index to insert the value into.
    /// * `value`: [V] - The value to insert.
    ///
    /// # Reverts
    ///
    /// * If `index` is larger than the length of the vector.
    /// * If `V` is a nested storage type, i.e. if it is a zero-sized type.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `if self.len() == index { 3 } else { 5 + (2 * (self.len() - index)) }`
    /// * Writes: `if self.len() == index { 2 } else { 2 + self.len() - index }`
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::storage::storage_vec::*;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     storage.vec.push(5);
    ///     storage.vec.push(15);
    ///
    ///     storage.vec.insert(1, 10);
    ///
    ///     assert_eq(5, storage.vec.get(0).unwrap().read());
    ///     assert_eq(10, storage.vec.get(1).unwrap().read());
    ///     assert_eq(15, storage.vec.get(2).unwrap().read());
    /// }
    /// ```
    #[storage(read, write)]
    pub fn insert(self, index: u64, value: V) {
        assert(__size_of::<V>() > 0);

        let len = read_quads::<u64>(self.field_id(), 0).unwrap_or(0);

        assert(index <= len);

        // if len is 0, index must also be 0 due to above check
        let key = sha256(self.field_id());
        if len == index {
            let offset = offset_calculator::<V>(index);
            write_quads::<V>(key, offset, value);

            // increments len by 1
            write_quads(self.field_id(), 0, len + 1);

            return;
        }

        // for every element in the vec with an index larger than the input index,
        // move the element up one index.
        // performed in reverse to prevent data overwriting
        let mut count = len - 1;
        while count >= index {
            // shifts all the values up one index
            let write_offset = offset_calculator::<V>(count + 1);
            let read_offset = offset_calculator::<V>(count);
            write_quads::<V>(
                key,
                write_offset,
                read_quads::<V>(key, read_offset)
                    .unwrap(),
            );

            if count == 0 {
                break;
            }
            count -= 1;
        }

        // inserts the value into the now unused index
        let offset = offset_calculator::<V>(index);
        write_quads::<V>(key, offset, value);

        // increments len by 1
        write_quads(self.field_id(), 0, len + 1);
    }

    /// Returns the length of the vector.
    ///
    /// # Returns
    ///
    /// * [u64] - The length of the vector.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1`
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::storage::storage_vec::*;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///     assert_eq(0, storage.vec.len());
    ///     storage.vec.push(5);
    ///     assert_eq(1, storage.vec.len());
    ///     storage.vec.push(10);
    ///     assert_eq(2, storage.vec.len());
    /// }
    /// ```
    #[storage(read)]
    pub fn len(self) -> u64 {
        read_quads::<u64>(self.field_id(), 0).unwrap_or(0)
    }

    /// Returns true if the vector is empty, otherwise false.
    ///
    /// # Returns
    ///
    /// * [bool] - Indicates whether the vector is empty or not.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1` (to get vector's length)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::storage::storage_vec::*;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     assert(storage.vec.is_empty());
    ///     storage.vec.push(5);
    ///     assert(!storage.vec.is_empty());
    ///     storage.vec.clear();
    ///     assert(storage.vec.is_empty());
    /// }
    /// ```
    #[storage(read)]
    pub fn is_empty(self) -> bool {
        read_quads::<u64>(self.field_id(), 0).unwrap_or(0) == 0
    }

    /// Swaps the positions of the two elements at the given indices.
    ///
    /// # Additional Information
    ///
    /// **This method is not supported for nested storage types and will revert if `V` is a nested storage type.**
    ///
    /// # Arguments
    ///
    /// * `element1_index`: [u64] - The index of the first element.
    /// * `element2_index`: [u64] - The index of the second element.
    ///
    /// # Reverts
    ///
    /// * If `element1_index` or `element2_index` is out of bounds.
    /// * If `V` is a nested storage type, i.e. if it is a zero-sized type.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `5`
    /// * Writes: `2`
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::storage::storage_vec::*;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///     storage.vec.push(15);
    ///
    ///     storage.vec.swap(0, 2);
    ///     assert_eq(15, storage.vec.get(0).unwrap().read());
    ///     assert_eq(10, storage.vec.get(1).unwrap().read());
    ///     assert_eq(5, storage.vec.get(2).unwrap().read());
    /// ```
    #[storage(read, write)]
    pub fn swap(self, element1_index: u64, element2_index: u64) {
        assert(__size_of::<V>() > 0);

        let len = read_quads::<u64>(self.field_id(), 0).unwrap_or(0);
        assert(element1_index < len);
        assert(element2_index < len);

        if element1_index == element2_index {
            return;
        }

        let key = sha256(self.field_id());
        let element1_offset = offset_calculator::<V>(element1_index);
        let element2_offset = offset_calculator::<V>(element2_index);

        let element1_value = read_quads::<V>(key, element1_offset).unwrap();

        write_quads::<V>(
            key,
            element1_offset,
            read_quads::<V>(key, element2_offset)
                .unwrap(),
        );
        write_quads::<V>(key, element2_offset, element1_value);
    }

    /// Returns the first element of the vector, or `None` if it is empty.
    ///
    /// # Returns
    ///
    /// * [Option<StorageKey<V>>] - The location in storage of the value stored at
    /// the start of the vector, or `None` if the vector is empty.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1` (to get vector's length)
    ///
    /// # Examples
    ///
    /// ```sway
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///     assert(storage.vec.first().is_none());
    ///     storage.vec.push(5);
    ///     assert_eq(5, storage.vec.first().unwrap().read());
    /// }
    /// ```
    #[storage(read)]
    pub fn first(self) -> Option<StorageKey<V>> {
        let key = sha256(self.field_id());
        match read_quads::<u64>(self.field_id(), 0).unwrap_or(0) {
            0 => None,
            _ => Some(StorageKey::<V>::new(key, 0, sha256((0, key)))),
        }
    }

    /// Returns the last element of the vector, or `None` if it is empty.
    ///
    /// # Returns
    ///
    /// * [Option<StorageKey<V>>] - The location in storage of the value stored at
    /// the end of the vector, or `None` if the vector is empty.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1` (to get vector's length)
    ///
    /// # Examples
    ///
    /// ```sway
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///     assert(storage.vec.last().is_none());
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///     assert_eq(10, storage.vec.last().unwrap().read());
    /// }
    /// ```
    #[storage(read)]
    pub fn last(self) -> Option<StorageKey<V>> {
        let key = sha256(self.field_id());
        match read_quads::<u64>(self.field_id(), 0).unwrap_or(0) {
            0 => None,
            len => {
                let offset = offset_calculator::<V>(len - 1);
                Some(StorageKey::<V>::new(key, offset, sha256((len - 1, key))))
            },
        }
    }

    /// Reverses the order of elements in the vector, in place.
    ///
    /// # Additional Information
    ///
    /// **This method is not supported for nested storage types and will panic if `V` is a nested storage type.**
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1 + (3 * (self.len() / 2))`
    /// * Writes: `2 * (self.len() / 2)`
    ///
    /// # Examples
    ///
    /// ```sway
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///     storage.vec.push(15);
    ///
    ///     storage.vec.reverse();
    ///
    ///     assert_eq(15, storage.vec.get(0).unwrap().read());
    ///     assert_eq(10, storage.vec.get(1).unwrap().read());
    ///     assert_eq(5, storage.vec.get(2).unwrap().read());
    /// }
    /// ```
    #[storage(read, write)]
    pub fn reverse(self) {
        assert(__size_of::<V>() > 0);

        let len = read_quads::<u64>(self.field_id(), 0).unwrap_or(0);

        if len < 2 {
            return;
        }

        let key = sha256(self.field_id());
        let mid = len / 2;
        let mut i = 0;
        while i < mid {
            let i_offset = offset_calculator::<V>(i);
            let other_offset = offset_calculator::<V>(len - i - 1);

            let element1_value = read_quads::<V>(key, i_offset).unwrap();

            write_quads::<V>(key, i_offset, read_quads::<V>(key, other_offset).unwrap());
            write_quads::<V>(key, other_offset, element1_value);

            i += 1;
        }
    }

    /// Replaces all elements in the vector with `value`.
    ///
    /// # Additional Information
    ///
    /// **This method is not supported for nested storage types and will revert if `V` is a nested storage type.**
    ///
    /// # Arguments
    ///
    /// * `value`: [V] - Value to copy to each element of the vector.
    ///
    /// # Reverts
    ///
    /// * If `V` is a nested storage type, i.e. if it is a zero-sized type.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1 + self.len()`
    /// * Writes: `self.len()`
    ///
    /// # Examples
    ///
    /// ```sway
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///     storage.vec.push(15);
    ///
    ///     storage.vec.fill(20);
    ///
    ///     assert_eq(20, storage.vec.get(0).unwrap().read());
    ///     assert_eq(20, storage.vec.get(1).unwrap().read());
    ///     assert_eq(20, storage.vec.get(2).unwrap().read());
    /// }
    /// ```
    #[storage(read, write)]
    pub fn fill(self, value: V) {
        assert(__size_of::<V>() > 0);

        let len = read_quads::<u64>(self.field_id(), 0).unwrap_or(0);

        let key = sha256(self.field_id());
        let mut i = 0;
        while i < len {
            let offset = offset_calculator::<V>(i);
            write_quads::<V>(key, offset, value);
            i += 1;
        }
    }

    /// Resizes `self` in place so that `len` is equal to `new_len`.
    ///
    /// # Additional Information
    ///
    /// If `new_len` is greater than `len`, `self` is extended by the difference, with each
    /// additional element being set to `value`. If the `new_len` is less than `len`, `self` is
    /// simply truncated.
    ///
    /// > **_WARNING:_** **If `V` is a nested storage type and `new_len` is less than `len`,
    /// `resize` will reduce the vector length and remove the remaining elements _from the vector_,
    /// but the nested content of those removed elements will remain in the storage.**
    ///
    /// # Arguments
    ///
    /// * `new_len`: [u64] - The new length to expand or truncate to.
    /// * `value`: [V] - The value to set new elements to, if the `new_len` is greater than the current length.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads - `if new_len > self.len() { new_len - len + 2 } else { 2 }`
    /// * Writes - `if new_len > self.len() { new_len - len + 1 } else { 1 }`
    ///
    /// # Examples
    ///
    /// ```sway
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///
    ///     storage.vec.resize(4, 20);
    ///
    ///     assert_eq(5, storage.vec.get(0).unwrap().read());
    ///     assert_eq(10, storage.vec.get(1).unwrap().read());
    ///     assert_eq(20, storage.vec.get(2).unwrap().read());
    ///     assert_eq(20, storage.vec.get(3).unwrap().read());
    ///
    ///     storage.vec.resize(2, 0);
    ///
    ///     assert_eq(5, storage.vec.get(0).unwrap().read());
    ///     assert_eq(10, storage.vec.get(1).unwrap().read());
    ///     assert_eq(None, storage.vec.get(2));
    ///     assert_eq(None, storage.vec.get(3));
    /// }
    /// ```
    #[storage(read, write)]
    pub fn resize(self, new_len: u64, value: V) {
        let mut len = read_quads::<u64>(self.field_id(), 0).unwrap_or(0);
        let key = sha256(self.field_id());
        while len < new_len {
            let offset = offset_calculator::<V>(len);
            write_quads::<V>(key, offset, value);
            len += 1;
        }
        write_quads::<u64>(self.field_id(), 0, new_len);
    }

    /// Stores given `vec` as a `StorageVec`.
    ///
    /// # Additional Information
    ///
    /// This will overwrite any existing values in the `StorageVec`.
    ///
    /// **This method is not supported for nested storage types and will revert if `V` is a nested storage type.**
    ///
    /// # Arguments
    ///
    /// * `vec`: [Vec<V>] - The vector to store in storage.
    ///
    /// # Reverts
    ///
    /// * If `V` is a nested storage type, i.e. if it is a zero-sized type.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Writes: `2`
    ///
    /// # Examples
    ///
    /// ```sway
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///     let mut vec = Vec::<u64>::new();
    ///     vec.push(5);
    ///     vec.push(10);
    ///     vec.push(15);
    ///
    ///     storage.vec.store_vec(vec);
    ///
    ///     assert_eq(5, storage.vec.get(0).unwrap().read());
    ///     assert_eq(10, storage.vec.get(1).unwrap().read());
    ///     assert_eq(15, storage.vec.get(2).unwrap().read());
    /// }
    /// ```
    #[storage(write)]
    pub fn store_vec(self, vec: Vec<V>) {
        assert(__size_of::<V>() > 0);

        let size_V_bytes = __size_of::<V>();

        // Handle cases where elements are not aligned to word boundaries and pad them.
        let slice = if size_V_bytes % 8 != 0 {
            let vec_slice = vec.as_raw_slice();
            let size_V_bytes_padded = ((size_V_bytes + 7) / 8) * 8;
            let number_of_bytes = size_V_bytes_padded * vec.len();
            let ptr = alloc_bytes(number_of_bytes);
            let mut i = 0;
            while i < vec.len() {
                vec_slice
                    .ptr()
                    .add::<V>(i)
                    .copy_bytes_to(ptr.add_uint_offset(size_V_bytes_padded * i), size_V_bytes);
                i += 1;
            }

            raw_slice::from_parts::<u8>(ptr, number_of_bytes)
        } else {
            vec.as_raw_slice()
        };

        // Get the number of storage slots needed based on the size of bytes.
        let number_of_bytes = slice.number_of_bytes();
        let number_of_slots = (number_of_bytes + 31) >> 5;
        let mut ptr = slice.ptr();

        // The capacity needs to be a multiple of 32 bytes so we can
        // make the 'quad' storage instruction store without accessing unallocated heap memory.
        ptr = realloc_bytes(ptr, number_of_bytes, number_of_slots * 32);

        // Store `number_of_slots * 32` bytes starting at storage slot `key`.
        let _ = __state_store_quad(sha256(self.field_id()), ptr, number_of_slots);

        // Store the length, NOT the bytes.
        // This differs from the existing `write_slice()` function to be compatible with `StorageVec`.
        write_quads::<u64>(self.field_id(), 0, vec.len());
    }

    /// Returns all elements contained in `self` as a `Vec<V>`.
    ///
    /// # Additional Information
    ///
    /// **This method is not supported for nested storage types and will revert if `V` is a nested storage type.**
    ///
    /// # Returns
    ///
    /// * [Vec<V>] - The vector containing all elements of `self`.
    ///
    /// # Reverts
    ///
    /// * If `V` is a nested storage type, i.e. if it is a zero-sized type.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `2`
    ///
    /// # Examples
    ///
    /// ```sway
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///     let mut vec = Vec::<u64>::new();
    ///     vec.push(5);
    ///     vec.push(10);
    ///     vec.push(15);
    ///
    ///     storage.vec.store_vec(vec);
    ///
    ///     let loaded_vec = storage.vec.load_vec();
    ///
    ///     assert_eq(5, loaded_vec.get(0).unwrap());
    ///     assert_eq(10, loaded_vec.get(1).unwrap());
    ///     assert_eq(15, loaded_vec.get(2).unwrap());
    /// }
    /// ```
    #[storage(read)]
    pub fn load_vec(self) -> Vec<V> {
        assert(__size_of::<V>() > 0);

        // Get the length of the slice that is stored.
        match read_quads::<u64>(self.field_id(), 0).unwrap_or(0) {
            0 => Vec::new(),
            len => {
                // Get the number of storage slots needed based on the size.
                let size_V_bytes = __size_of::<V>();

                let bytes = if size_V_bytes % 8 != 0 {
                    let size_V_bytes_padded = ((size_V_bytes + 7) / 8) * 8;
                    len * size_V_bytes_padded
                } else {
                    len * size_V_bytes
                };

                let number_of_slots = (bytes + 31) >> 5;
                let ptr = alloc_bytes(number_of_slots * 32);
                let _ = __state_load_quad(sha256(self.field_id()), ptr, number_of_slots);

                if size_V_bytes % 8 != 0 {
                    let len_bytes = len * size_V_bytes;
                    let size_V_bytes_padded = ((size_V_bytes + 7) / 8) * 8;
                    let new_vec = alloc_bytes(len_bytes);
                    let mut i = 0;
                    while i < len {
                        ptr
                            .add_uint_offset((size_V_bytes_padded * i))
                            .copy_bytes_to(new_vec.add::<V>(i), size_V_bytes);
                        i += 1;
                    }

                    Vec::from(
                        asm(ptr: (new_vec, len_bytes)) {
                            ptr: raw_slice
                        },
                    )
                } else {
                    Vec::from(
                        asm(ptr: (ptr, bytes)) {
                            ptr: raw_slice
                        },
                    )
                }
            }
        }
    }

    /// Returns an [Iterator] to iterate over this `StorageVec`.
    ///
    /// # Returns
    ///
    /// * [StorageVecIter<V>] - The iterator over this `StorageVec`.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1`    (to get the vector's length)
    ///
    /// # Examples
    ///
    /// ```sway
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///     storage.vec.push(15);
    ///
    ///     let iter = storage.vec.iter();
    ///
    ///     assert_eq(5, iter.next().unwrap().read());
    ///     assert_eq(10, iter.next().unwrap().read());
    ///     assert_eq(15, iter.next().unwrap().read());
    ///
    ///     let iter = storage.vec.iter();
    ///
    ///     for elem in storage.vec.iter() {
    ///         let elem_value = elem.read();
    ///         log(elem_value);
    ///     }
    /// }
    /// ```
    #[storage(read)]
    pub fn iter(self) -> StorageVecIter<V> {
        StorageVecIter {
            values: self,
            len: read_quads::<u64>(self.field_id(), 0).unwrap_or(0),
            index: 0,
        }
    }
}

pub struct StorageVecIter<V> {
    values: StorageKey<StorageVec<V>>,
    len: u64,
    index: u64,
}

#[cfg(experimental_dynamic_storage = false)]
impl<V> Iterator for StorageVecIter<V> {
    type Item = StorageKey<V>;
    fn next(ref mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            return None;
        }

        let key = sha256(self.values.field_id());
        let offset = offset_calculator::<V>(self.index);
        let result = Some(StorageKey::<V>::new(key, offset, sha256((self.index, key))));

        self.index += 1;

        result
    }
}

/// Calculates the offset of the element at `index`
/// starting from the beginning of the slot at which
/// the `StorageVec` content starts.
///
/// Adds padding to the type size, so it can correctly
/// use the storage api.
#[cfg(experimental_dynamic_storage = false)]
fn offset_calculator<T>(index: u64) -> u64 {
    let size_in_bytes = __size_of::<T>();
    let size_in_bytes = (size_in_bytes + (8 - 1)) - ((size_in_bytes + (8 - 1)) % 8);
    (index * size_in_bytes) / 8
}

// A chunked-dynamic-slot-based `StorageVec` has two distinctive modes
// of operations and storage access:
//
// 1. Storing non-zero-sized types
//    E.g., `StorageVec<u64>`. In this mode, elements are spread across one or
//    more dynamic storage slots, each holding at most `CHUNK_MAX_SIZE` bytes of
//    element data. Every element is **fully contained within a single slot** —
//    no element ever straddles a slot boundary.
//
//    All slots hold the same maximum number of elements:
//      `elems_per_slot = CHUNK_MAX_SIZE / size_of::<V>()` (floor division)
//
//    The first slot is at `self.field_id()` and has the following layout:
//      `[u64 len (8 bytes) | V[0] | V[1] | ... | V[elems_per_slot - 1] ]`
//    Its total size is `8 + CHUNK_MAX_SIZE` bytes: 8 bytes for the length
//    header followed by up to `CHUNK_MAX_SIZE` bytes of element data.
//
//    Subsequent slots are at `self.field_id() + 1`, `self.field_id() + 2`, etc.
//    (computed via `add_u64_to_b256`), and each holds the same `elems_per_slot`
//    elements starting at byte offset 0.
//
//    Given the element at `index`:
//      `chunk_number      = index / elems_per_slot`
//      `index_in_chunk    = index % elems_per_slot`
//      slot key           = `self.field_id() + chunk_number` (via `add_u64_to_b256`)
//      byte offset in slot = `(8 if chunk_number == 0 else 0) + index_in_chunk * size_of::<V>()`
//
//    Methods that shrink the vector (`pop`, `resize`) **do not zero out the residual
//    element bytes** beyond the new length. Those bytes remain in storage but are
//    logically non-existent and will be overwritten by future `push`/`resize` calls.
//
//    All `StorageVec` methods are fully supported.
//
// 2. Storing nested storage types (zero-sized types)
//    E.g., `StorageVec<StorageVec<u64>>`. In this mode, the `self.field_id`
//    stores the vector's length, while the nested storage type at `index`
//    is stored at the slot calculated by `sha256((index, self.field_id))`.
//    In this mode, some `StorageVec` methods are not supported at all,
//    or have limitations or exceptional behaviors.
//
// Because of these two distinctive modes and the fact that implementation of
// one of them gets optimized away (which we do want!), some of the methods have
// different storage access patterns, in terms of reading and writing,
// depending on `V` being a storage type or not.
//
// This is because the purity warnings are generated at the end of the compilation
// pipeline.
//
// For methods that are not supported at all for nested types, like, e.g. `set`,
// the dead code in the `STORES_STORAGE_TYPE = true` branch is eliminated by the
// compiler. To prevent purity warnings when such methods are instantiated for
// nested storage types, we add a dummy `let _ = __state_preload(b256::zero())` call to
// simulate read access, and a dummy `let _ = __state_clear(b256::zero(), 0)` call
// to simulate write access. This is a workaround, but currently we don't have
// better options. The real issue here is the problematic duality of the original
// `StorageVec` API, which currently cannot be expressed via type system and trait
// constraints.

/// Maximum number of bytes of element data stored per storage slot by the chunked `StorageVec`.
/// Every slot — including slot 0 — holds at most `CHUNK_MAX_SIZE` bytes of element data, so
/// all slots have the same element capacity of `CHUNK_MAX_SIZE / size_of::<V>()` elements.
///
/// Slot 0 additionally stores an 8-byte `u64` length header before the element area, making
/// its total size `8 + CHUNK_MAX_SIZE` bytes.
#[cfg(experimental_dynamic_storage = true)]
const CHUNK_MAX_SIZE: u64 = 1024;

#[cfg(experimental_dynamic_storage = true)]
impl<V> StorageKey<StorageVec<V>> {
    // TODO: We should have only one `STORES_STORAGE_TYPE` constant
    //       declared here as an associated constant. This is currently not possible
    //       because of these two limitations:
    //       1. Having the below `const` uncommented causes compiler panic:
    //          https://github.com/FuelLabs/sway/issues/7615
    //       2. All `StorageKey<StorageVec<V>>` types will actually share the same
    //          constant value because they have the same path, and are identified
    //          in IR only by path. **This is a known limitation that can lead to
    //          invalid compilation if 1. is solved before.**
    //       **We can use the below constant only when both above issues are solved.**
    //       Until then, we will have a local `STORES_STORAGE_TYPE` constant in
    //       every method that needs it.

    // /// True if the `StorageVec` stores a storage type, i.e., if `V` is a storage type.
    // const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

    /// Returns the storage slot key and byte offset within that slot for the element at `index`.
    ///
    /// Elements are packed into consecutive storage slots. Every slot holds the same
    /// maximum number of elements — `CHUNK_MAX_SIZE / size_of::<V>()` — and each
    /// element is **fully contained within its slot** (no cross-boundary elements).
    ///
    /// - Slot 0 (`self.field_id()`):
    ///     `[u64 len (8 bytes) | V[0] .. V[elems_per_slot - 1]]`
    ///     Max slot size: `8 + CHUNK_MAX_SIZE` bytes.
    /// - Slot N (`self.field_id() + N`, N ≥ 1):
    ///     `[V[N*elems_per_slot] .. V[(N+1)*elems_per_slot - 1]]`
    ///     Max slot size: `CHUNK_MAX_SIZE` bytes.
    ///
    /// This method does not perform storage access; it only computes storage slot keys and offsets.
    fn get_slot_and_offset_of_elem(self, index: u64) -> (b256, u64) {
        const SIZE_OF_V: u64 = __size_of::<V>();

        // Number of elements that fit in each slot's element area.
        // All slots share the same capacity because slot 0's element area is also
        // CHUNK_MAX_SIZE bytes (its extra 8-byte header is in addition to that).
        const ELEMS_PER_SLOT: u64 = CHUNK_MAX_SIZE / SIZE_OF_V;

        let chunk_number = index / ELEMS_PER_SLOT;
        let offset_in_slot = (index % ELEMS_PER_SLOT) * SIZE_OF_V;

        // Slot 0's element area begins at byte 8 (after the length header).
        // All other slots' element areas begin at byte 0.
        let mut slot = self.field_id();
        let offset_in_slot = if chunk_number > 0 {
            add_u64_to_b256(slot, chunk_number);
            offset_in_slot
        } else {
            offset_in_slot + 8
        };
        (slot, offset_in_slot)
    }

    /// Appends `value` to the end of the vector.
    ///
    /// # Arguments
    ///
    /// * `value`: [V] - The item being added to the end of the vector.
    ///
    /// # Number of Storage Accesses
    ///
    /// ## When `V` is a Non-storage Type
    ///
    /// * Reads: `1`    (vector's length)
    /// * Writes: `2`   (vector's length and new element)
    ///
    /// ## When `V` is a Nested Storage Type
    ///
    /// * Reads: `1`    (vector's length)
    /// * Writes: `1`   (new, increased, vector's length)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::storage::storage_vec::*;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    ///     vec_of_vec: StorageVec<StorageVec<u64>> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///     let five = 5_u64;
    ///     storage.vec.push(five);
    ///     assert_eq(five, storage.vec.get(0).unwrap().read());
    ///
    ///     storage.vec_of_vec.push(StorageVec {});
    ///     assert_eq(0, storage.vec_of_vec.get(0).unwrap().len());
    /// }
    /// ```
    #[storage(read, write)]
    pub fn push(self, value: V) {
        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        let len = self.len();
        let new_len = len + 1;

        if STORES_STORAGE_TYPE {
            // Nothing to actually store. We just need to increment
            // the vector's length. It is `get` that will return the
            // proper `StorageKey` for the nested storage type `V`.
            __state_store_slot(self.field_id(), __addr_of(new_len), 8);
        } else {
            const SIZE_OF_V: u64 = __size_of::<V>();

            // Update the length header first.
            //
            // We must update the length header (the first 8 bytes of slot 0) **before**
            // writing the element. If the slot was previously unused, `__state_update_slot`
            // requires the offset to be within the currently used size. Writing the length
            // first establishes those 8 bytes so subsequent updates to the same slot at
            // offset ≥ 8 are valid.
            __state_update_slot(self.field_id(), __addr_of(new_len), 0, 8); // Compute which slot and offset to write the new element.
            let (elem_slot, elem_offset) = self.get_slot_and_offset_of_elem(len);
            __state_update_slot(elem_slot, __addr_of(value), elem_offset, SIZE_OF_V);
        }
    }

    /// Removes the last element of the vector and returns it, or `None` if the vector is empty,
    /// or it contains a nested storage type.
    ///
    /// # Additional Information
    ///
    /// **If `V` is a nested storage type, `pop` always returns `None`, even if the vector is not
    /// empty and the last element got removed.**
    ///
    /// > **_WARNING:_** **If `V` is a nested storage type, `pop` will reduce the vector length and
    /// remove the last element _from the vector_, but the content of the nested storage type
    /// will remain in the storage.**
    ///
    /// The residual bytes of the removed element remain in storage but are logically non-existent
    /// and will be overwritten by future `push` calls.
    ///
    /// # Returns
    ///
    /// * [Option<V>] - The last element of the vector, or `None` if the vector is empty, or it contains a nested storage type.
    ///
    /// # Number of Storage Accesses
    ///
    /// ## When `V` is a Non-storage Type
    ///
    /// * Reads: `2`    (vector's length and the last element)
    /// * Writes: `1`   (vector's length)
    ///
    /// ## When `V` is a Nested Storage Type
    ///
    /// * Reads: `1`    (vector's length)
    /// * Writes: `1`   (new, decreased, vector's length)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::storage::storage_vec::*;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///     let five = 5_u64;
    ///     storage.vec.push(five);
    ///     let popped_value = storage.vec.pop().unwrap();
    ///     assert_eq(five, popped_value);
    ///     let none_value = storage.vec.pop();
    ///     assert(none_value.is_none());
    /// }
    /// ```
    #[storage(read, write)]
    pub fn pop(self) -> Option<V> {
        let len = self.len();

        if len == 0 {
            return None;
        }

        let new_len = len - 1;

        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        if STORES_STORAGE_TYPE {
            // Reducing len by 1, effectively removing the last item.
            // Note that the value stored in the nested storage type
            // **does not get removed from storage**.
            __state_update_slot(self.field_id(), __addr_of(new_len), 0, 8);
            None
        } else {
            const SIZE_OF_V: u64 = __size_of::<V>();

            // 1. Read the last element from its chunk slot before decrementing the length.
            let (elem_slot, elem_offset) = self.get_slot_and_offset_of_elem(len - 1);
            let last_element = read_slot::<V>(elem_slot, elem_offset);

            // 2. Decrement the length header in slot 0. The last element's bytes remain in the
            //    slot as residual but are logically removed; they will be overwritten on the next push.
            __state_update_slot(self.field_id(), __addr_of(new_len), 0, 8);

            last_element
        }
    }

    /// Gets the value at the given `index` or `None` if `index` is out of bounds.
    ///
    /// # Arguments
    ///
    /// * `index`: [u64] - The index to retrieve the item from.
    ///
    /// # Returns
    ///
    /// * [Option<StorageKey<V>>] - Location in storage of the value stored at `index` or `None` if `index` is out of bounds.
    ///
    /// # Number of Storage Accesses
    ///
    /// ## When `V` is a Non-storage Type
    ///
    /// * Reads: `1`    (vector's length)
    ///
    /// ## When `V` is a Nested Storage Type
    ///
    /// * Reads: `1`    (vector's length)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::storage::storage_vec::*;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     let five = 5_u64;
    ///     storage.vec.push(five);
    ///     assert_eq(five, storage.vec.get(0).unwrap().read());
    ///     assert(storage.vec.get(1).is_none());
    /// }
    /// ```
    #[storage(read)]
    pub fn get(self, index: u64) -> Option<StorageKey<V>> {
        let len = self.len();

        if len <= index {
            return None;
        }

        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        let (slot, offset, field_id) = if STORES_STORAGE_TYPE {
            // For nested storage types, we want to use a unique `field_id`
            // set to `sha256((index, self.field_id())` to ensure every
            // nested storage type element will use it and store its content
            // in a different slot.
            let field_id = self.field_id();
            (field_id, 0, sha256((index, field_id)))
        } else {
            // For non-zero-sized types, their values are stored across chunk slots.
            // `get_slot_and_offset_of_elem` computes the exact slot and byte offset.
            let (elem_slot, elem_offset) = self.get_slot_and_offset_of_elem(index);
            (elem_slot, elem_offset, elem_slot)
        };

        Some(StorageKey::<V>::new(slot, offset, field_id))
    }

    /// Removes the element at the given `index` and moves all the elements at the following indices
    /// down one index. Returns the removed element.
    ///
    /// # Additional Information
    ///
    /// **This method is not supported for nested storage types and will panic if `V` is a nested storage type.**
    ///
    /// The residual bytes of the original last element remain in its chunk slot but are logically
    /// removed and will be overwritten by future `push` calls.
    ///
    /// # Arguments
    ///
    /// * `index`: [u64] - The index to remove the element from.
    ///
    /// # Returns
    ///
    /// * [V] - The removed element.
    ///
    /// # Panics
    ///
    /// * `StorageVecError::IndexOutOfBounds`, if `index` is out of bounds.
    /// * `StorageVecError::MethodDoesNotSupportNestedStorageTypes`, if `V` is a nested storage type, i.e. if it is a zero-sized type.
    ///
    /// # Number of Storage Accesses
    ///
    /// Shifting operates slot-by-slot using a single storage read per affected slot,
    /// so accesses scale with the number of chunk slots containing elements to shift,
    /// not with `len - index`.
    ///
    /// Let `k = floor((len - 1) / elems_per_slot) - floor(index / elems_per_slot)` be
    /// the number of chunk slots that follow the removed element's slot, where
    /// `elems_per_slot = CHUNK_MAX_SIZE / size_of::<V>()`.
    ///
    /// * Reads: `2`          (vector's length and removed element)
    /// * Reads: at most `1`  (tail of the removed element's slot for the intra-slot shift;
    ///                        skipped when the removed element is the last in its slot)
    /// * Reads: `k`          (one read per subsequent slot, loading all its elements at once)
    /// * Writes: at most `1` (shifted tail written back; paired with the read above)
    /// * Writes: `k`         (first element of each subsequent slot promoted to the freed
    ///                        tail position of the previous slot)
    /// * Writes: at most `k` (remaining elements of each subsequent slot shifted left;
    ///                        skipped when the slot has only one element)
    /// * Writes: `1`         (vector's length)
    ///
    /// Total: at most `3k + 4` storage operations,
    /// where `k ≤ ceil((len - index) / elems_per_slot) - 1`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::storage::storage_vec::*;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///     storage.vec.push(15);
    ///     let removed_value = storage.vec.remove(1);
    ///     assert(10 == removed_value);
    ///     assert(storage.vec.len() == 2);
    /// }
    /// ```
    #[storage(read, write)]
    pub fn remove(self, index: u64) -> V {
        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        if STORES_STORAGE_TYPE {
            // This workaround for satisfying the `#[storage]` attribute
            // is explained in the comment above this `StorageVec` impl.
            let _ = __state_preload(b256::zero());
            let _ = __state_clear(b256::zero(), 0);
            panic StorageVecError::MethodDoesNotSupportNestedStorageTypes;
        } else {
            let len = self.len();

            if index >= len {
                panic StorageVecError::IndexOutOfBounds(OutOfBounds {
                    length: len,
                    index,
                });
            }

            // TODO: Move `SIZE_OF_V` to the top of `else`, like in all other methods,
            //       once https://github.com/FuelLabs/sway/issues/7650 is fixed.
            const SIZE_OF_V: u64 = __size_of::<V>();
            const ELEMS_PER_SLOT: u64 = CHUNK_MAX_SIZE / SIZE_OF_V;

            let removed_chunk = index / ELEMS_PER_SLOT;
            let last_chunk = (len - 1) / ELEMS_PER_SLOT;

            // Slot 0's element area starts at byte 8 (after the length header);
            // all other slots' element areas start at byte 0.
            let removed_slot_elem_start: u64 = if removed_chunk == 0 { 8 } else { 0 };

            let (removed_slot, removed_offset) = self.get_slot_and_offset_of_elem(index);

            // Allocate one reusable buffer large enough for `ELEMS_PER_SLOT` elements.
            // This buffer will be used for all storage reads:
            //  - intra-slot left shifts of remaining elements
            //  - cross-slot element transfers
            //  - reading the removed element
            //
            // Note that, if we are removing the last element, there will be no shifts and no cross-slot transfers,
            // so we would only need a buffer of one element size.
            // Similarly, if we are removing an element from the last chunk, we only need to shift the remaining
            // elements in that chunk, so we only need a buffer for those remaining elements.
            //
            // But considering the cost of `aloc` compared to the cost of extra `if` checks and
            // additional arithmetic, it is gas and bytecode size cheaper to just always allocate the buffer
            // for the worst case of shifting a full slot.
            let buf = alloc_bytes(ELEMS_PER_SLOT * SIZE_OF_V);

            // 1. Read the removed element.
            let _ = __state_load_slot(removed_slot, buf, removed_offset, SIZE_OF_V);
            let removed_element = buf.read::<V>();

            // 2. Shift the elements that follow the removed element within its slot one
            //    position to the left.
            let end_of_removed_slot_data: u64 = removed_slot_elem_start + if removed_chunk < last_chunk {
                // Full slot.
                ELEMS_PER_SLOT * SIZE_OF_V
            } else {
                // Partial last slot: only the elements that actually exist.
                (len - removed_chunk * ELEMS_PER_SLOT) * SIZE_OF_V
            };
            let bytes_after_removed = end_of_removed_slot_data - (removed_offset + SIZE_OF_V);
            if bytes_after_removed > 0 {
                let _ = __state_load_slot(
                    removed_slot,
                    buf,
                    removed_offset + SIZE_OF_V,
                    bytes_after_removed,
                );
                __state_update_slot(removed_slot, buf, removed_offset, bytes_after_removed);
            }

            // 3. For each slot that comes after the removed slot:
            //      a. Move its first element into the freed tail position of the previous slot.
            //      b. Shift its remaining elements one position to the left within the slot.
            //
            // The byte offset of the "last element position" in the previous slot:
            // for the first iteration the previous slot is `removed_slot`, so its element
            // area starts at `removed_slot_elem_start`; for all later iterations the previous
            // slot has chunk >= 1, so its element area starts at 0.
            let mut prev_slot = removed_slot;
            let mut prev_slot_last_elem_offset = removed_slot_elem_start + (ELEMS_PER_SLOT - 1) * SIZE_OF_V;
            let mut chunk = removed_chunk + 1;
            let field_id = self.field_id();
            while chunk <= last_chunk {
                let mut cur_slot = field_id;
                add_u64_to_b256(cur_slot, chunk);

                let elems_in_cur_slot: u64 = if chunk < last_chunk {
                    ELEMS_PER_SLOT
                } else {
                    len - chunk * ELEMS_PER_SLOT
                };

                // a. Load all the elements contained in the current slot.
                let _ = __state_load_slot(cur_slot, buf, 0, elems_in_cur_slot * SIZE_OF_V);

                // b. Move the first element of cur_slot into the freed tail of prev_slot.
                __state_update_slot(prev_slot, buf, prev_slot_last_elem_offset, SIZE_OF_V);

                // c. Shift the remaining elements of cur_slot one position to the left.
                let remaining_bytes = (elems_in_cur_slot - 1) * SIZE_OF_V;
                if remaining_bytes > 0 {
                    __state_update_slot(cur_slot, buf.add::<u8>(SIZE_OF_V), 0, remaining_bytes);
                }

                prev_slot = cur_slot;
                // All slots with chunk >= 1 have their element area starting at byte 0,
                // so the last-element offset for every subsequent previous slot is the same.
                prev_slot_last_elem_offset = (ELEMS_PER_SLOT - 1) * SIZE_OF_V;
                chunk += 1;
            }

            // 4. Update the length header.
            let new_len = len - 1;
            __state_update_slot(field_id, __addr_of(new_len), 0, 8);

            removed_element
        }
    }

    /// Replaces the element at the given `index` with the last element, and removes the last element.
    /// Returns the replaced element.
    ///
    /// # Additional Information
    ///
    /// **This method is not supported for nested storage types and will panic if `V` is a nested storage type.**
    ///
    /// If `index` is the index of the last element, the last element will be returned and removed from the vector.
    ///
    /// The residual bytes of the original last element remain in its chunk slot but are logically
    /// removed and will be overwritten by future `push` calls.
    ///
    /// # Arguments
    ///
    /// * `index`: [u64] - The index at which to replace the element with the last element.
    ///
    /// # Returns
    ///
    /// * [V] - The replaced element.
    ///
    /// # Panics
    ///
    /// * `StorageVecError::IndexOutOfBounds`, if `index` is out of bounds.
    /// * `StorageVecError::MethodDoesNotSupportNestedStorageTypes`, if `V` is a nested storage type, i.e. if it is a zero-sized type.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `3`    (vector's length, element at `index`, and last element; last element read skipped if `index` is already the last)
    /// * Writes: `2`   (element at `index` overwritten with the last element, and vector's length; element write skipped if `index` is the last)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::storage::storage_vec::*;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///     storage.vec.push(15);
    ///     let removed_value = storage.vec.swap_remove(0);
    ///     assert_eq(5, removed_value);
    ///     let swapped_value = storage.vec.get(0).unwrap().read();
    ///     assert_eq(15, swapped_value); // The first element is replaced by the last element.
    ///     assert_eq(2, storage.vec.len()); // The last element is removed.
    /// }
    /// ```
    #[storage(read, write)]
    pub fn swap_remove(self, index: u64) -> V {
        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        if STORES_STORAGE_TYPE {
            // This workaround for satisfying the `#[storage]` attribute
            // is explained in the comment above this `StorageVec` impl.
            let _ = __state_preload(b256::zero());
            let _ = __state_clear(b256::zero(), 0);
            panic StorageVecError::MethodDoesNotSupportNestedStorageTypes;
        } else {
            const SIZE_OF_V: u64 = __size_of::<V>();

            let len = self.len();

            if index >= len {
                panic StorageVecError::IndexOutOfBounds(OutOfBounds {
                    length: len,
                    index,
                });
            }

            // 1. Read the element to be removed.
            let (index_elem_slot, index_elem_offset) = self.get_slot_and_offset_of_elem(index);
            let elem_ptr = alloc_bytes(SIZE_OF_V);
            let _ = __state_load_slot(index_elem_slot, elem_ptr, index_elem_offset, SIZE_OF_V);
            let removed_element = elem_ptr.read::<V>();

            // 2. Overwrite element at `index` with the last element (only if `index` is not the last).
            if index != len - 1 {
                let (last_elem_slot, last_elem_offset) = self.get_slot_and_offset_of_elem(len - 1);
                // Reuse `elem_ptr` since `removed_element` is already copied to a local.
                let _ = __state_load_slot(last_elem_slot, elem_ptr, last_elem_offset, SIZE_OF_V);
                __state_update_slot(index_elem_slot, elem_ptr, index_elem_offset, SIZE_OF_V);
            }

            // 3. Update the length header.
            // The original last element's bytes remain as residual but are logically removed.
            let new_len = len - 1;
            __state_update_slot(self.field_id(), __addr_of(new_len), 0, 8);

            removed_element
        }
    }

    /// Sets the `value` at the given `index`.
    ///
    /// # Additional Information
    ///
    /// **This method is not supported for nested storage types and will panic if `V` is a nested storage type.**
    ///
    /// # Arguments
    ///
    /// * `index`: [u64] - The index to set the value at.
    /// * `value`: [V] - The value to be set.
    ///
    /// # Panics
    ///
    /// * `StorageVecError::IndexOutOfBounds`, if `index` is out of bounds.
    /// * `StorageVecError::MethodDoesNotSupportNestedStorageTypes`, if `V` is a nested storage type, i.e. if it is a zero-sized type.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1`    (vector's length)
    /// * Writes: `1`   (update the element in its chunk slot)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::storage::storage_vec::*;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///
    ///     storage.vec.set(0, 15);
    ///     let set_value = storage.vec.get(0).unwrap().read();
    ///     assert_eq(15, set_value);
    /// }
    /// ```
    #[storage(read, write)]
    pub fn set(self, index: u64, value: V) {
        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        if STORES_STORAGE_TYPE {
            // This workaround for satisfying the `#[storage]` attribute
            // is explained in the comment above this `StorageVec` impl.
            let _ = __state_preload(b256::zero());
            let _ = __state_clear(b256::zero(), 0);
            panic StorageVecError::MethodDoesNotSupportNestedStorageTypes;
        } else {
            const SIZE_OF_V: u64 = __size_of::<V>();

            let len = self.len();

            if index >= len {
                panic StorageVecError::IndexOutOfBounds(OutOfBounds {
                    length: len,
                    index,
                });
            }

            // Write the value into the appropriate chunk slot at the computed byte offset.
            let (elem_slot, elem_offset) = self.get_slot_and_offset_of_elem(index);
            __state_update_slot(elem_slot, __addr_of(value), elem_offset, SIZE_OF_V);
        }
    }

    /// Inserts the `value` at the given `index`, moving the current value at `index`
    /// as well as the following indices up by one index.
    ///
    /// # Additional Information
    ///
    /// **This method is not supported for nested storage types and will panic if `V` is a nested storage type.**
    ///
    /// If `index` is equal to vector's length, appends the `value` at the end of the
    /// vector, effectively acting the same as `push`.
    ///
    /// # Arguments
    ///
    /// * `index`: [u64] - The index to insert the value into.
    /// * `value`: [V] - The value to insert.
    ///
    /// # Panics
    ///
    /// * `StorageVecError::IndexOutOfBounds`, if `index` is larger than the length of the vector.
    /// * `StorageVecError::MethodDoesNotSupportNestedStorageTypes`, if `V` is a nested storage type, i.e. if it is a zero-sized type.
    ///
    /// # Number of Storage Accesses
    ///
    /// Shifting operates slot-by-slot using a single storage read per affected slot,
    /// so accesses scale with the number of chunk slots containing elements to shift,
    /// not with `len - index`.
    ///
    /// Let `k = floor(len / elems_per_slot) - floor(index / elems_per_slot)` be
    /// the number of chunk slots following (and including) the insert slot that
    /// contain elements to shift, where `elems_per_slot = CHUNK_MAX_SIZE / size_of::<V>()`.
    ///
    /// ## When appending (`index == len`)
    ///
    /// * Writes: `2`   (vector's length and new element)
    ///
    /// ## When inserting in the middle (`index < len`)
    ///
    /// * Reads: `1`          (vector's length)
    /// * Reads: `k`          (one read per affected slot, loading all its elements at once)
    /// * Writes: `1`         (vector's length)
    /// * Writes: `k`         (last element of each slot promoted to the first position of the next slot)
    /// * Writes: at most `k` (remaining elements of each slot shifted right; skipped when the slot
    ///                        has only one element, or the slot is the last and was partial)
    /// * Writes: `1`         (new element at `index`)
    ///
    /// Total: at most `3k + 3` storage operations.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::storage::storage_vec::*;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     storage.vec.push(5);
    ///     storage.vec.push(15);
    ///
    ///     storage.vec.insert(1, 10);
    ///
    ///     assert_eq(5, storage.vec.get(0).unwrap().read());
    ///     assert_eq(10, storage.vec.get(1).unwrap().read());
    ///     assert_eq(15, storage.vec.get(2).unwrap().read());
    /// }
    /// ```
    #[storage(read, write)]
    pub fn insert(self, index: u64, value: V) {
        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        if STORES_STORAGE_TYPE {
            // This workaround for satisfying the `#[storage]` attribute
            // is explained in the comment above this `StorageVec` impl.
            let _ = __state_preload(b256::zero());
            let _ = __state_clear(b256::zero(), 0);
            panic StorageVecError::MethodDoesNotSupportNestedStorageTypes;
        } else {
            let len = self.len();
            let field_id = self.field_id();

            if index > len {
                panic StorageVecError::IndexOutOfBounds(OutOfBounds {
                    length: len,
                    index,
                });
            }

            // Update the length header first, before touching any elements.
            let new_len = len + 1;
            __state_update_slot(field_id, __addr_of(new_len), 0, 8);

            // Compute the insert position once; used both for shifting and for the final write.
            let (insert_slot, insert_offset) = self.get_slot_and_offset_of_elem(index);

            if index < len {
                // TODO: Move `SIZE_OF_V` to the top of `else`, like in all other methods,
                //       once https://github.com/FuelLabs/sway/issues/7650 is fixed.
                const SIZE_OF_V: u64 = __size_of::<V>();
                const ELEMS_PER_SLOT: u64 = CHUNK_MAX_SIZE / SIZE_OF_V;

                let insert_chunk = index / ELEMS_PER_SLOT;
                let old_last_chunk = (len - 1) / ELEMS_PER_SLOT;
                let new_last_chunk = len / ELEMS_PER_SLOT;

                // One reusable buffer large enough for a full slot's elements.
                let buf = alloc_bytes(ELEMS_PER_SLOT * SIZE_OF_V);

                if old_last_chunk > insert_chunk {
                    // 1. Handle old_last_chunk.
                    // Shift its elements right by 1 to make room at position 0 for the
                    // carry element that will arrive from the preceding chunk.
                    // old_last_chunk >= 1 (since old_last_chunk > insert_chunk >= 0),
                    // so its element area starts at byte 0.
                    let mut old_last_slot = field_id;
                    add_u64_to_b256(old_last_slot, old_last_chunk);

                    let elems_in_old_last = len - old_last_chunk * ELEMS_PER_SLOT;
                    let _ = __state_load_slot(old_last_slot, buf, 0, elems_in_old_last * SIZE_OF_V);

                    if new_last_chunk > old_last_chunk {
                        // Sub-case A: old_last_chunk was full (elems_in_old_last == ELEMS_PER_SLOT).
                        // Its last element overflows to the brand-new last chunk slot.
                        let mut new_last_slot = field_id;
                        add_u64_to_b256(new_last_slot, new_last_chunk);

                        let remaining_bytes = (ELEMS_PER_SLOT - 1) * SIZE_OF_V;
                        __state_store_slot(new_last_slot, buf.add::<u8>(remaining_bytes), SIZE_OF_V);
                        if remaining_bytes > 0 {
                            __state_update_slot(old_last_slot, buf, SIZE_OF_V, remaining_bytes);
                        }
                    } else {
                        // Sub-case B: old_last_chunk was partial. All elements shift right
                        // within the slot; position 0 is freed for the carry from the preceding chunk.
                        __state_update_slot(old_last_slot, buf, SIZE_OF_V, elems_in_old_last * SIZE_OF_V);
                    }

                    // 2. Loop from old_last_chunk-1 down to insert_chunk+1.
                    // Each of these chunks is full (ELEMS_PER_SLOT elements, since they precede
                    // old_last_chunk). Load all elements, move the last to next_slot[0], shift
                    // the rest right within the current slot.
                    // old_last_chunk >= 1 ensures old_last_chunk - 1 does not underflow.
                    let mut next_slot = old_last_slot;
                    let mut chunk = old_last_chunk - 1;
                    while chunk > insert_chunk {
                        // chunk >= 1 (since chunk > insert_chunk >= 0),
                        // guarantees that element area always starts at byte 0.
                        let mut cur_slot = field_id;
                        add_u64_to_b256(cur_slot, chunk);

                        let _ = __state_load_slot(cur_slot, buf, 0, ELEMS_PER_SLOT * SIZE_OF_V);

                        let remaining_bytes = (ELEMS_PER_SLOT - 1) * SIZE_OF_V;
                        __state_update_slot(next_slot, buf.add::<u8>(remaining_bytes), 0, SIZE_OF_V);
                        if remaining_bytes > 0 {
                            __state_update_slot(cur_slot, buf, SIZE_OF_V, remaining_bytes);
                        }
                        next_slot = cur_slot;
                        chunk -= 1;
                    }

                    // 3. Handle insert_chunk.
                    // insert_chunk is full (ELEMS_PER_SLOT elements, since insert_chunk < old_last_chunk).
                    // Load the elements from index onward, move the last to next_slot[0],
                    // shift the rest right within insert_slot.
                    let index_in_chunk = index % ELEMS_PER_SLOT;
                    let elems_to_shift = ELEMS_PER_SLOT - index_in_chunk;
                    let _ = __state_load_slot(insert_slot, buf, insert_offset, elems_to_shift * SIZE_OF_V);

                    let remaining_bytes = (elems_to_shift - 1) * SIZE_OF_V;
                    __state_update_slot(next_slot, buf.add::<u8>(remaining_bytes), 0, SIZE_OF_V);
                    if remaining_bytes > 0 {
                        __state_update_slot(insert_slot, buf, insert_offset + SIZE_OF_V, remaining_bytes);
                    }
                } else {
                    // old_last_chunk == insert_chunk.
                    let index_in_chunk = index % ELEMS_PER_SLOT;
                    let elems_in_insert_chunk = len - insert_chunk * ELEMS_PER_SLOT;
                    let elems_to_shift = elems_in_insert_chunk - index_in_chunk;
                    let _ = __state_load_slot(insert_slot, buf, insert_offset, elems_to_shift * SIZE_OF_V);
                    if new_last_chunk > insert_chunk {
                        // The insert chunk was full (len is a multiple of ELEMS_PER_SLOT), so the
                        // last element overflows into the brand-new next chunk.
                        let mut new_last_slot = field_id;
                        add_u64_to_b256(new_last_slot, new_last_chunk);

                        let remaining_bytes = (elems_to_shift - 1) * SIZE_OF_V;
                        __state_store_slot(new_last_slot, buf.add::<u8>(remaining_bytes), SIZE_OF_V);
                        if remaining_bytes > 0 {
                            __state_update_slot(insert_slot, buf, insert_offset + SIZE_OF_V, remaining_bytes);
                        }
                    } else {
                        // The insert chunk was partial; all elements stay within insert_slot.
                        __state_update_slot(
                            insert_slot,
                            buf,
                            insert_offset + SIZE_OF_V,
                            elems_to_shift * SIZE_OF_V,
                        );
                    }
                }
            }

            // Write the new value at the insert position.
            // TODO: Move `SIZE_OF_V` to the top of `else`, like in all other methods,
            //       once https://github.com/FuelLabs/sway/issues/7650 is fixed.
            const SIZE_OF_V: u64 = __size_of::<V>();
            __state_update_slot(insert_slot, __addr_of(value), insert_offset, SIZE_OF_V);
        }
    }

    /// Returns the length of the vector.
    ///
    /// # Returns
    ///
    /// * [u64] - The length of the vector.
    ///
    /// # Number of Storage Accesses
    ///
    /// ## When `V` is a Non-storage Type
    ///
    /// * Reads: `1`    (vector's length)
    ///
    /// ## When `V` is a Nested Storage Type
    ///
    /// * Reads: `1`    (vector's length)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::storage::storage_vec::*;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///     assert_eq(0, storage.vec.len());
    ///     storage.vec.push(5);
    ///     assert_eq(1, storage.vec.len());
    ///     storage.vec.push(10);
    ///     assert_eq(2, storage.vec.len());
    /// }
    /// ```
    #[storage(read)]
    pub fn len(self) -> u64 {
        // The length is stored as a `u64` at byte offset 0 in slot 0 (`self.field_id()`).
        // `__state_load_word` with offset 0 reads the first word (8 bytes) of the slot.
        // If the slot was never written, it returns 0, which correctly represents an empty vector.
        __state_load_word(self.field_id(), 0)
    }

    /// Returns true if the vector is empty, otherwise false.
    ///
    /// # Returns
    ///
    /// * [bool] - Indicates whether the vector is empty or not.
    ///
    /// # Number of Storage Accesses
    ///
    /// ## When `V` is a Non-storage Type
    ///
    /// * Reads: `1`    (vector's length)
    ///
    /// ## When `V` is a Nested Storage Type
    ///
    /// * Reads: `1`    (vector's length)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::storage::storage_vec::*;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     assert(storage.vec.is_empty());
    ///     storage.vec.push(5);
    ///     assert(!storage.vec.is_empty());
    ///     storage.vec.clear();
    ///     assert(storage.vec.is_empty());
    /// }
    /// ```
    #[storage(read)]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// Swaps the positions of the two elements at the given indices.
    ///
    /// # Additional Information
    ///
    /// **This method is not supported for nested storage types and will panic if `V` is a nested storage type.**
    ///
    /// If `element1_index` and `element2_index` are equal, this method does not read from or write into the storage,
    /// aside from reading the vector's length.
    ///
    /// # Arguments
    ///
    /// * `element1_index`: [u64] - The index of the first element.
    /// * `element2_index`: [u64] - The index of the second element.
    ///
    /// # Panics
    ///
    /// * `StorageVecError::IndexOutOfBounds`, if `element1_index` or `element2_index` is out of bounds.
    /// * `StorageVecError::MethodDoesNotSupportNestedStorageTypes`, if `V` is a nested storage type, i.e. if it is a zero-sized type.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `3`    (vector's length and the two elements; element reads skipped if the indices are equal)
    /// * Writes: `2`   (the swapped elements; skipped if the indices are equal)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::storage::storage_vec::*;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///     storage.vec.push(15);
    ///
    ///     storage.vec.swap(0, 2);
    ///     assert_eq(15, storage.vec.get(0).unwrap().read());
    ///     assert_eq(10, storage.vec.get(1).unwrap().read());
    ///     assert_eq(5, storage.vec.get(2).unwrap().read());
    /// }
    /// ```
    #[storage(read, write)]
    pub fn swap(self, element1_index: u64, element2_index: u64) {
        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        if STORES_STORAGE_TYPE {
            // This workaround for satisfying the `#[storage]` attribute
            // is explained in the comment above this `StorageVec` impl.
            let _ = __state_preload(b256::zero());
            let _ = __state_clear(b256::zero(), 0);
            panic StorageVecError::MethodDoesNotSupportNestedStorageTypes;
        } else {
            const SIZE_OF_V: u64 = __size_of::<V>();

            let len = self.len();

            if element1_index >= len {
                panic StorageVecError::IndexOutOfBounds(OutOfBounds {
                    length: len,
                    index: element1_index,
                });
            }

            if element2_index >= len {
                panic StorageVecError::IndexOutOfBounds(OutOfBounds {
                    length: len,
                    index: element2_index,
                });
            }

            if element1_index == element2_index {
                return;
            }

            let (slot1, offset1) = self.get_slot_and_offset_of_elem(element1_index);
            let (slot2, offset2) = self.get_slot_and_offset_of_elem(element2_index);

            let elem1_ptr = alloc_bytes(SIZE_OF_V);
            let _ = __state_load_slot(slot1, elem1_ptr, offset1, SIZE_OF_V);

            let elem2_ptr = alloc_bytes(SIZE_OF_V);
            let _ = __state_load_slot(slot2, elem2_ptr, offset2, SIZE_OF_V);

            __state_update_slot(slot1, elem2_ptr, offset1, SIZE_OF_V);
            __state_update_slot(slot2, elem1_ptr, offset2, SIZE_OF_V);
        }
    }

    /// Returns the first element of the vector, or `None` if it is empty.
    ///
    /// # Returns
    ///
    /// * [Option<StorageKey<V>>] - The location in storage of the value stored at
    /// the start of the vector, or `None` if the vector is empty.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1`    (vector's length)
    ///
    /// # Examples
    ///
    /// ```sway
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///     assert(storage.vec.first().is_none());
    ///     storage.vec.push(5);
    ///     assert_eq(5, storage.vec.first().unwrap().read());
    /// }
    /// ```
    #[storage(read)]
    pub fn first(self) -> Option<StorageKey<V>> {
        let len = self.len();

        if len == 0 {
            return None;
        }

        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        let field_id = self.field_id();
        let (slot, offset, field_id) = if STORES_STORAGE_TYPE {
            // Nested storage types are stored at a unique `field_id`
            // set to `sha256((0, self.field_id())` to ensure every
            // nested storage type element will use it and store its content
            // in a different slot. The offset within that dedicated slot is 0.
            (field_id, 0, sha256((0, field_id)))
        } else {
            // Element 0 is always in slot 0 at byte offset 8 (after the length header).
            (field_id, 8, field_id)
        };

        Some(StorageKey::<V>::new(slot, offset, field_id))
    }

    /// Returns the last element of the vector, or `None` if it is empty.
    ///
    /// # Returns
    ///
    /// * [Option<StorageKey<V>>] - The location in storage of the value stored at
    /// the end of the vector, or `None` if the vector is empty.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1`    (vector's length)
    ///
    /// # Examples
    ///
    /// ```sway
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///     assert(storage.vec.last().is_none());
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///     assert_eq(10, storage.vec.last().unwrap().read());
    /// }
    /// ```
    #[storage(read)]
    pub fn last(self) -> Option<StorageKey<V>> {
        let len = self.len();

        if len == 0 {
            return None;
        }

        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        let (slot, offset, field_id) = if STORES_STORAGE_TYPE {
            // Nested storage types are stored at a unique `field_id`
            // set to `sha256((len - 1, self.field_id())`.
            let field_id = self.field_id();
            (field_id, 0, sha256((len - 1, field_id)))
        } else {
            let (elem_slot, elem_offset) = self.get_slot_and_offset_of_elem(len - 1);
            (elem_slot, elem_offset, elem_slot)
        };

        Some(StorageKey::<V>::new(slot, offset, field_id))
    }

    /// Reverses the order of elements in the vector, in place.
    ///
    /// # Additional Information
    ///
    /// **This method is not supported for nested storage types and will panic if `V` is a nested storage type.**
    ///
    /// # Panics
    ///
    /// * `StorageVecError::MethodDoesNotSupportNestedStorageTypes`, if `V` is a nested storage type, i.e. if it is a zero-sized type.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1`    (vector's length)
    ///
    /// When all elements fit in the first chunk slot (i.e. `len <= elems_per_slot`,
    /// where `elems_per_slot = CHUNK_MAX_SIZE / size_of::<V>()`):
    ///
    /// * Reads: `1`    (load the entire element area)
    /// * Writes: `1`   (write the reversed element area back)
    ///
    /// Otherwise:
    ///
    /// * Reads + Writes: `2 * floor(len / 2)` (one read and one write per element in each swapped pair)
    ///
    /// # Examples
    ///
    /// ```sway
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///     storage.vec.push(15);
    ///
    ///     storage.vec.reverse();
    ///
    ///     assert_eq(15, storage.vec.get(0).unwrap().read());
    ///     assert_eq(10, storage.vec.get(1).unwrap().read());
    ///     assert_eq(5, storage.vec.get(2).unwrap().read());
    /// }
    /// ```
    #[storage(read, write)]
    pub fn reverse(self) {
        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        if STORES_STORAGE_TYPE {
            // This workaround for satisfying the `#[storage]` attribute
            // is explained in the comment above this `StorageVec` impl.
            let _ = __state_preload(b256::zero());
            let _ = __state_clear(b256::zero(), 0);
            panic StorageVecError::MethodDoesNotSupportNestedStorageTypes;
        } else {
            const SIZE_OF_V: u64 = __size_of::<V>();
            const ELEMS_PER_SLOT: u64 = CHUNK_MAX_SIZE / SIZE_OF_V;

            let len = self.len();

            if len < 2 {
                return;
            }

            if len <= ELEMS_PER_SLOT {
                // All elements live in slot 0 (element area starts at byte 8).
                // Load the whole area, reverse it in memory, write it back in one shot.
                let slot = self.field_id();
                let elem_bytes = len * SIZE_OF_V;
                let buf = alloc_bytes(elem_bytes);
                let temp = alloc_bytes(SIZE_OF_V);
                let _ = __state_load_slot(slot, buf, 8, elem_bytes);

                // Reverse in memory: swap element pairs from the outside in.
                let mut left = 0u64;
                let mut right = len - 1;
                while left < right {
                    let left_ptr = buf.add::<u8>(left * SIZE_OF_V);
                    let right_ptr = buf.add::<u8>(right * SIZE_OF_V);
                    left_ptr.copy_to::<u8>(temp, SIZE_OF_V);
                    right_ptr.copy_to::<u8>(left_ptr, SIZE_OF_V);
                    temp.copy_to::<u8>(right_ptr, SIZE_OF_V);
                    left += 1;
                    right -= 1;
                }

                __state_update_slot(slot, buf, 8, elem_bytes);
            } else {
                // Elements span multiple slots: swap element pairs one by one.
                let left_ptr = alloc_bytes(SIZE_OF_V);
                let right_ptr = alloc_bytes(SIZE_OF_V);

                let mid = len / 2;
                let mut i = 0;
                while i < mid {
                    let j = len - i - 1;

                    let (left_slot, left_offset) = self.get_slot_and_offset_of_elem(i);
                    let (right_slot, right_offset) = self.get_slot_and_offset_of_elem(j);

                    let _ = __state_load_slot(left_slot, left_ptr, left_offset, SIZE_OF_V);
                    let _ = __state_load_slot(right_slot, right_ptr, right_offset, SIZE_OF_V);

                    __state_update_slot(left_slot, right_ptr, left_offset, SIZE_OF_V);
                    __state_update_slot(right_slot, left_ptr, right_offset, SIZE_OF_V);

                    i += 1;
                }
            }
        }
    }

    /// Replaces all elements in the vector with `value`.
    ///
    /// # Additional Information
    ///
    /// **This method is not supported for nested storage types and will panic if `V` is a nested storage type.**
    ///
    /// # Arguments
    ///
    /// * `value`: [V] - Value to copy to each element of the vector.
    ///
    /// # Panics
    ///
    /// * `StorageVecError::MethodDoesNotSupportNestedStorageTypes`, if `V` is a nested storage type, i.e. if it is a zero-sized type.
    ///
    /// # Number of Storage Accesses
    ///
    /// Let `N = ceil(len / elems_per_slot)` be the number of chunk slots in use, where
    /// `elems_per_slot = CHUNK_MAX_SIZE / size_of::<V>()`.
    ///
    /// * Reads: `1`    (vector's length)
    /// * Writes: `N`   (one write per chunk slot)
    ///
    /// # Examples
    ///
    /// ```sway
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///     storage.vec.push(15);
    ///
    ///     storage.vec.fill(20);
    ///
    ///     assert_eq(20, storage.vec.get(0).unwrap().read());
    ///     assert_eq(20, storage.vec.get(1).unwrap().read());
    ///     assert_eq(20, storage.vec.get(2).unwrap().read());
    /// }
    /// ```
    #[storage(read, write)]
    pub fn fill(self, value: V) {
        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        if STORES_STORAGE_TYPE {
            // This workaround for satisfying the `#[storage]` attribute
            // is explained in the comment above this `StorageVec` impl.
            let _ = __state_preload(b256::zero());
            let _ = __state_clear(b256::zero(), 0);
            panic StorageVecError::MethodDoesNotSupportNestedStorageTypes;
        } else {
            const SIZE_OF_V: u64 = __size_of::<V>();
            const ELEMS_PER_SLOT: u64 = CHUNK_MAX_SIZE / SIZE_OF_V;

            let len = self.len();

            if len == 0 {
                return;
            }

            let last_chunk = (len - 1) / ELEMS_PER_SLOT;

            // Build a full-slot buffer filled with `value` repeated ELEMS_PER_SLOT times.
            // This lets us write each chunk slot in a single call.
            let buf = alloc_bytes(ELEMS_PER_SLOT * SIZE_OF_V);
            let value_ptr = __addr_of(value);
            let mut i = 0u64;
            while i < ELEMS_PER_SLOT {
                value_ptr.copy_to::<u8>(buf.add::<u8>(i * SIZE_OF_V), SIZE_OF_V);
                i += 1;
            }

            let field_id = self.field_id();

            // Chunk 0: element area starts at byte 8 (after the length header), so we
            // must use __state_update_slot to avoid clobbering the header.
            // If chunk 0 is also the last chunk, write only the elements that exist.
            let elems_in_chunk0: u64 = if last_chunk == 0 {
                len
            } else {
                ELEMS_PER_SLOT
            };
            __state_update_slot(field_id, buf, 8, elems_in_chunk0 * SIZE_OF_V);

            if last_chunk > 0 {
                // Chunks 1 through last_chunk-1 (if any):
                // all are full, so we can write them in a single call each.
                let mut chunk = 1u64;
                while chunk < last_chunk {
                    let mut slot = field_id;
                    add_u64_to_b256(slot, chunk);

                    __state_store_slot(slot, buf, ELEMS_PER_SLOT * SIZE_OF_V);
                    chunk += 1;
                }

                // Last chunk: may be partial.
                let mut last_slot = field_id;
                add_u64_to_b256(last_slot, last_chunk);

                let elems_in_last = len - last_chunk * ELEMS_PER_SLOT;
                __state_store_slot(last_slot, buf, elems_in_last * SIZE_OF_V);
            }
        }
    }

    /// Resizes `self` in place so that `len` is equal to `new_len`.
    ///
    /// # Additional Information
    ///
    /// If `new_len` is greater than `len`, `self` is extended by the difference, with each
    /// additional element being set to `value`. If the `new_len` is less than `len`, `self` is
    /// simply truncated.
    ///
    /// > **_WARNING:_** **If `V` is a nested storage type and `new_len` is less than `len`,
    /// `resize` will reduce the vector length and remove the remaining elements _from the vector_,
    /// but the nested content of those removed elements will remain in the storage.**
    ///
    /// When shrinking, the residual bytes of removed elements remain in their chunk slots but are
    /// logically non-existent and will be overwritten by future `push`/`resize` calls.
    ///
    /// # Arguments
    ///
    /// * `new_len`: [u64] - The new length to expand or truncate to.
    /// * `value`: [V] - The value to set new elements to, if the `new_len` is greater than the current length.
    ///
    /// # Number of Storage Accesses
    ///
    /// ## When `V` is a Non-storage Type
    ///
    /// ### When shrinking (`new_len < len`) or unchanged (`new_len == len`)
    ///
    /// * Reads: `1`    (vector's length)
    /// * Writes: `1`   (vector's length)
    ///
    /// ### When growing (`new_len > len`)
    ///
    /// * Reads: `1`    (vector's length)
    /// * Writes: up to `new_len - len + 1`   (vector's length and one write per new element)
    ///
    /// ## When `V` is a Nested Storage Type
    ///
    /// * Writes: `1`   (to write the new vector's length)
    ///
    /// # Examples
    ///
    /// ```sway
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///
    ///     storage.vec.resize(4, 20);
    ///
    ///     assert_eq(5, storage.vec.get(0).unwrap().read());
    ///     assert_eq(10, storage.vec.get(1).unwrap().read());
    ///     assert_eq(20, storage.vec.get(2).unwrap().read());
    ///     assert_eq(20, storage.vec.get(3).unwrap().read());
    ///
    ///     storage.vec.resize(2, 0);
    ///
    ///     assert_eq(5, storage.vec.get(0).unwrap().read());
    ///     assert_eq(10, storage.vec.get(1).unwrap().read());
    ///     assert_eq(None, storage.vec.get(2));
    ///     assert_eq(None, storage.vec.get(3));
    /// }
    /// ```
    #[storage(read, write)]
    pub fn resize(self, new_len: u64, value: V) {
        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        if STORES_STORAGE_TYPE {
            // For nested storage types, we only adjust the length.
            // The eventual existing content of the nested storage types
            // is not touched, matching the behavior of `push` and `pop`.
            // Add a dummy preload to satisfy the `#[storage(read, write)]` attribute
            // for this branch (see the comment above this `StorageVec` impl).
            let _ = __state_preload(b256::zero());
            __state_store_slot(self.field_id(), __addr_of(new_len), 8);
        } else {
            const SIZE_OF_V: u64 = __size_of::<V>();

            let len = self.len();

            // Update the length header first, before writing any new elements.
            //
            // We must update the length header (the first 8 bytes of slot 0) **before**
            // writing new elements. If slot 0 was previously unused, `__state_update_slot`
            // requires the offset to be within the currently used size. Writing the length
            // first establishes those 8 bytes so subsequent element writes are valid.
            __state_update_slot(self.field_id(), __addr_of(new_len), 0, 8);

            if new_len > len {
                // Growing: write the `value` into each newly added element position.
                let mut i = len;
                while i < new_len {
                    let (elem_slot, elem_offset) = self.get_slot_and_offset_of_elem(i);
                    __state_update_slot(elem_slot, __addr_of(value), elem_offset, SIZE_OF_V);
                    i += 1;
                }
            }
            // If shrinking, only the length header update (done above) is needed.
            // Residual bytes of removed elements remain in their chunk slots but are
            // logically non-existent and will be overwritten on the next push/resize.
        }
    }

    /// Stores given `vec` as a `StorageVec`.
    ///
    /// # Additional Information
    ///
    /// This will overwrite any existing values in the `StorageVec`.
    ///
    /// **This method is not supported for nested storage types and will panic if `V` is a nested storage type.**
    ///
    /// # Arguments
    ///
    /// * `vec`: [Vec<V>] - The vector to store in storage.
    ///
    /// # Panics
    ///
    /// * `StorageVecError::MethodDoesNotSupportNestedStorageTypes`, if `V` is a nested storage type, i.e. if it is a zero-sized type.
    ///
    /// # Number of Storage Accesses
    ///
    /// Elements are written to chunk slots in bulk. Every slot holds at most
    /// `CHUNK_MAX_SIZE / size_of::<V>()` elements. The first slot additionally stores the
    /// vector's length before the element area.
    ///
    /// * Writes: `ceil(len / (CHUNK_MAX_SIZE / size_of::<V>()))` slots
    ///   (at least 1 even when `len == 0`, because the vector's length is always written)
    ///
    /// # Examples
    ///
    /// ```sway
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///     let mut heap_vec = Vec::<u64>::new();
    ///     heap_vec.push(5);
    ///     heap_vec.push(10);
    ///     heap_vec.push(15);
    ///
    ///     storage.vec.store_vec(heap_vec);
    ///
    ///     assert_eq(5, storage.vec.get(0).unwrap().read());
    ///     assert_eq(10, storage.vec.get(1).unwrap().read());
    ///     assert_eq(15, storage.vec.get(2).unwrap().read());
    /// }
    /// ```
    #[storage(write)]
    pub fn store_vec(self, vec: Vec<V>) {
        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        if STORES_STORAGE_TYPE {
            // This workaround for satisfying the `#[storage]` attribute
            // is explained in the comment above this `StorageVec` impl.
            let _ = __state_clear(b256::zero(), 0);
            panic StorageVecError::MethodDoesNotSupportNestedStorageTypes;
        } else {
            let len = vec.len();
            let field_id = self.field_id();

            // Write the length header first so that subsequent `__state_update_slot` calls
            // targeting slot 0 at offset ≥ 8 have a valid base.
            __state_update_slot(field_id, __addr_of(len), 0, 8);

            if len == 0 {
                return;
            }

            let (elements_ptr, elements_bytes) = vec.as_raw_slice().into_parts(); // Every slot holds the same number of elements. The element area of each
// slot is CHUNK_MAX_SIZE bytes, so `slot_elem_bytes` is the maximum byte
// count for that area. Note: `slot_elem_bytes <= CHUNK_MAX_SIZE` always,
// since we floor-divide then multiply back.

// TODO: Move `SIZE_OF_V` to the top of `else`, like in all other methods,
//       once https://github.com/FuelLabs/sway/issues/7650 is fixed.
            const SIZE_OF_V: u64 = __size_of::<V>();
            const ELEMS_PER_SLOT: u64 = CHUNK_MAX_SIZE / SIZE_OF_V;
            const SLOT_ELEM_BYTES: u64 = ELEMS_PER_SLOT * SIZE_OF_V;

            // All elements fit in the first slot (after the 8-byte header).
            if elements_bytes <= SLOT_ELEM_BYTES {
                __state_update_slot(field_id, elements_ptr, 8, elements_bytes);
            } else { // Write the first slot's full element area into slot 0 (at byte offset 8).
                __state_update_slot(field_id, elements_ptr, 8, SLOT_ELEM_BYTES); // Write the remaining elements into subsequent chunk slots.
                let mut bytes_written = SLOT_ELEM_BYTES;
                let mut chunk_number: u64 = 1;
                while bytes_written < elements_bytes {
                    let remaining_bytes = elements_bytes - bytes_written;
                    let chunk_bytes = if remaining_bytes > SLOT_ELEM_BYTES {
                        SLOT_ELEM_BYTES
                    } else {
                        remaining_bytes
                    };

                    let mut chunk_slot = field_id;
                    add_u64_to_b256(chunk_slot, chunk_number);

                    __state_store_slot(
                        chunk_slot,
                        elements_ptr
                            .add::<u8>(bytes_written),
                        chunk_bytes,
                    );

                    bytes_written += chunk_bytes;
                    chunk_number += 1;
                }
            }
        }
    }

    /// Returns all elements contained in `self` as a `Vec<V>`.
    ///
    /// # Additional Information
    ///
    /// **This method is not supported for nested storage types and will panic if `V` is a nested storage type.**
    ///
    /// # Returns
    ///
    /// * [Vec<V>] - The vector containing all elements of `self`.
    ///
    /// # Panics
    ///
    /// * `StorageVecError::MethodDoesNotSupportNestedStorageTypes`, if `V` is a nested storage type, i.e. if it is a zero-sized type.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1 + ceil(len / (CHUNK_MAX_SIZE / size_of::<V>()))`
    ///   (one for the vector's length, plus one per chunk slot with element data;
    ///    element reads skipped when the vector is empty)
    ///
    /// # Examples
    ///
    /// ```sway
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///     let mut vec = Vec::<u64>::new();
    ///     vec.push(5);
    ///     vec.push(10);
    ///     vec.push(15);
    ///
    ///     storage.vec.store_vec(vec);
    ///
    ///     let loaded_vec = storage.vec.load_vec();
    ///
    ///     assert_eq(5, loaded_vec.get(0).unwrap());
    ///     assert_eq(10, loaded_vec.get(1).unwrap());
    ///     assert_eq(15, loaded_vec.get(2).unwrap());
    /// }
    /// ```
    #[storage(read)]
    pub fn load_vec(self) -> Vec<V> {
        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        if STORES_STORAGE_TYPE {
            // This workaround for satisfying the `#[storage]` attribute
            // is explained in the comment above this `StorageVec` impl.
            let _ = __state_preload(self.field_id());
            panic StorageVecError::MethodDoesNotSupportNestedStorageTypes;
        } else {
            let len = self.len();

            if len == 0 {
                return Vec::new();
            }

            // Every slot holds the same number of elements (same calculation as in `store_vec`).

            // TODO: Move `SIZE_OF_V` to the top of `else`, like in all other methods,
            //       once https://github.com/FuelLabs/sway/issues/7650 is fixed.
            const SIZE_OF_V: u64 = __size_of::<V>();
            const ELEMS_PER_SLOT: u64 = CHUNK_MAX_SIZE / SIZE_OF_V;
            const SLOT_ELEM_BYTES: u64 = ELEMS_PER_SLOT * SIZE_OF_V;

            let elements_bytes = len * SIZE_OF_V;
            let elements_ptr = alloc_bytes(elements_bytes);

            let field_id = self.field_id();
            if elements_bytes <= SLOT_ELEM_BYTES {
                // All elements fit in the first slot (starting at byte offset 8).
                let _ = __state_load_slot(field_id, elements_ptr, 8, elements_bytes);
            } else {
                // Load the first slot's full element area from slot 0 (starting at byte offset 8).
                let _ = __state_load_slot(field_id, elements_ptr, 8, SLOT_ELEM_BYTES);

                // Load the remaining elements from subsequent chunk slots.
                let mut bytes_read = SLOT_ELEM_BYTES;
                let mut chunk_number: u64 = 1;
                while bytes_read < elements_bytes {
                    let chunk_bytes = if elements_bytes - bytes_read > SLOT_ELEM_BYTES {
                        SLOT_ELEM_BYTES
                    } else {
                        elements_bytes - bytes_read
                    };

                    let mut chunk_slot = field_id;
                    add_u64_to_b256(chunk_slot, chunk_number);

                    let _ = __state_load_slot(
                        chunk_slot,
                        elements_ptr
                            .add::<u8>(bytes_read),
                        0,
                        chunk_bytes,
                    );

                    bytes_read += chunk_bytes;
                    chunk_number += 1;
                }
            }

            Vec::from(raw_slice::from_parts::<V>(elements_ptr, len))
        }
    }

    /// Returns an [Iterator] to iterate over this `StorageVec`.
    ///
    /// # Returns
    ///
    /// * [StorageVecIter<V>] - The iterator over this `StorageVec`.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1`    (vector's length)
    ///
    /// # Examples
    ///
    /// ```sway
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///     storage.vec.push(15);
    ///
    ///     let iter = storage.vec.iter();
    ///
    ///     assert_eq(5, iter.next().unwrap().read());
    ///     assert_eq(10, iter.next().unwrap().read());
    ///     assert_eq(15, iter.next().unwrap().read());
    ///
    ///     let iter = storage.vec.iter();
    ///
    ///     for elem in storage.vec.iter() {
    ///         let elem_value = elem.read();
    ///         log(elem_value);
    ///     }
    /// }
    /// ```
    #[storage(read)]
    pub fn iter(self) -> StorageVecIter<V> {
        StorageVecIter {
            values: self,
            len: self.len(),
            index: 0,
        }
    }
}

#[cfg(experimental_dynamic_storage = true)]
impl<V> Iterator for StorageVecIter<V> {
    type Item = StorageKey<V>;
    fn next(ref mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            return None;
        }

        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        let (slot, offset, field_id) = if STORES_STORAGE_TYPE {
            // For nested storage types, each element has a unique `field_id`
            // set to `sha256((index, storage_vec_field_id))` to ensure each
            // nested storage type element stores its content in a different slot.
            let storage_vec_field_id = self.values.field_id();
            (storage_vec_field_id, 0, sha256((self.index, storage_vec_field_id)))
        } else {
            // For non-zero-sized types, values are spread across chunk slots.
            // Compute the exact slot and byte offset via `get_slot_and_offset_of_elem`.
            let (elem_slot, elem_offset) = self.values.get_slot_and_offset_of_elem(self.index);
            (elem_slot, elem_offset, elem_slot)
        };

        let result = Some(StorageKey::<V>::new(slot, offset, field_id));

        self.index += 1;

        result
    }
}

#[cfg(experimental_dynamic_storage = true)]
#[inline(always)]
fn add_u64_to_b256(ref mut num: b256, val: u64) {
    asm(num: num, val: val) {
        wqop num num val i0;
    }
}

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
    /// will remind in the storage.**
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
    /// but the nested content of those removed elements will remind in the storage.**
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

        // Handle cases where elements are less than the size of word and pad to the size of a word
        let slice = if size_V_bytes < 8 {
            let vec_slice = vec.as_raw_slice();
            let number_of_words = 8 * vec.len();
            let ptr = alloc_bytes(number_of_words);
            let mut i = 0;
            while i < vec.len() {
                // Insert into raw slice as offsets of 1 word per element
                // (size_of_word * element)
                vec_slice
                    .ptr()
                    .add::<V>(i)
                    .copy_bytes_to(ptr.add_uint_offset(8 * i), size_V_bytes);
                i += 1;
            }

            raw_slice::from_parts::<V>(ptr, number_of_words)
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

                let bytes = if size_V_bytes < 8 {
                    // Len * size_of_word
                    len * 8
                } else {
                    len * size_V_bytes
                };

                let number_of_slots = (bytes + 31) >> 5;
                let ptr = alloc_bytes(number_of_slots * 32);
                // Load the stored slice into the pointer.
                let _ = __state_load_quad(sha256(self.field_id()), ptr, number_of_slots);

                if size_V_bytes < 8 {
                    let len_bytes = len * size_V_bytes;
                    let new_vec = alloc_bytes(len_bytes);
                    let mut i = 0;
                    while i < len {
                        // The stored vec is offset with 1 word per element, remove the padding for elements less than the size of a word
                        // (size_of_word * element)
                        ptr
                            .add_uint_offset((8 * i))
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

// A dynamic-slot-based `StorageVec` has two distinctive modes
// of operations and storage access:
//
// 1. Storing non-zero-sized types
//    E.g., `StorageVec<u64>`. In this mode, the whole content
//    of the storage vec is stored in a single dynamic slot at `self.field_id`
//    and all the `StorageVec` methods are fully supported.
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
// E.g., `push` requires a read and a write when `V` is a nested storage type,
// and otherwise only a `write`. If we choose `#[storage(read, write)]` as the
// purity attribute, we will get a warning that `read` is not needed, if `push`
// is called for non-storage types.
//
// This is because the purity warnings are generated at the end of the compilation
// pipeline.
//
// Currently, the only way to avoid purity warnings when `V` is a nested storage type
// is the following:
// - we omit `read` in `#[storage]` attributes and leave only `write`. The purity
//   attributes will reflect only the "When `V` is a Nested Storage Type" part of
//   the method doc-comment.
// - for methods that are not supported at all for nested types, like, e.g. `set`,
//   we add a dummy `let _ = __state_clear(b256::zero(), 0);` call to simulate `write`
//   access. This is a workaround, but currently we don't have better options.
//   The real issue here is the problematic duality of the original `StorageVec` API,
//   which currently cannot be expressed via type system and trait constraints.
#[cfg(experimental_dynamic_storage = true)]
impl<V> StorageKey<StorageVec<V>> {
    // TODO: We should have only one `STORES_STORAGE_TYPE` constant
    //       declared here as an associated constant. This is currently not possible
    //       because of these two limitations:
    //       1. Having the below `const` uncommented causes compiler panic:
    //          TODO-IG!: Add link to GitHub issue.
    //       2. All `StorageKey<StorageVec<V>>` types will actually share the same
    //          constant value because they have the same path, and are identified
    //          in IR only by path. **This is a known limitation that can lead to
    //          invalid compilation if 1. is solved before.**
    //       **We can use the below constant only when both above issues are solved.**
    //       Until then, we will have a local `STORES_STORAGE_TYPE` constant in
    //       every method that needs it.

    // /// True if the `StorageVec` stores a storage type, i.e., if `V` is a storage type.
    // const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

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
    /// * Internal preloads: `1` (to preload current content)
    /// * Writes: `1`            (to append the `value`)
    ///
    /// ## When `V` is a Nested Storage Type
    ///
    /// * Reads: `1`    (to get the vector's length)
    /// * Writes: `1`   (to write the new, increased, vector's length)
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
    #[storage(write)]
    pub fn push(self, value: V) {
        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        if STORES_STORAGE_TYPE {
            // Nothing to actually store. We just need to increment
            // the vector's length. It is `get` that will return the
            // proper `StorageKey` for the nested storage type `V`.
            let len = read_slot::<u64>(self.field_id(), 0).unwrap_or(0);
            write_slot(self.field_id(), len + 1);
        } else {
            append_slot(self.field_id(), value);
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
    /// will remind in the storage.
    ///
    /// # Returns
    ///
    /// * [Option<V>] - The last element of the vector, or `None` if the vector is empty, or it contains a nested storage type.
    ///
    /// # Number of Storage Accesses
    ///
    /// ## When `V` is a Non-storage Type
    ///
    /// * Preloads: `1` (to get the vector's length)
    /// * Reads: `1`    (to read the current content)
    /// * Writes: `1`   (to write the truncated content)
    ///
    /// ## When `V` is a Nested Storage Type
    ///
    /// * Reads: `1`    (to get the vector's length)
    /// * Writes: `1`   (to write the new, decreased, vector's length)
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

        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        if STORES_STORAGE_TYPE {
            // Reducing len by 1, effectively removing the last item.
            // Note that the value stored in the nested storage type
            // **does not get removed from storage**.
            write_slot(self.field_id(), len - 1);
            None
        } else {
            let len_in_bytes = len * __size_of::<V>();
            let new_len_in_bytes = len_in_bytes - __size_of::<V>();

            // 1. Get the current slot content.
            let content_ptr = alloc_bytes(len_in_bytes);
            let _ = __state_load_slot(self.field_id(), content_ptr, 0, len_in_bytes);

            // 2. Truncate the slot content.
            // Not that if the `new_len_in_bytes` is zero, the slot will still
            // remain occupied but with an empty content. This fits the semantic
            // of the empty `StorageVec` so there is no need for an additional
            // check for zero and clearing the slot as a special case.
            __state_store_slot(self.field_id(), content_ptr, new_len_in_bytes);

            // 3. Return the last element.
            Some(content_ptr.add::<u8>(new_len_in_bytes).read::<V>())
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
    /// * Preloads: `1` (to get the vector's length)
    ///
    /// ## When `V` is a Nested Storage Type
    ///
    /// * Reads: `1`    (to get the vector's length)
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

        let (offset, field_id) = if STORES_STORAGE_TYPE {
            // For nested storage types, we want to use a unique `field_id`
            // set to `sha256((index, self.field_id())` to ensure every
            // nested storage type element will use it and store its content
            // in a different slot.
            (0, sha256((index, self.field_id())))
        } else {
            // For non-zero-sized types, we know that their values are
            // stored in the `self.field_id()` slot.
            (index * __size_of::<V>(), self.field_id())
        };

        Some(StorageKey::<V>::new(self.field_id(), offset, field_id))
    }

    /// Removes the element at the given `index` and moves all the elements at the following indices
    /// down one index. Returns the removed element.
    ///
    /// # Additional Information
    ///
    /// **This method is not supported for nested storage types and will panic if `V` is a nested storage type.**
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
    /// * Preloads: `1` (to get vector's length)
    /// * Loads: `1` (to read current slot content)
    /// * Writes: `1` (to write truncated content)
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
    #[storage(write)]
    pub fn remove(self, index: u64) -> V {
        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        if STORES_STORAGE_TYPE {
            // This workaround for satisfying the `#[storage]` attribute
            // is explained in the comment above this `StorageVec` impl.
            let _ = __state_clear(b256::zero(), 0);
            panic StorageVecError::MethodDoesNotSupportNestedStorageTypes;
        } else {
            let size_of_v = __size_of::<V>();
            let len = __state_preload(self.field_id()) / size_of_v;

            if index >= len {
                panic StorageVecError::IndexOutOfBounds(OutOfBounds {
                    length: len,
                    index
                });
            }

            let len_in_bytes = len * size_of_v;
            let index_offset = index * size_of_v;
            let new_len_in_bytes = len_in_bytes - size_of_v;

            // 1. Read current slot content.
            let content_ptr = alloc_bytes(len_in_bytes);
            let _ = __state_load_slot(self.field_id(), content_ptr, 0, len_in_bytes);

            // 2. Read the element to be removed before overriding the content.
            let removed_element = content_ptr.add::<u8>(index_offset).read::<V>();

            // 3. Shift everything after `index` one element to the left.
            let content_after_removed_len = len_in_bytes - index_offset - size_of_v;
            if content_after_removed_len > 0 {
                // We first have to copy the trailing content to a new location,
                // because memory copying requires non-overlapping memory locations.
                let content_after_removed_ptr = alloc_bytes(content_after_removed_len);
                content_ptr
                    .add::<u8>(index_offset + size_of_v)
                    .copy_bytes_to(content_after_removed_ptr, content_after_removed_len);

                // Finally, copy the copy of the trailing content back into the original content.
                content_after_removed_ptr
                    .copy_bytes_to(content_ptr.add::<u8>(index_offset), content_after_removed_len);
            }

            // 4. Write new content without the removed element to storage.
            __state_store_slot(self.field_id(), content_ptr, new_len_in_bytes);

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
    /// * Preloads: `1` (to get vector's length)
    /// * Loads: `1` (to read current slot content)
    /// * Writes: `1` (to write content with the last element placed at `index`, truncated by one element)
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
    #[storage(write)]
    pub fn swap_remove(self, index: u64) -> V {
        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        if STORES_STORAGE_TYPE {
            // This workaround for satisfying the `#[storage]` attribute
            // is explained in the comment above this `StorageVec` impl.
            let _ = __state_clear(b256::zero(), 0);
            panic StorageVecError::MethodDoesNotSupportNestedStorageTypes;
        } else {
            let size_of_v = __size_of::<V>();
            let len = __state_preload(self.field_id()) / size_of_v;

            if index >= len {
                panic StorageVecError::IndexOutOfBounds(OutOfBounds {
                    length: len,
                    index
                });
            }

            let len_in_bytes = len * size_of_v;
            let index_offset = index * size_of_v;
            let new_len_in_bytes = len_in_bytes - size_of_v;

            // 1. Read current slot content.
            let content_ptr = alloc_bytes(len_in_bytes);
            let _ = __state_load_slot(self.field_id(), content_ptr, 0, len_in_bytes);

            // 2. Read the element to be replaced/removed before overriding the content.
            let removed_element = content_ptr.add::<u8>(index_offset).read::<V>();

            // 3. Overwrite element at `index` with the last element (only if `index` is not already the last).
            if index != len - 1 {
                let last_element_ptr = content_ptr.add::<u8>(new_len_in_bytes);
                last_element_ptr.copy_bytes_to(content_ptr.add::<u8>(index_offset), size_of_v);
            }

            // 4. Write new content without the last element to storage.
            __state_store_slot(self.field_id(), content_ptr, new_len_in_bytes);

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
    /// * Preloads: `1` (to get the vector's length)
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
    #[storage(write)]
    pub fn set(self, index: u64, value: V) {
        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        if STORES_STORAGE_TYPE {
            // This workaround for satisfying the `#[storage]` attribute
            // is explained in the comment above this `StorageVec` impl.
            let _ = __state_clear(b256::zero(), 0);
            panic StorageVecError::MethodDoesNotSupportNestedStorageTypes;
        } else {
            let len = __state_preload(self.field_id()) / __size_of::<V>();
            if index >= len {
                panic StorageVecError::IndexOutOfBounds(OutOfBounds {
                    length: len,
                    index
                });
            }

            __state_update_slot(self.field_id(), __addr_of::<V>(value), index * __size_of::<V>(), __size_of::<V>());
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
    /// * Preloads: `1` (to get vector's length)
    /// * Loads: `0` in case of an append at the end, or `1` if inserting
    /// * Writes: `1` in case of an append at the end, or `2` if inserting
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
    #[storage(write)]
    pub fn insert(self, index: u64, value: V) {
        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        if STORES_STORAGE_TYPE {
            // This workaround for satisfying the `#[storage]` attribute
            // is explained in the comment above this `StorageVec` impl.
            let _ = __state_clear(b256::zero(), 0);
            panic StorageVecError::MethodDoesNotSupportNestedStorageTypes;
        } else {
            let len = __state_preload(self.field_id()) / __size_of::<V>();
            if index > len {
                panic StorageVecError::IndexOutOfBounds(OutOfBounds {
                    length: len,
                    index
                });
            }

            if index == len {
                append_slot(self.field_id(), value);
            } else {
                let len_in_bytes = len * __size_of::<V>();
                let index_offset = index * __size_of::<V>();

                // 1. Get the current slot content.
                let content_ptr = alloc_bytes(len_in_bytes);
                let _ = __state_load_slot(self.field_id(), content_ptr, 0, len_in_bytes);

                // 2. Write the `value` at `index`.
                __state_update_slot(self.field_id(), __addr_of::<V>(value), index_offset, __size_of::<V>());

                // 3. Write the previous content that should come after the inserted `value`.
                let content_after_index_ptr = content_ptr.add::<u8>(index_offset);
                let content_after_index_len = len_in_bytes - index_offset;
                __state_update_slot(self.field_id(), content_after_index_ptr, index_offset + __size_of::<V>(), content_after_index_len);
            }
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
    /// * Preloads: `1` (to get the vector's length)
    ///
    /// ## When `V` is a Nested Storage Type
    ///
    /// * Reads: `1`    (to get the vector's length)
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
        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        if STORES_STORAGE_TYPE {
            read_slot::<u64>(self.field_id(), 0).unwrap_or(0)
        } else {
            __state_preload(self.field_id()) / __size_of::<V>()
        }
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
    /// * Preloads: `1` (to get the vector's length)
    ///
    /// ## When `V` is a Nested Storage Type
    ///
    /// * Reads: `1`    (to get the vector's length)
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
    /// aside from preloading the vectors length.
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
    /// * Preloads: `1` (to get the vector's length)
    /// * Loads: `1` (to read current slot content when the indices differ)
    /// * Writes: `1` (to write the swapped content when the indices differ)
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
    #[storage(write)]
    pub fn swap(self, element1_index: u64, element2_index: u64) {
        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        if STORES_STORAGE_TYPE {
            // This workaround for satisfying the `#[storage]` attribute
            // is explained in the comment above this `StorageVec` impl.
            let _ = __state_clear(b256::zero(), 0);
            panic StorageVecError::MethodDoesNotSupportNestedStorageTypes;
        } else {
            let size_of_v = __size_of::<V>();
            let len = __state_preload(self.field_id()) / size_of_v;

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

            let len_in_bytes = len * size_of_v;
            let element1_offset = element1_index * size_of_v;
            let element2_offset = element2_index * size_of_v;

            let content_ptr = alloc_bytes(len_in_bytes);
            let _ = __state_load_slot(self.field_id(), content_ptr, 0, len_in_bytes);

            // Copy first element to a temporary.
            let element1_value_ptr = alloc_bytes(size_of_v);
            content_ptr
                .add::<u8>(element1_offset)
                .copy_bytes_to(element1_value_ptr, size_of_v);

            content_ptr
                .add::<u8>(element2_offset)
                .copy_bytes_to(content_ptr.add::<u8>(element1_offset), size_of_v);
            element1_value_ptr
                .copy_bytes_to(content_ptr.add::<u8>(element2_offset), size_of_v);

            __state_store_slot(self.field_id(), content_ptr, len_in_bytes);
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
        let len = self.len();

        if len == 0 {
            return None;
        }

        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        let field_id = if STORES_STORAGE_TYPE {
            // Nested storage types are stored at a unique `field_id`
            // set to `sha256((<index>, self.field_id())` to ensure every
            // nested storage type element will use it and store its content
            // in a different slot.
            sha256((0, self.field_id()))
        } else {
            // For non-zero-sized types, their values are
            // stored in the `self.field_id()` slot.
            self.field_id()
        };

        Some(StorageKey::<V>::new(self.field_id(), 0, field_id))
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
        let len = self.len();

        if len == 0 {
            return None;
        }

        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        let (offset, field_id) = if STORES_STORAGE_TYPE {
            // Nested storage types are stored at a unique `field_id`
            // set to `sha256((<index>, self.field_id())` to ensure every
            // nested storage type element will use it and store its content
            // in a different slot.
            (0, sha256((len - 1, self.field_id())))
        } else {
            // For non-zero-sized types, their values are
            // stored in the `self.field_id()` slot.
            ((len - 1) * __size_of::<V>(), self.field_id())
        };

        Some(StorageKey::<V>::new(self.field_id(), offset, field_id))
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
    /// * Preloads: `1` (to get vector's length)
    /// * Loads: `1` when `self.len() >= 2`, otherwise `0`
    /// * Writes: `1` when `self.len() >= 2`, otherwise `0`
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
    #[storage(write)]
    pub fn reverse(self) {
        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        if STORES_STORAGE_TYPE {
            // This workaround for satisfying the `#[storage]` attribute
            // is explained in the comment above this `StorageVec` impl.
            let _ = __state_clear(b256::zero(), 0);
            panic StorageVecError::MethodDoesNotSupportNestedStorageTypes;
        } else {
            let size_of_v = __size_of::<V>();
            let len = __state_preload(self.field_id()) / size_of_v;

            if len < 2 {
                return;
            }

            let len_in_bytes = len * size_of_v;
            let content_ptr = alloc_bytes(len_in_bytes);
            let _ = __state_load_slot(self.field_id(), content_ptr, 0, len_in_bytes);

            // Allocate a temporary for swaps.
            let tmp_ptr = alloc_bytes(size_of_v);

            let mid = len / 2;
            let mut i = 0;
            while i < mid {
                let left_offset = i * size_of_v;
                let right_offset = (len - i - 1) * size_of_v;

                // tmp <- left.
                content_ptr
                    .add::<u8>(left_offset)
                    .copy_bytes_to(tmp_ptr, size_of_v);

                // right -> left.
                content_ptr
                    .add::<u8>(right_offset)
                    .copy_bytes_to(content_ptr.add::<u8>(left_offset), size_of_v);

                // tmp -> right.
                tmp_ptr
                    .copy_bytes_to(content_ptr.add::<u8>(right_offset), size_of_v);

                i += 1;
            }

            __state_store_slot(self.field_id(), content_ptr, len_in_bytes);
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
    /// * Preloads: `1` (to get vector's length)
    /// * Writes: `1` if `self.len() > 0`, otherwise `0`
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

    ///     storage.vec.fill(20);
    ///
    ///     assert_eq(20, storage.vec.get(0).unwrap().read());
    ///     assert_eq(20, storage.vec.get(1).unwrap().read());
    ///     assert_eq(20, storage.vec.get(2).unwrap().read());
    /// }
    /// ```
    #[storage(write)]
    pub fn fill(self, value: V) {
        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        if STORES_STORAGE_TYPE {
            // This workaround for satisfying the `#[storage]` attribute
            // is explained in the comment above this `StorageVec` impl.
            let _ = __state_clear(b256::zero(), 0);
            panic StorageVecError::MethodDoesNotSupportNestedStorageTypes;
        } else {
            let size_of_v = __size_of::<V>();
            let len = __state_preload(self.field_id()) / size_of_v;

            if len == 0 {
                return;
            }

            let len_in_bytes = len * size_of_v;
            let content_ptr = alloc_bytes(len_in_bytes);

            let value_ptr = __addr_of::<V>(value);

            let mut i = 0;
            while i < len {
                value_ptr
                    .copy_bytes_to(content_ptr.add::<u8>(i * size_of_v), size_of_v);
                i += 1;
            }

            __state_store_slot(self.field_id(), content_ptr, len_in_bytes);
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
    /// but the nested content of those removed elements will remind in the storage.**
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
    /// * Preloads: `1` (to get vector's length)
    /// * Loads: `1` when `new_len < len`, or when `new_len > len && len > 0`; otherwise `0`
    /// * Writes: `1` when `new_len != len`; otherwise `0`
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
    #[storage(write)]
    pub fn resize(self, new_len: u64, value: V) {
        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        if STORES_STORAGE_TYPE {
            // For nested storage types, we only adjust the length.
            // The eventual existing content of the nested storage types
            // is not touched, matching the behavior of `push` and `pop`.
            write_slot(self.field_id(), new_len);
        } else {
            let size_of_v = __size_of::<V>();
            let len = __state_preload(self.field_id()) / size_of_v;

            if new_len == len {
                return;
            }

            let len_in_bytes = len * size_of_v;
            let new_len_in_bytes = new_len * size_of_v;

            let content_ptr = alloc_bytes(new_len_in_bytes);

            if new_len < len {
                // Truncation keeps the prefix and drops the tail.
                // We need to load only the prefix part, `new_len_in_bytes`.
                let _ = __state_load_slot(self.field_id(), content_ptr, 0, new_len_in_bytes);
            } else {
                // Growth keeps existing content and fills the tail in-memory before the write.
                // We need to load all the existing content, `len_in_bytes`.
                if len_in_bytes > 0 {
                    let _ = __state_load_slot(self.field_id(), content_ptr, 0, len_in_bytes);
                }

                let value_ptr = __addr_of::<V>(value);
                let mut i = len;
                while i < new_len {
                    value_ptr
                        .copy_bytes_to(content_ptr.add::<u8>(i * size_of_v), size_of_v);
                    i += 1;
                }
            }

            __state_store_slot(self.field_id(), content_ptr, new_len_in_bytes);
        }
    }

    /// Stores given `vec` as a `StorageVec`.
    ///
    /// # Additional Information
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
    /// * Writes: `1`
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
            let (ptr, len) = vec.as_raw_slice().into_parts();
            __state_store_slot(self.field_id(), ptr, len);
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
    /// * Preloads: `1` (to get vector's length)
    /// * Reads: `1`
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
            let len_in_bytes = __state_preload(self.field_id());
            if len_in_bytes == 0 {
                Vec::new()
            } else {
                let content_ptr = alloc_bytes(len_in_bytes);
                let _ = __state_load_slot(self.field_id(), content_ptr, 0, len_in_bytes);
                Vec::from(raw_slice::from_parts::<V>(content_ptr, len_in_bytes / __size_of::<V>()))
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
    /// ## When `V` is a Non-storage Type
    ///
    /// * Preloads: `1` (to get the vector's length)
    ///
    /// ## When `V` is a Nested Storage Type
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

        let storage_vec_field_id = self.values.field_id();

        const STORES_STORAGE_TYPE: bool = __size_of::<V>() == 0;

        let (offset, field_id) = if STORES_STORAGE_TYPE {
            // For nested storage types, we know that they use a unique `field_id`
            // set to `sha256((self.index, storage_vec_field_id)` that ensurs every
            // nested storage type element will use it and store its content
            // in a different slot.
            (0, sha256((self.index, storage_vec_field_id)))
        } else {
            // For non-zero-sized types, we know that their values are
            // stored in the `self.field_id()` slot.
            (self.index * __size_of::<V>(), storage_vec_field_id)
        };

        let result = Some(StorageKey::<V>::new(storage_vec_field_id, offset, field_id));

        self.index += 1;

        result
    }
}

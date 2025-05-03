library;

use ::alloc::{alloc_bytes, realloc_bytes};
use ::assert::assert;
use ::hash::*;
use ::option::Option::{self, *};
use ::storage::storage_api::*;
use ::storage::storage_key::*;
use ::vec::{Vec, VecIter};
use ::iterator::Iterator;
use ::codec::*;
use ::debug::*;

/// A persistent vector struct.
pub struct StorageVec<V> {}

impl<V> StorageKey<StorageVec<V>> {
    /// Appends the value to the end of the vector.
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
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     let five = 5_u64;
    ///     storage.vec.push(five);
    ///     assert(five == storage.vec.get(0).unwrap().read());
    /// }
    /// ```
    #[storage(read, write)]
    pub fn push(self, value: V) {
        let len = read::<u64>(self.field_id(), 0).unwrap_or(0);

        // Storing the value at the current length index (if this is the first item, starts off at 0)
        let key = sha256(self.field_id());
        let offset = offset_calculator::<V>(len);
        write::<V>(key, offset, value);

        // Incrementing the length
        write(self.field_id(), 0, len + 1);
    }

    /// Removes the last element of the vector and returns it, `None` if empty.
    ///
    /// # Returns
    ///
    /// * [Option<V>] - The last element `V` or `None`.
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
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     let five = 5_u64;
    ///     storage.vec.push(five);
    ///     let popped_value = storage.vec.pop().unwrap();
    ///     assert(five == popped_value);
    ///     let none_value = storage.vec.pop();
    ///     assert(none_value.is_none());
    /// }
    /// ```
    #[storage(read, write)]
    pub fn pop(self) -> Option<V> {
        let len = read::<u64>(self.field_id(), 0).unwrap_or(0);

        // if the length is 0, there is no item to pop from the vec
        if len == 0 {
            return None;
        }

        // reduces len by 1, effectively removing the last item in the vec
        write(self.field_id(), 0, len - 1);

        let key = sha256(self.field_id());
        let offset = offset_calculator::<V>(len - 1);
        read::<V>(key, offset)
    }

    /// Gets the value in the given index, `None` if index is out of bounds.
    ///
    /// # Arguments
    ///
    /// * `index`: [u64] - The index of the vec to retrieve the item from.
    ///
    /// # Returns
    ///
    /// * [Option<StorageKey<V>>] - Describes the raw location in storage of the value stored at
    /// `key` or `None` if out of bounds.
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
    ///     assert(five == storage.vec.get(0).unwrap().read());
    ///     assert(storage.vec.get(1).is_none());
    /// }
    /// ```
    #[storage(read)]
    pub fn get(self, index: u64) -> Option<StorageKey<V>> {
        let len = read::<u64>(self.field_id(), 0).unwrap_or(0);

        // if the index is larger or equal to len, there is no item to return
        if len <= index {
            return None;
        }

        let key = sha256(self.field_id());
        let offset = offset_calculator::<V>(index);
        // This StorageKey can be read by the standard storage api.
        // Field Id must be unique such that nested storage vecs work as they have a
        // __size_of() zero and will therefore always have an offset of zero.
        Some(StorageKey::<V>::new(key, offset, sha256((index, key))))
    }

    /// Removes the element in the given index and moves all the elements in the following indexes
    /// down one index. Also returns the element.
    ///
    /// # Additional Information
    ///
    ///  **_WARNING:_** Expensive for larger vecs.
    ///
    /// # Arguments
    ///
    /// * `index`: [u64] - The index of the vec to remove the item from.
    ///
    /// # Returns
    ///
    /// * [V] - The element that has been removed at the index.
    ///
    /// # Reverts
    ///
    /// * Reverts if index is larger or equal to length of the vec.
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
    ///     assert(10 == removed_value);
    ///     assert(storage.vec.len() == 2);
    /// }
    /// ```
    #[storage(read, write)]
    pub fn remove(self, index: u64) -> V {
        let len = read::<u64>(self.field_id(), 0).unwrap_or(0);

        // if the index is larger or equal to len, there is no item to remove
        assert(index < len);

        // gets the element before removing it, so it can be returned
        let key = sha256(self.field_id());
        let removed_offset = offset_calculator::<V>(index);
        let removed_element = read::<V>(key, removed_offset).unwrap();

        // for every element in the vec with an index greater than the input index,
        // shifts the index for that element down one
        let mut count = index + 1;
        while count < len {
            // gets the storage location for the previous index and
            // moves the element of the current index into the previous index
            let write_offset = offset_calculator::<V>(count - 1);
            let read_offset = offset_calculator::<V>(count);
            write::<V>(key, write_offset, read::<V>(key, read_offset).unwrap());

            count += 1;
        }

        // decrements len by 1
        write(self.field_id(), 0, len - 1);

        removed_element
    }

    /// Removes the element at the specified index and fills it with the last element.
    /// This does not preserve ordering and returns the element.
    ///
    /// # Arguments
    ///
    /// * `index`: [u64] - The index of the vec to remove the item from.
    ///
    /// # Returns
    ///
    /// * [V] - The element that has been removed at the index.
    ///
    /// # Reverts
    ///
    /// * Reverts if index is larger or equal to length of the vec.
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
    ///     assert(5 == removed_value);
    ///     let swapped_value = storage.vec.get(0).unwrap().read();
    ///     assert(15 == swapped_value);
    /// }
    /// ```
    #[storage(read, write)]
    pub fn swap_remove(self, index: u64) -> V {
        let len = read::<u64>(self.field_id(), 0).unwrap_or(0);

        // if the index is larger or equal to len, there is no item to remove
        assert(index < len);

        let key = sha256(self.field_id());
        // gets the element before removing it, so it can be returned
        let element_offset = offset_calculator::<V>(index);
        let element_to_be_removed = read::<V>(key, element_offset).unwrap();

        let last_offset = offset_calculator::<V>(len - 1);
        let last_element = read::<V>(key, last_offset).unwrap();

        write::<V>(key, element_offset, last_element);

        // decrements len by 1
        write(self.field_id(), 0, len - 1);

        element_to_be_removed
    }

    /// Sets or mutates the value at the given index.
    ///
    /// # Arguments
    ///
    /// * `index`: [u64] - The index of the vec to set the value at
    /// * `value`: [V] - The value to be set
    ///
    /// # Reverts
    ///
    /// * Reverts if index is larger than or equal to the length of the vec.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `2`
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
    ///     storage.vec.push(15);
    ///
    ///     storage.vec.set(0, 20);
    ///     let set_value = storage.vec.get(0).unwrap().read();
    ///     assert(20 == set_value);
    /// }
    /// ```
    #[storage(read, write)]
    pub fn set(self, index: u64, value: V) {
        let len = read::<u64>(self.field_id(), 0).unwrap_or(0);

        // if the index is higher than or equal len, there is no element to set
        assert(index < len);

        let key = sha256(self.field_id());
        let offset = offset_calculator::<V>(index);
        write::<V>(key, offset, value);
    }

    /// Inserts the value at the given index, moving the current index's value
    /// as well as the following index's value up by one index.
    ///
    /// # Additional Information
    ///
    /// > **_WARNING:_** Expensive for larger vecs.
    ///
    /// # Arguments
    ///
    /// * `index`: [u64] - The index of the vec to insert the item into.
    /// * `value`: [V] - The value to insert into the vec.
    ///
    /// # Reverts
    ///
    /// * Reverts if index is larger than the length of the vec.
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
    ///     assert(5 == storage.vec.get(0).unwrap().read());
    ///     assert(10 == storage.vec.get(1).unwrap().read());
    ///     assert(15 == storage.vec.get(2).unwrap().read());
    /// }
    /// ```
    #[storage(read, write)]
    pub fn insert(self, index: u64, value: V) {
        let len = read::<u64>(self.field_id(), 0).unwrap_or(0);

        // if the index is larger than len, there is no space to insert
        assert(index <= len);

        // if len is 0, index must also be 0 due to above check
        let key = sha256(self.field_id());
        if len == index {
            let offset = offset_calculator::<V>(index);
            write::<V>(key, offset, value);

            // increments len by 1
            write(self.field_id(), 0, len + 1);

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
            write::<V>(key, write_offset, read::<V>(key, read_offset).unwrap());

            if count == 0 {
                break;
            }
            count -= 1;
        }

        // inserts the value into the now unused index
        let offset = offset_calculator::<V>(index);
        write::<V>(key, offset, value);

        // increments len by 1
        write(self.field_id(), 0, len + 1);
    }

    /// Returns the length of the vector.
    ///
    /// # Returns
    ///
    /// * [u64] - The stored length of the vector.
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
    ///     assert(0 == storage.vec.len());
    ///     storage.vec.push(5);
    ///     assert(1 == storage.vec.len());
    ///     storage.vec.push(10);
    ///     assert(2 == storage.vec.len());
    /// }
    /// ```
    #[storage(read)]
    pub fn len(self) -> u64 {
        read::<u64>(self.field_id(), 0).unwrap_or(0)
    }

    /// Checks whether the len is zero or not.
    ///
    /// # Returns
    ///
    /// * [bool] - Indicates whether the vector is or is not empty.
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
    ///     assert(true == storage.vec.is_empty());
    ///
    ///     storage.vec.push(5);
    ///
    ///     assert(false == storage.vec.is_empty());
    ///
    ///     storage.vec.clear();
    ///
    ///     assert(true == storage.vec.is_empty());
    /// }
    /// ```
    #[storage(read)]
    pub fn is_empty(self) -> bool {
        read::<u64>(self.field_id(), 0).unwrap_or(0) == 0
    }

    /// Swaps two elements.
    ///
    /// # Arguments
    ///
    /// * `element1_index`: [u64] - The index of the first element.
    /// * `element2_index`: [u64] - The index of the second element.
    ///
    /// # Reverts
    ///
    /// * If `element1_index` or `element2_index` is greater than the length of the vector.
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
    ///     assert(15 == storage.vec.get(0).unwrap().read());
    ///     assert(10 == storage.vec.get(1).unwrap().read());
    ///     assert(5 == storage.vec.get(2).unwrap().read());
    /// ```
    #[storage(read, write)]
    pub fn swap(self, element1_index: u64, element2_index: u64) {
        let len = read::<u64>(self.field_id(), 0).unwrap_or(0);
        assert(element1_index < len);
        assert(element2_index < len);

        if element1_index == element2_index {
            return;
        }

        let key = sha256(self.field_id());
        let element1_offset = offset_calculator::<V>(element1_index);
        let element2_offset = offset_calculator::<V>(element2_index);

        let element1_value = read::<V>(key, element1_offset).unwrap();

        write::<V>(
            key,
            element1_offset,
            read::<V>(key, element2_offset)
                .unwrap(),
        );
        write::<V>(key, element2_offset, element1_value);
    }

    /// Returns the first element of the vector, or `None` if it is empty.
    ///
    /// # Returns
    ///
    /// * [Option<StorageKey<V>>] - Describes the raw location in storage of the value stored at
    /// the start of the vector or zero if the vector is empty.
    ///
    /// # Number of Storage Accesses
    ///
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
    ///     assert(storage.vec.first().is_none());
    ///
    ///     storage.vec.push(5);
    ///
    ///     assert(5 == storage.vec.first().unwrap().read());
    /// }
    /// ```
    #[storage(read)]
    pub fn first(self) -> Option<StorageKey<V>> {
        let key = sha256(self.field_id());
        match read::<u64>(self.field_id(), 0).unwrap_or(0) {
            0 => None,
            _ => Some(StorageKey::<V>::new(key, 0, sha256((0, key)))),
        }
    }

    /// Returns the last element of the vector, or `None` if it is empty.
    ///
    /// # Returns
    ///
    /// * [Option<StorageKey<V>>] - Describes the raw location in storage of the value stored at
    /// the end of the vector or zero if the vector is empty.
    ///
    /// # Number of Storage Accesses
    ///
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
    ///     assert(storage.vec.last().is_none());
    ///
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///
    ///     assert(10 == storage.vec.last().unwrap().read());
    /// }
    /// ```
    #[storage(read)]
    pub fn last(self) -> Option<StorageKey<V>> {
        let key = sha256(self.field_id());
        match read::<u64>(self.field_id(), 0).unwrap_or(0) {
            0 => None,
            len => {
                let offset = offset_calculator::<V>(len - 1);
                Some(StorageKey::<V>::new(key, offset, sha256((len - 1, key))))
            },
        }
    }

    /// Reverses the order of elements in the vector, in place.
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
    ///     storage.vec.reverse();
    ///
    ///     assert(15 == storage.vec.get(0).unwrap().read());
    ///     assert(10 == storage.vec.get(1).unwrap().read());
    ///     assert(5 == storage.vec.get(2).unwrap().read());
    /// }
    /// ```
    #[storage(read, write)]
    pub fn reverse(self) {
        let len = read::<u64>(self.field_id(), 0).unwrap_or(0);

        if len < 2 {
            return;
        }

        let key = sha256(self.field_id());
        let mid = len / 2;
        let mut i = 0;
        while i < mid {
            let i_offset = offset_calculator::<V>(i);
            let other_offset = offset_calculator::<V>(len - i - 1);

            let element1_value = read::<V>(key, i_offset).unwrap();

            write::<V>(key, i_offset, read::<V>(key, other_offset).unwrap());
            write::<V>(key, other_offset, element1_value);

            i += 1;
        }
    }

    /// Fills `self` with elements by cloning `value`.
    ///
    /// # Arguments
    ///
    /// * `value`: [V] - Value to copy to each element of the vector.
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
    ///     storage.vec.fill(20);
    ///
    ///     assert(20 == storage.vec.get(0).unwrap().read());
    ///     assert(20 == storage.vec.get(1).unwrap().read());
    ///     assert(20 == storage.vec.get(2).unwrap().read());
    /// }
    /// ```
    #[storage(read, write)]
    pub fn fill(self, value: V) {
        let len = read::<u64>(self.field_id(), 0).unwrap_or(0);

        let key = sha256(self.field_id());
        let mut i = 0;
        while i < len {
            let offset = offset_calculator::<V>(i);
            write::<V>(key, offset, value);
            i += 1;
        }
    }

    /// Resizes `self` in place so that `len` is equal to `new_len`.
    ///
    /// # Additional Information
    ///
    /// If `new_len` is greater than `len`, `self` is extended by the difference, with each
    /// additional slot being filled with `value`. If the `new_len` is less than `len`, `self` is
    /// simply truncated.
    ///
    /// # Arguments
    ///
    /// * `new_len`: [u64] - The new length to expand or truncate to
    /// * `value`: [V] - The value to fill into new slots if the `new_len` is greater than the current length
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
    ///     storage.vec.resize(4, 20);
    ///
    ///     assert(5 == storage.vec.get(0).unwrap().read());
    ///     assert(10 == storage.vec.get(1).unwrap().read());
    ///     assert(20 == storage.vec.get(2).unwrap().read());
    ///     assert(20 == storage.vec.get(3).unwrap().read());
    ///
    ///     storage.vec.resize(2, 0);
    ///
    ///     assert(5 == storage.vec.get(0).unwrap().read());
    ///     assert(10 == storage.vec.get(1).unwrap().read());
    ///     assert(None == storage.vec.get(2));
    ///     assert(None == storage.vec.get(3));
    /// }
    /// ```
    #[storage(read, write)]
    pub fn resize(self, new_len: u64, value: V) {
        let mut len = read::<u64>(self.field_id(), 0).unwrap_or(0);
        let key = sha256(self.field_id());
        while len < new_len {
            let offset = offset_calculator::<V>(len);
            write::<V>(key, offset, value);
            len += 1;
        }
        write::<u64>(self.field_id(), 0, new_len);
    }

    // TODO: This should be moved into the vec.sw file and `From<StorageKey<StorageVec>> for Vec`
    // implemented instead of this when https://github.com/FuelLabs/sway/issues/409 is resolved.
    // Implementation will change from this:
    // ```sway
    // let my_vec = Vec::new();
    // storage.storage_vec.store_vec(my_vec);
    // let other_vec = storage.storage_vec.load_vec();
    // ```
    // To this:
    // ```sway
    // let my_vec = Vec::new();
    // storage.storage_vec = my_vec.into();
    // let other_vec = Vec::from(storage.storage_vec);
    // ```
    /// Stores a `Vec` as a `StorageVec`.
    ///
    /// # Additional Information
    ///
    /// This will overwrite any existing values in the `StorageVec`.
    ///
    /// # Arguments
    ///
    /// * `vec`: [Vec<V>] - The vector to store in storage.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Writes - `2`
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
    ///     assert(5 == storage.vec.get(0).unwrap().read());
    ///     assert(10 == storage.vec.get(1).unwrap().read());
    ///     assert(15 == storage.vec.get(2).unwrap().read());
    /// }
    /// ```
    #[storage(write)]
    pub fn store_vec(self, vec: Vec<V>) {
        let size_V_bytes = __size_of::<V>();

        // Handle cases where elements are less than the size of word and pad to the size of a word
        let slice = if size_V_bytes < 8 {
            let number_of_words = 8 * vec.len();
            let ptr = alloc_bytes(number_of_words);
            let mut i = 0;
            for element in vec.iter() {
                // Insert into raw slice as offsets of 1 word per element
                // (size_of_word * element)
                ptr.add_uint_offset(8 * i).write(element);
                i += 1;
            }
            raw_slice::from_parts::<V>(ptr, number_of_words)
        } else {
            raw_slice::from_parts::<V>(vec.ptr(), vec.len())
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
        write::<u64>(self.field_id(), 0, vec.len());
    }

    /// Load a `Vec` from the `StorageVec`.
    ///
    /// # Additional Information
    ///
    /// This method does not work for any `V` type that has a 0 size, such as `StorageVec` itself. Meaning you cannot use this method on a `StorageVec<StorageVec<T>>`.
    ///
    /// # Returns
    ///
    /// * [Option<Vec<V>>] - The vector constructed from storage or `None`.
    ///
    /// # Reverts
    ///
    /// * If the size of type `V` is 0.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads - `2`
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
    ///     let returned_vec = storage.vec.load_vec();
    ///
    ///     assert(5 == returned_vec.get(0).unwrap());
    ///     assert(10 == returned_vec.get(1).unwrap());
    ///     assert(15 == returned_vec.get(2).unwrap());
    /// }
    /// ```
    #[storage(read)]
    pub fn load_vec(self) -> Vec<V> {
        // Get the length of the slice that is stored.
        match read::<u64>(self.field_id(), 0).unwrap_or(0) {
            0 => Vec::new(),
            len => {
                // Get the number of storage slots needed based on the size.
                let size_V_bytes = __size_of::<V>();

                assert(size_V_bytes != 0);

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
    /// * [StorageVecIter<V>] - The struct which can be iterated over.
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
    ///     // Get the iterator
    ///     let iter = storage.vec.iter();
    ///
    ///     assert_eq(5, iter.next().unwrap().read());
    ///     assert_eq(10, iter.next().unwrap().read());
    ///     assert_eq(15, iter.next().unwrap().read());
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
            len: read::<u64>(self.field_id(), 0).unwrap_or(0),
            index: 0,
        }
    }
}

pub struct StorageVecIter<V> {
    values: StorageKey<StorageVec<V>>,
    len: u64,
    index: u64,
}

impl<V> Iterator for StorageVecIter<V> {
    type Item = StorageKey<V>;
    fn next(ref mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            return None
        }

        let key = sha256(self.values.field_id());
        let offset = offset_calculator::<V>(self.index);
        let result = Some(StorageKey::<V>::new(key, offset, sha256((self.index, key))));

        self.index += 1;

        result
    }
}

// Add padding to type so it can correctly use the storage api
fn offset_calculator<T>(offset: u64) -> u64 {
    let size_in_bytes = __size_of::<T>();
    let size_in_bytes = (size_in_bytes + (8 - 1)) - ((size_in_bytes + (8 - 1)) % 8);
    (offset * size_in_bytes) / 8
}

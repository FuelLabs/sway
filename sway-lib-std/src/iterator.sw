//! The iterator trait to iterate over elements.
library;

use ::option::Option::{self, *};

pub trait Iterator {
    /// The type of the elements being iterated over.
    type Item;
    /// Advances the iterator and returns the next value.
    ///
    /// # Additional Information
    ///
    /// Returns [`None`] when iteration is finished. Individual iterator
    /// implementations may choose to resume iteration, and so calling `next()`
    /// again may or may not eventually start returning [`Some(Item)`] again at some
    /// point.
    ///
    /// # Undefined Behavior
    ///
    /// Modifying underlying collection during iteration is a logical error and
    /// results in undefined behavior. E.g.:
    ///
    /// ```sway
    /// let mut vec = Vec::new();
    ///
    /// vec.push(1);
    ///
    /// let mut iter = vec.iter();
    ///
    /// vec.clear(); // Collection modified.
    ///
    /// let _ = iter.next(); // Undefined behavior.
    /// ```
    ///
    /// # Examples
    ///
    /// ```sway
    /// let mut vec = Vec::new();
    ///
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    ///
    /// let mut iter = vec.iter();
    ///
    /// // A call to next() returns the next value...
    /// assert_eq(Some(1), iter.next());
    /// assert_eq(Some(2), iter.next());
    /// assert_eq(Some(3), iter.next());
    ///
    /// // ... and then `None` once it's over.
    /// assert_eq(None, iter.next());
    ///
    /// // More calls may or may not return `None`.
    /// // In the case of `Vec`, they always will.
    /// assert_eq(None, iter.next());
    /// assert_eq(None, iter.next());
    /// ```
    fn next(ref mut self) -> Option<Self::Item>;
}

// Array Iterator

impl<T, const N: u64> [T; N] {
    pub fn iter(self) -> ArrayIterator<T, N> {
        ArrayIterator {
            array: self,
            idx: 0,
        }
    }
}

pub struct ArrayIterator<T, const N: u64> {
    array: [T; N],
    idx: u64,
}

impl<T, const N: u64> Iterator for ArrayIterator<T, N> {
    type Item = T;
    fn next(ref mut self) -> Option<Self::Item> {
        if self.idx >= N {
            None
        } else {
            let elem = __elem_at(&self.array, self.idx);
            self.idx += 1;
            Some(*elem)
        }
    }
}

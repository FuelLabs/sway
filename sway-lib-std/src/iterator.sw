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

#[cfg(experimental_const_generics = true)]
impl<T, const N: u64> [T; N] {
    fn iter(self) -> ArrayIterator<T, N> {
        ArrayIterator { array: self, idx: 0 }
    }
}

#[cfg(experimental_const_generics = true)]
pub struct ArrayIterator<T, const N: u64> {
    array: [T; N],
    idx: u64,
}

#[cfg(experimental_const_generics = true)]
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

// Tests

#[cfg(experimental_const_generics = true)]
#[test]
fn ok_array_iterator_manual() {
    use ::assert::*;
    let array: [u64; 3] = [1u64, 2u64, 3u64];

    let mut iterator = array.iter();
    let a = iterator.next();
    let b = iterator.next();
    let c = iterator.next();
    let d = iterator.next();

    assert(a == Some(1u64));
    assert(b == Some(2u64));
    assert(c == Some(3u64));
    assert(d == None);
}

#[cfg(experimental_const_generics = true)]
#[test]
fn ok_array_iterator_for() {
    use ::assert::*;
    let array: [u64; 3] = [1u64, 2u64, 3u64];

    let mut value = 0;
    for v in array.iter() {
        value += v;
    }
    assert(value == 6);
}

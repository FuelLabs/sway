library;

use ::option::Option::{self, *};

pub trait Iterator {
    /// The type of the elements being iterated over.
    type Item;
    /// Advances the iterator and returns the next value.
    ///
    /// Returns [`None`] when iteration is finished. Individual iterator
    /// implementations may choose to resume iteration, and so calling `next()`
    /// again may or may not eventually start returning [`Some(Item)`] again at some
    /// point.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let mut a = Vec::new();
    ///
    /// a.push(1);
    /// a.push(2);
    /// a.push(3);
    ///
    /// let mut iter = a.iter();
    ///
    /// // A call to next() returns the next value...
    /// assert_eq!(Some(1), iter.next());
    /// assert_eq!(Some(2), iter.next());
    /// assert_eq!(Some(3), iter.next());
    ///
    /// // ... and then None once it's over.
    /// assert_eq!(None, iter.next());
    ///
    /// // More calls may or may not return `None`. Here, they always will.
    /// assert_eq!(None, iter.next());
    /// assert_eq!(None, iter.next());
    /// ```
    fn next(ref mut self) -> Option<Self::Item>;
}

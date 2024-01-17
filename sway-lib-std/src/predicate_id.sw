//! A wrapper around the `b256` type to help enhance type-safety.
library;

use ::convert::From;
use ::hash::{Hash, Hasher};

/// The `PredicateId` type, a struct wrapper around the inner `b256` value.
pub struct PredicateId {
    /// The underlying raw `b256` data of the predicate.
    value: b256,
}

impl core::ops::Eq for PredicateId {
    fn eq(self, other: Self) -> bool {
        self.value == other.value
    }
}

/// Functions for casting between the `b256` and `PredicateId` types.
impl From<b256> for PredicateId {
    /// Casts raw `b256` data to an `PredicateId`.
    ///
    /// # Arguments
    ///
    /// * `bits`: [b256] - The raw `b256` data to be casted.
    ///
    /// # Returns
    ///
    /// * [PredicateId] - The newly created `PredicateId` from the raw `b256`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///    let predicate_id = PredicateId::from(ZERO_B256);
    /// }
    /// ```
    fn from(bits: b256) -> Self {
        Self { value: bits }
    }

    /// Casts an `PredicateId` to raw `b256` data.
    ///
    /// # Returns
    ///
    /// * [b256] - The underlying raw `b256` data of the `PredicateId`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///     let predicate_id = PredicateId::from(ZERO_B256);
    ///     let b256_data = predicate_id.into();
    ///     assert(b256_data == ZERO_B256);
    /// }
    /// ```
    fn into(self) -> b256 {
        self.value
    }
}

impl Hash for PredicateId {
    fn hash(self, ref mut state: Hasher) {
        let PredicateId { value } = self;
        value.hash(state);
    }
}


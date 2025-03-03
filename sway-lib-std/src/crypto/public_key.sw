library;

use ::b512::B512;
use ::bytes::Bytes;
use ::alloc::alloc_bytes;
use ::constants::ZERO_B256;
use ::convert::{From, TryFrom, TryInto};
use ::option::Option::{self, *};
use ::hash::*;

/// Asymmetric public key, i.e. verifying key, in uncompressed form.
///
/// # Additional Information
///
/// It should be noted that while Secp256k1 and Secp256r1 uses 64 byte public keys, Ed25519 uses 32 byte public keys.
pub struct PublicKey {
    /// The underlying raw data of the public key.
    bytes: Bytes,
}

impl PublicKey {
    /// Creates a new instance of a PublicKey signature.
    ///
    /// # Returns
    ///
    /// [PublicKey] - A public key.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::PublicKey;
    ///
    /// fn foo() {
    ///     let new_key = PublicKey::new();
    ///     assert(new_key.bytes().len() == 0);
    /// }
    /// ```
    pub fn new() -> Self {
        Self {
            bytes: Bytes::new(),
        }
    }

    /// Returns the underlying raw `Bytes` data of the public key.
    ///
    /// # Returns
    ///
    /// * [Bytes] - The raw data of the public key.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::PublicKey;
    ///
    /// fn foo() -> {
    ///     let new_key = PublicKey::new();
    ///     assert(new_key.bytes().len() == 64);
    /// }
    /// ```
    pub fn bytes(self) -> Bytes {
        self.bytes
    }

    /// Returns whether the public key is the zero public key.
    ///
    /// # Returns
    ///
    /// * [bool] - `true` if the public key is zero, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::PublicKey;
    ///
    /// fn foo() -> {
    ///     let new_key = PublicKey::new();
    ///     assert(new_key.is_zero() == true);
    /// }
    /// ```
    pub fn is_zero(self) -> bool {
        let mut iter = 0;
        while iter < self.bytes.len() {
            if self.bytes.get(iter).unwrap() != 0 {
                return false;
            }
            iter += 1;
        }

        true
    }
}

impl From<B512> for PublicKey {
    fn from(bits: B512) -> Self {
        Self {
            bytes: Bytes::from(raw_slice::from_parts::<u8>(__addr_of(bits), 64)),
        }
    }
}

impl From<(b256, b256)> for PublicKey {
    fn from(components: (b256, b256)) -> Self {
        Self {
            bytes: Bytes::from(raw_slice::from_parts::<u8>(__addr_of(components), 64)),
        }
    }
}

// Used for Ed25519 signatures
impl From<b256> for PublicKey {
    fn from(bits: b256) -> Self {
        Self {
            bytes: Bytes::from(bits),
        }
    }
}

impl TryFrom<Bytes> for PublicKey {
    fn try_from(bytes: Bytes) -> Option<Self> {
        // Public key can only have a length of 32 or 64 bytes
        if bytes.len() == 32 || bytes.len() == 64 {
            Some(Self { bytes })
        } else {
            None
        }
    }
}

impl TryInto<(b256, b256)> for PublicKey {
    fn try_into(self) -> Option<(b256, b256)> {
        if self.bytes.len() != 64 {
            return None;
        }

        let b256_1 = asm(bits: self.bytes.ptr()) {
            bits: b256
        };
        let b256_2 = asm(bits: self.bytes.ptr().add_uint_offset(32)) {
            bits: b256
        };

        Some((b256_1, b256_2))
    }
}

impl TryInto<B512> for PublicKey {
    fn try_into(self) -> Option<B512> {
        if self.bytes.len() != 64 {
            return None;
        }

        let b256_1 = asm(bits: self.bytes.ptr()) {
            bits: b256
        };
        let b256_2 = asm(bits: self.bytes.ptr().add_uint_offset(32)) {
            bits: b256
        };
        Some(B512::from((b256_1, b256_2)))
    }
}

// Used for Ed25519 signatures
impl TryInto<b256> for PublicKey {
    fn try_into(self) -> Option<b256> {
        if self.bytes.len() != 32 {
            return None;
        }

        Some(asm(bits: self.bytes().ptr()) {
            bits: b256
        })
    }
}

#[cfg(experimental_partial_eq = false)]
impl core::ops::Eq for PublicKey {
    fn eq(self, other: Self) -> bool {
        if self.bytes.len() != other.bytes.len() {
            return false;
        }

        let mut iter = 0;
        while iter < self.bytes.len() {
            if self.bytes.get(iter).unwrap() != other.bytes.get(iter).unwrap()
            {
                return false;
            }
            iter += 1;
        }

        true
    }
}
#[cfg(experimental_partial_eq = true)]
impl core::ops::PartialEq for PublicKey {
    fn eq(self, other: Self) -> bool {
        if self.bytes.len() != other.bytes.len() {
            return false;
        }

        let mut iter = 0;
        while iter < self.bytes.len() {
            if self.bytes.get(iter).unwrap() != other.bytes.get(iter).unwrap()
            {
                return false;
            }
            iter += 1;
        }

        true
    }
}
#[cfg(experimental_partial_eq = true)]
impl core::ops::Eq for PublicKey {}

impl Hash for PublicKey {
    fn hash(self, ref mut state: Hasher) {
        state.write(self.bytes);
    }
}

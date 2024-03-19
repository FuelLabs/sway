library;

use ::b512::B512;
use ::bytes::Bytes;
use ::alloc::alloc_bytes;
use ::constants::ZERO_B256;
use ::convert::From;
use ::hash::*;

/// Asymmetric 64 byte public key, i.e. verifying key, in uncompressed form.
///
/// # Additional Information
///
/// It should be noted that while Secp256k1 and Secp256r1 uses 64 byte public keys, Ed25519 uses 32 byte public keys.
/// For Ed25519 signatures only the upper 32 bytes are used.
pub struct PublicKey {
    /// The underlying raw `[u8; 64]` data of the public key.
    bits: [u8; 64]
}

impl PublicKey {
    /// Creates a zeroed out instances of a PublicKey signature.
    ///
    /// # Returns
    ///
    /// [PublicKey] - A zero public key.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::PublicKey;
    ///
    /// fn foo() {
    ///     let new_key = PublicKey::new();
    ///     assert(new_key.bits()[0] == 0u8);
    ///     assert(new_key.bits()[63] == 0u8);
    /// }
    /// ```
    pub fn new() -> Self {
        Self {
            bits: [0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8],
        }
    }


    /// Returns the underlying raw `[u8; 64]` data of the public key.
    ///
    /// # Returns
    ///
    /// * [[u8; 64]] - The raw data of the public key.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::PublicKey;
    ///
    /// fn foo() -> {
    ///     let new_key = PublicKey::new();
    ///     assert(new_key.bits()[0] == 0u8);
    /// }
    /// ```
    pub fn bits(self) -> [u8; 64] {
        self.bits
    }
}

impl From<B512> for PublicKey {
    fn from(bits: B512) -> Self {
        Self {
            bits: asm (bits: bits.bits()) { bits: [u8; 64] }
        }
    }
}

impl From<PublicKey> for B512 {
    fn from(public_key: PublicKey) -> Self {
        let b256_1 = asm (bits: public_key.bits()) { bits: b256 };
        let b256_2 = asm (bits: public_key.bits()[32]) { bits: b256 };
        B512::from((b256_1, b256_2))
    }
}

impl From<(b256, b256)> for PublicKey {
    fn from(components: (b256, b256)) -> Self {
        Self {
            bits: asm (components: components) { components: [u8; 64] }
        }
    }
}

impl From<PublicKey> for (b256, b256) {
    fn from(public_key: PublicKey) -> (b256, b256) {
        let b256_1 = asm (bits: public_key.bits()) { bits: b256 };
        let b256_2 = asm (bits: public_key.bits()[32]) { bits: b256 };
        (b256_1, b256_2)
    }
}

// Used for Ed25519 signatures
impl From<b256> for PublicKey {
    fn from(components: b256) -> Self {
        let components: (b256, b256) = (components, ZERO_B256);
        Self {
            bits: asm (components: components) { components: [u8; 64] }
        }
    }
}

// Used for Ed25519 signatures
impl From<PublicKey> for b256 {
    fn from(public_key: PublicKey) -> b256 {
        asm (bits: public_key.bits()) { bits: b256 }
    }
}

impl core::ops::Eq for PublicKey {
    fn eq(self, other: Self) -> bool {
        let self_b256_1 = asm (bits: self.bits) { bits: b256 };
        let self_b256_2 = asm (bits: self.bits[32]) { bits: b256 };
        let other_b256_1 = asm (bits: other.bits) { bits: b256 };
        let other_b256_2 = asm (bits: other.bits[32]) { bits: b256 };

        self_b256_1 == other_b256_1 && self_b256_2 == other_b256_2
    }
}

impl Hash for PublicKey {
    fn hash(self, ref mut state: Hasher) {
        let ptr = alloc_bytes(64); // eight word capacity
        let (word_1, word_2, word_3, word_4, word_5, word_6, word_7, word_8) = asm(r1: self) {
            r1: (u64, u64, u64, u64, u64, u64, u64, u64)
        };

        asm(
            ptr: ptr,
            val_1: word_1,
            val_2: word_2,
            val_3: word_3,
            val_4: word_4,
            val_5: word_5,
            val_6: word_6,
            val_7: word_7,
            val_8: word_8,
        ) {
            sw ptr val_1 i0;
            sw ptr val_2 i1;
            sw ptr val_3 i2;
            sw ptr val_4 i3;
            sw ptr val_5 i4;
            sw ptr val_6 i5;
            sw ptr val_7 i6;
            sw ptr val_8 i7;
        };

        state.write(Bytes::from(raw_slice::from_parts::<u8>(ptr, 64)));
    }
}

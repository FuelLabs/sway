library;

use ::alloc::alloc_bytes;
use ::b512::B512;
use ::bytes::Bytes;
use ::convert::From;
use ::crypto::{cryptographic_error::CryptographicError, message::Message, public_key::PublicKey};
use ::hash::*;
use ::result::Result::{self, *};

/// An ed25519 signature.
pub struct Ed25519 {
    /// The underlying raw `[u8; 64]` data of the signature.
    bits: [u8; 64],
}

impl Ed25519 {
    /// Creates a zeroed out instances of a Ed25519 signature.
    ///
    /// # Returns
    ///
    /// [Ed25519] - A zero ed25519 signature.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::Ed25519;
    ///
    /// fn foo() {
    ///     let new_ed25519 = Ed25519::new();
    ///     assert(new_ed25519.bits()[0] == 0u8);
    ///     assert(new_ed25519.bits()[63] == 0u8);
    /// }
    /// ```
    pub fn new() -> Self {
        Self {
            bits: [
                0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
                0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
                0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
                0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
                0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            ],
        }
    }

    /// Returns the underlying raw `[u8; 64]` data of the signature.
    ///
    /// # Returns
    ///
    /// * [[u8; 64]] - The raw data of the signature.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::Ed25519;
    ///
    /// fn foo() -> {
    ///     let new_ed25519 = Ed25519::new();
    ///     assert(new_ed25519.bits()[0] == 0u8);
    /// }
    /// ```
    pub fn bits(self) -> [u8; 64] {
        self.bits
    }
}

impl Ed25519 {
    /// Verifies that a 32-byte curve25519 public key derived from the private key was used to sign a message.
    /// Returns a `Result` to let the caller choose an error handling strategy.
    ///
    /// # Additional Information
    ///
    /// NOTE: This uses a 32-byte public key. Only the upper 32 bytes of `PublicKey` are used.
    ///
    /// # Arguments
    ///
    /// * `public_key`: [PublicKey] - The public key that signed the message.
    /// * `message`: [Message] - The hashed signed data.
    ///
    /// # Returns
    ///
    /// * [Result<bool, CryptographicError>] - A verified result or an error.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{crypto::{Ed25519, Signature, Message, PublicKey}, constants::ZERO_B256};
    ///
    /// fn foo() {
    ///     let pub_key = 0x314fa58689bbe1da2430517de2d772b384a1c1d2e9cb87e73c6afcf246045b10;
    ///     let msg_hash = 0x1e45523606c96c98ba970ff7cf9511fab8b25e1bcd52ced30b81df1e4a9c4323;
    ///     let hi = 0xf38cef9361894be6c6e0eddec28a663d099d7ddff17c8077a1447d7ecb4e6545;
    ///     let lo = 0xf5084560039486d3462dd65a40c80a74709b2f06d450ffc5dc00345c6b2cdd00;
    ///     let signature: Ed25519Signature = Ed25519Signature::from((hi, lo));
    ///     let message: Message = Message::from(msg_hash);
    ///     // Only the upper 32 bytes are valid for 32-byte curve25519 public keys
    ///     let public_key: PublicKey = PublicKey::from((pub_key, ZERO_B256));
    ///
    ///     // A verified public key with signature
    ///     let verified = signature.verify(pub_key, msg_hash);
    ///     assert(verified.is_ok());
    /// }
    /// ```
    pub fn verify(self, public_key: PublicKey, message: Message) -> Result<(), CryptographicError> {
        let was_error = asm(
            buffer: __addr_of(public_key),
            sig: __addr_of(self),
            hash: __addr_of(message),
        ) {
            ed19 buffer sig hash;
            err
        };

        // check the $err register to see if the `ed19` opcode succeeded
        if was_error == 1 {
            Err(CryptographicError::InvalidSignature)
        } else {
            Ok(())
        }
    }
}

impl From<B512> for Ed25519 {
    fn from(bits: B512) -> Self {
        Self {
            bits: asm(bits: bits.bits()) {
                bits: [u8; 64]
            },
        }
    }
}

impl From<Ed25519> for B512 {
    fn from(signature: Ed25519) -> Self {
        let b256_1 = asm(bits: signature.bits()) {
            bits: b256
        };
        let b256_2 = asm(bits: signature.bits()[32]) {
            bits: b256
        };
        B512::from((b256_1, b256_2))
    }
}

impl From<(b256, b256)> for Ed25519 {
    fn from(components: (b256, b256)) -> Self {
        Self {
            bits: asm(components: components) {
                components: [u8; 64]
            },
        }
    }
}

impl From<Ed25519> for (b256, b256) {
    fn from(signature: Ed25519) -> (b256, b256) {
        let b256_1 = asm(bits: signature.bits()) {
            bits: b256
        };
        let b256_2 = asm(bits: signature.bits()[32]) {
            bits: b256
        };
        (b256_1, b256_2)
    }
}

impl core::ops::Eq for Ed25519 {
    fn eq(self, other: Self) -> bool {
        let self_b256_1 = asm(bits: self.bits) {
            bits: b256
        };
        let self_b256_2 = asm(bits: self.bits[32]) {
            bits: b256
        };
        let other_b256_1 = asm(bits: other.bits) {
            bits: b256
        };
        let other_b256_2 = asm(bits: other.bits[32]) {
            bits: b256
        };

        self_b256_1 == other_b256_1 && self_b256_2 == other_b256_2
    }
}

impl Hash for Ed25519 {
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

#[test]
fn test_ed_verify() {
    use ::assert::assert;
    use ::constants::ZERO_B256;

    let pub_key = 0x314fa58689bbe1da2430517de2d772b384a1c1d2e9cb87e73c6afcf246045b10;
    let msg = ZERO_B256;
    let msg_hash = sha256(msg);
    let hi = 0xf38cef9361894be6c6e0eddec28a663d099d7ddff17c8077a1447d7ecb4e6545;
    let lo = 0xf5084560039486d3462dd65a40c80a74709b2f06d450ffc5dc00345c6b2cdd00;

    let public_key: PublicKey = PublicKey::from(pub_key);
    let signature: Ed25519 = Ed25519::from((hi, lo));
    let message: Message = Message::from(msg_hash);

    // A verified public key with signature 
    let verified = signature.verify(public_key, message);
    assert(verified.is_ok());
}

#[test(should_revert)]
fn test_revert_ed_verify() {
    use ::assert::assert;
    use ::constants::ZERO_B256;

    let pub_key = 0x314fa58689bbe1da2430517de2d772b384a1c1d2e9cb87e73c6afcf246045b10;
    let msg = ZERO_B256;
    let msg_hash = sha256(msg);
    let hi = ZERO_B256;
    let lo = 0xf5084560039486d3462dd65a40c80a74709b2f06d450ffc5dc00345c6b2cdd00;

    let public_key: PublicKey = PublicKey::from(pub_key);
    let signature: Ed25519 = Ed25519::from((hi, lo));
    let message: Message = Message::from(msg_hash);

    let verified = signature.verify(public_key, message);
    assert(verified.is_ok());
}

library;

use ::alloc::alloc_bytes;
use ::b512::B512;
use ::bytes::Bytes;
use ::convert::{From, Into, TryFrom};
use ::crypto::{message::Message, public_key::PublicKey, signature_error::SignatureError};
use ::hash::*;
use ::result::Result::{self, *};
use ::option::Option::{self, *};
use ::ops::*;
use ::codec::*;

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
            bits: [0u8; 64],
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

    /// Verifies that a 32-byte curve25519 public key derived from the private key was used to sign a message.
    /// Returns a `Result` to let the caller choose an error handling strategy.
    ///
    /// # Additional Information
    ///
    /// NOTE: This uses a 32-byte public key.
    ///
    /// # Arguments
    ///
    /// * `public_key`: [PublicKey] - The public key that signed the message.
    /// * `message`: [Message] - The hashed signed data.
    ///
    /// # Returns
    ///
    /// * [Result<bool, SignatureError>] - A verified result or an error.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{crypto::{Ed25519, Signature, Message, PublicKey}, constants::ZERO_B256};
    ///
    /// fn foo() {
    ///     let signature: Ed25519 = Ed25519::from((
    ///         0xf38cef9361894be6c6e0eddec28a663d099d7ddff17c8077a1447d7ecb4e6545,
    ///         0xf5084560039486d3462dd65a40c80a74709b2f06d450ffc5dc00345c6b2cdd00
    ///     ));
    ///     let message: Message = Message::from(0x1e45523606c96c98ba970ff7cf9511fab8b25e1bcd52ced30b81df1e4a9c4323);
    ///     // Only 32 bytes are valid for 32-byte curve25519 public keys
    ///     let public_key: PublicKey = PublicKey::from(0x314fa58689bbe1da2430517de2d772b384a1c1d2e9cb87e73c6afcf246045b10);
    ///
    ///     // A verified public key with signature
    ///     let verified = signature.verify(pub_key, msg_hash);
    ///     assert(verified.is_ok());
    /// }
    /// ```
    pub fn verify(self, public_key: PublicKey, message: Message) -> Result<(), SignatureError> {
        if public_key.bytes().len() != 32 {
            return Err(SignatureError::InvalidPublicKey);
        }

        let was_error = asm(
            buffer: public_key.bytes().ptr(),
            sig: __addr_of(self),
            hash: message.bytes().ptr(),
            len: message.bytes().len(),
        ) {
            ed19 buffer sig hash len;
            err
        };

        // check the $err register to see if the `ed19` opcode succeeded
        if was_error == 1 {
            Err(SignatureError::InvalidSignature)
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
impl From<(b256, b256)> for Ed25519 {
    fn from(components: (b256, b256)) -> Self {
        Self {
            bits: asm(components: components) {
                components: [u8; 64]
            },
        }
    }
}

impl From<[u8; 64]> for Ed25519 {
    fn from(array: [u8; 64]) -> Self {
        Self { bits: array }
    }
}

impl TryFrom<Bytes> for Ed25519 {
    fn try_from(bytes: Bytes) -> Option<Self> {
        if bytes.len() != 64 {
            return None;
        }

        let bits = asm(ptr: bytes.ptr()) {
            ptr: [u8; 64]
        };

        Some(Self { bits })
    }
}

impl Into<B512> for Ed25519 {
    fn into(self) -> B512 {
        let ptr = __addr_of(self.bits);
        let b256_1 = asm(bits: ptr) {
            bits: b256
        };
        let b256_2 = asm(bits: ptr.add_uint_offset(32)) {
            bits: b256
        };
        B512::from((b256_1, b256_2))
    }
}

impl Into<(b256, b256)> for Ed25519 {
    fn into(self) -> (b256, b256) {
        let ptr = __addr_of(self.bits);
        let b256_1 = asm(bits: ptr) {
            bits: b256
        };
        let b256_2 = asm(bits: ptr.add_uint_offset(32)) {
            bits: b256
        };
        (b256_1, b256_2)
    }
}

impl Into<Bytes> for Ed25519 {
    fn into(self) -> Bytes {
        Bytes::from(raw_slice::from_parts::<u8>(__addr_of(self.bits), 64))
    }
}

impl PartialEq for Ed25519 {
    fn eq(self, other: Self) -> bool {
        asm(result, r2: self.bits, r3: other.bits, r4: 64) {
            meq result r2 r3 r4;
            result: bool
        }
    }
}
impl Eq for Ed25519 {}

impl Hash for Ed25519 {
    fn hash(self, ref mut state: Hasher) {
        state.write(Bytes::from(raw_slice::from_parts::<u8>(__addr_of(self.bits), 64)));
    }
}

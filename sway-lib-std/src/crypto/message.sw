library;

use ::b512::B512;
use ::bytes::Bytes;
use ::alloc::alloc_bytes;
use ::convert::From;
use ::hash::*;

/// Normalized (hashed) message authenticated by a signature
pub struct Message {
    /// The underlying raw `[u8; 64]` data of the message.
    bits: [u8; 32]
}

impl Message {
    /// Creates a zeroed out instances of a Message.
    ///
    /// # Returns
    ///
    /// [Message] - A zeroed Message.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::Message;
    ///
    /// fn foo() {
    ///     let new_message = Message::new();
    ///     assert(new_message.bits()[0] == 0u8);
    ///     assert(new_message.bits()[31] == 0u8);
    /// }
    /// ```
    pub fn new() -> Self {
        Self {
            bits: [0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8],
        }
    }

    /// Returns the underlying raw `[u8; 32]` data of the signature.
    ///
    /// # Returns
    ///
    /// * [[u8; 32]] - The raw data of the signature.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::Message;
    ///
    /// fn foo() -> {
    ///     let new_message = Message::new();
    ///     assert(new_message.bits()[0] == 0u8);
    /// }
    /// ```
    pub fn bits(self) -> [u8; 32] {
        self.bits
    }
}

impl From<b256> for Message {
    fn from(bits: b256) -> Self {
        Self {
            bits: asm (bits: bits) { bits: [u8; 32] }
        }
    }
}

impl From<Message> for b256 {
    fn from(message: Message) -> Self {
        asm (bits: message.bits()) { bits: b256 }
    }
}

impl core::ops::Eq for Message {
    fn eq(self, other: Self) -> bool {
        let self_b256 = asm (bits: self.bits) { bits: b256 };
        let other_b256 = asm (bits: other.bits) { bits: b256 };

        self_b256 == other_b256
    }
}

impl Hash for Message {
    fn hash(self, ref mut state: Hasher) {
        let ptr = alloc_bytes(32); // four word capacity
        let (word_1, word_2, word_3, word_4) = asm(r1: self) {
            r1: (u64, u64, u64, u64)
        };

        asm(
            ptr: ptr,
            val_1: word_1,
            val_2: word_2,
            val_3: word_3,
            val_4: word_4,
        ) {
            sw ptr val_1 i0;
            sw ptr val_2 i1;
            sw ptr val_3 i2;
            sw ptr val_4 i3;
        };

        state.write(Bytes::from(raw_slice::from_parts::<u8>(ptr, 32)));
    }
}

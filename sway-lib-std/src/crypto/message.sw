library;

use ::b512::B512;
use ::bytes::Bytes;
use ::alloc::alloc_bytes;
use ::convert::{From, TryFrom, TryInto};
use ::option::Option::{self, *};
use ::hash::*;
use ::ops::*;
use ::codec::*;
use ::debug::*;

/// Normalized (hashed) message authenticated by a signature.
pub struct Message {
    /// The underlying raw data of the message.
    bytes: Bytes,
}

impl Message {
    /// Creates a new instance of a Message.
    ///
    /// # Returns
    ///
    /// [Message] - A new, empty Message.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::Message;
    ///
    /// fn foo() {
    ///     let new_message = Message::new();
    ///     assert(new_message.bytes().len() == 0);
    /// }
    /// ```
    pub fn new() -> Self {
        Self {
            bytes: Bytes::new(),
        }
    }

    /// Returns the underlying raw `Bytes` data of the signature.
    ///
    /// # Returns
    ///
    /// * [Bytes] - The raw data of the signature.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::Message;
    ///
    /// fn foo() -> {
    ///     let new_message = Message::new();
    ///     assert(new_message.bytes().len() == 0);
    /// }
    /// ```
    pub fn bytes(self) -> Bytes {
        self.bytes
    }
}

impl From<b256> for Message {
    fn from(bits: b256) -> Self {
        Self {
            bytes: Bytes::from(bits),
        }
    }
}

impl From<Bytes> for Message {
    fn from(bytes: Bytes) -> Self {
        Self { bytes }
    }
}

impl TryInto<b256> for Message {
    fn try_into(self) -> Option<b256> {
        if self.bytes.len() != 32 {
            return None;
        }

        Some(asm(bits: self.bytes.ptr()) {
            bits: b256
        })
    }
}

impl PartialEq for Message {
    fn eq(self, other: Self) -> bool {
        self.bytes == other.bytes
    }
}
impl Eq for Message {}

impl Hash for Message {
    fn hash(self, ref mut state: Hasher) {
        // We want to hash just the raw bytes of the message,
        // and not the `self.bytes` `Bytes` itself.
        // The fact that the message bytes are stored in a `Bytes` type
        // is just an implementation detail.
        // The hash is computed only over the raw bytes of the message.
        state.write(self.bytes);
    }
}

library;

use ::b512::B512;
use ::bytes::Bytes;
use ::alloc::alloc_bytes;
use ::convert::{From, TryFrom};
use ::option::Option::{self, *};
use ::hash::*;

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

impl TryFrom<Message> for b256 {
    fn try_from(message: Message) -> Option<Self> {
        if message.bytes().len() != 32 {
            return None;
        }

        Some(asm(bits: message.bytes().ptr()) {
            bits: b256
        })
    }
}

impl core::ops::Eq for Message {
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

impl Hash for Message {
    fn hash(self, ref mut state: Hasher) {
        state.write(self.bytes);
    }
}

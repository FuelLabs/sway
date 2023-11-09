use sway_ir::{Type, Context};

/// Provides information about type sizes, raw and aligned to word boundaries.
#[derive(Clone, Debug)]
pub(crate) struct TypeSize {
    size_in_bytes: u64,
}

impl TypeSize {
    /// Creates the type size for the [Type] `ty`.
    pub(crate) fn for_type(ty: &Type, context: &Context) -> Self {
        Self {
            size_in_bytes: ty.size_in_bytes(context),
        }
    }

    /// Returns the actual (unaligned) size of the type in bytes.
    pub(crate) fn in_bytes(&self) -> u64 {
       self.size_in_bytes 
    }

    /// Returns the size of the type in bytes, aligned to word boundary.
    pub(crate) fn in_bytes_aligned(&self) -> u64 {
        (self.size_in_bytes + 7) - ((self.size_in_bytes + 7) % 8)
    }

    /// Returns the size of the type in words.
    pub(crate) fn in_words(&self) -> u64 {
        (self.size_in_bytes + 7) / 8
    }
}
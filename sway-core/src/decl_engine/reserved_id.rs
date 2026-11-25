use std::fmt;

use crate::decl_engine::DeclId;

/// A capability token proving the ownership of a declaration slot reserved
/// in the [DeclEngine](super::DeclEngine) via
/// [DeclEngineReserve::reserve](super::DeclEngineReserve::reserve).
///
/// The token represents the construction phase of the declaration stored
/// in the reserved slot and is the only way to mutate it:
/// - [DeclEngineReserve::fill](super::DeclEngineReserve::fill) writes the
///   declaration content, and can be called multiple times, e.g., once for
///   each type checking pass that gradually completes the declaration.
/// - [DeclEngineReserve::freeze](super::DeclEngineReserve::freeze) consumes
///   the token and ends the construction phase.
///
/// Since the token is neither [Clone] nor [Copy], after the freeze, the
/// declaration behind the reserved [DeclId] is guaranteed to be immutable,
/// at compile time.
///
/// The [DeclId] of the reserved slot is available via [Self::id] already
/// during the construction phase. This allows the id to be, e.g., embedded
/// into other declarations before the declaration behind it is filled.
/// This is the primary motivation for the reservation: breaking the
/// circularity between, e.g., an impl block declaration
/// and its items, which point at each other.
pub struct ReservedDeclId<T> {
    id: DeclId<T>,
}

impl<T> ReservedDeclId<T> {
    /// Creates a new token for the reserved slot behind the `id`.
    ///
    /// Deliberately restricted to the [crate::decl_engine] module - only
    /// the [DeclEngine](super::DeclEngine) can create tokens, via
    /// [DeclEngineReserve::reserve](super::DeclEngineReserve::reserve).
    pub(super) fn new(id: DeclId<T>) -> Self {
        Self { id }
    }

    /// The [DeclId] of the reserved slot.
    ///
    /// The id can be freely copied and embedded into other declarations
    /// during the construction phase, but the declaration behind it must
    /// not be read before the first
    /// [DeclEngineReserve::fill](super::DeclEngineReserve::fill).
    pub fn id(&self) -> DeclId<T> {
        self.id
    }
}

impl<T> fmt::Debug for ReservedDeclId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ReservedDeclId").field(&self.id).finish()
    }
}

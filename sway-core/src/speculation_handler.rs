//! Support for speculative compilation. Compiling within a private error
//! [Handler] whose errors may be deliberately discarded.

use std::borrow::Borrow;

use sway_error::handler::Handler;

use crate::Engines;

/// Extends the [Handler] with support for _speculative compilation_:
/// compiling within a private, speculation [Handler] whose errors may be
/// deliberately discarded by the caller.
///
/// E.g., the trait auto-implementations are generated for every struct and
/// enum, and legitimately fail to type check for the types that do not support
/// the auto-implemented trait. Such a failure only means that the type does not
/// get the implementation, and its errors are deliberately discarded.
///
/// Discarding errors of a failed compilation requires cleaning up the residuals
/// that the failure leaves behind in the [Engines]. In a regular compilation,
/// residuals left behind by failures are irrelevant, because the emitted errors
/// fail the whole compilation. A speculation whose errors are discarded,
/// however, might have a need to clean up after itself.
///
/// Currently, only the [DeclEngine][crate::decl_engine::DeclEngine]
/// needs such a clean-up. A failed speculation leaves decl slots open (see
/// [DeclEngineReserve][crate::decl_engine::DeclEngineReserve]), which would
/// break the invariant that a successfully finished type checking of a program
/// leaves no open decl slots.
///
/// Use this API exactly where the private handler wraps a compilation that can
/// alter engines, and whose errors may be discarded.
///
/// Note that a speculation currently **scopes only the decl slots side effects of the
/// compilation it runs**. **Its other side effects remain**, e.g., the type
/// unifications in the [TypeEngine][crate::TypeEngine].
pub trait SpeculationHandler: Borrow<Handler> {
    /// Runs the speculative compilation `f` within a fresh speculation
    /// [Handler] and returns `f`'s result, together with that handler.
    /// The caller decides what to do with the errors collected in the returned
    /// handler, e.g., to discard them, or to conditionally integrate them
    /// into an enclosing handler, perhaps only partially.
    ///
    /// This is the speculation primitive. For the common cases, prefer the
    /// [SpeculationHandler::discard] and [SpeculationHandler::discard_if]
    /// convenience methods.
    fn speculate<T>(engines: &Engines, f: impl FnOnce(&Handler) -> T) -> (T, Handler);

    /// Runs the speculative compilation `f` within a fresh speculation
    /// [Handler], unconditionally discards the errors collected during the
    /// speculation, and returns `f`'s result.
    fn discard<T>(&self, engines: &Engines, f: impl FnOnce(&Handler) -> T) -> T {
        let (result, _speculation_handler) = Self::speculate(engines, f);
        result
    }

    /// Runs the speculative compilation `f` within a fresh speculation
    /// [Handler] and returns `f`'s result. The errors collected during the
    /// speculation are discarded if `discard` returns true, and appended to
    /// `self` otherwise.
    fn discard_if<T>(
        &self,
        engines: &Engines,
        f: impl FnOnce(&Handler) -> T,
        discard: impl FnOnce(&Handler, &T) -> bool,
    ) -> T {
        let (result, speculation_handler) = Self::speculate(engines, f);
        if !discard(&speculation_handler, &result) {
            self.borrow().append(speculation_handler);
        }
        result
    }
}

impl SpeculationHandler for Handler {
    fn speculate<T>(engines: &Engines, f: impl FnOnce(&Handler) -> T) -> (T, Handler) {
        let speculation_handler = Handler::default();
        // Currently, only the `DeclEngine` needs a clean-up of the residuals
        // of a speculation. The decl slots that a failed speculation leaves
        // open get forgotten.
        let result = engines
            .de()
            .forget_open_slots_during(|| f(&speculation_handler));
        (result, speculation_handler)
    }
}

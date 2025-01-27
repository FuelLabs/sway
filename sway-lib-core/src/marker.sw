//! Marker traits that represent certain properties of types.
//!
//! Sway types can be classified in various ways according to their intrinsic properties.
//! These classifications are represented as marker traits. Marker traits are implemented
//! by the compiler and cannot be explicitly implemented in code.

library;


/// A marker for error types.
///
/// Error types are types whose instances can be arguments to the `panic` instruction.
///
/// [Error] is automatically implemented for:
/// - string slices,
/// - and enums annotated with the `#[error_type]` attribute.
#[cfg(experimental_error_type = true)]
pub trait Error { }

/// A marker for enum types.
#[cfg(experimental_error_type = true)]
pub trait Enum { }

// Marker traits cannot be explicitly implement in code, except in this module.
// If a marker trait needs to be implemented for a built-in type, those implementation
// will be provided here.

#[cfg(experimental_error_type = true)]
impl Error for str { }

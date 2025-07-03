library;

use ::codec::*;
use ::debug::*;

/// The error type used when a cryptographic signature function fails.
pub enum SignatureError {
    /// The error variant used when the recover fails.
    UnrecoverablePublicKey: (),
    /// The error variant used when the public key is of the wrong type.
    InvalidPublicKey: (),
    /// The error variant used when signature verification fails.
    InvalidSignature: (),
    /// The error variant used when an invalid operation was performed.
    InvalidOperation: (),
}

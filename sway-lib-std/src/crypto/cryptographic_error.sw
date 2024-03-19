library;

/// The error type used when a cryptographic function fails.
pub enum CryptographicError {
    /// The error variant used when the recover fails.
    UnrecoverablePublicKey: (),
    /// The error variant used when signature verification fails.
    InvalidSignature: (),
    /// The error varient used when an invalid operation was performed.
    InvalidOperation: (),
}


library;

/// The error type used when performing elliptic curve operations for Zero Knowledge cryptography.
pub enum ZKError {
    /// An unsupported curve was used.
    UnsupportedCurve: (),
    /// The elliptic curve point used was invalid.
    InvalidEllipticCurvePoint: (),
    /// The elliptice curve scalar used was invalid.
    InvalidEllipticCurveScalar: (),
}

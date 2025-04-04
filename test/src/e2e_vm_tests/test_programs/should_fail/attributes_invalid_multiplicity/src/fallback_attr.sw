library;

#[fallback]
pub fn ok() { } // Actually semantically also not ok, but the compilation will not reach that phase.

#[fallback]
#[fallback]
#[fallback]
pub fn not_ok() { }
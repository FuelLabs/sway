library;

#[fallback]
pub fn ok() { } // Actually semantically also not ok, but the compilation will not reach that phase.

#[fallback(invalid)] // Should be no invalid arg error or warning here.
pub fn also_ok() { }
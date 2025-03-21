library;

#[fallback]
pub fn ok_1() { } // Actually semantically also not ok, but the compilation will not reach that phase.

#[fallback()]
pub fn ok_2() { } // Actually semantically also not ok, but the compilation will not reach that phase.

#[fallback(invalid)]
pub fn not_ok_1() { }
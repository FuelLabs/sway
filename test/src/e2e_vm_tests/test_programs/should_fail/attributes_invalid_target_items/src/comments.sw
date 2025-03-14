/// Invalid outer comment #1.
/// Invalid outer comment #2.

/// Invalid outer comment #3.

/// Invalid outer comment #4.
/// Invalid outer comment #5.
library;

//! Invalid inner comment A #1.
//! Invalid inner comment A #2.
//! Invalid inner comment A #3.
#[test]
//! Invalid inner comment B #1.

//! Invalid inner comment B #2.
#[payable(invalid)]
#[allow(deprecated)]
//! Invalid inner comment C.
#[allow(dead_code)]
fn f() { }

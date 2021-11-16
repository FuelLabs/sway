library assertions;
use ::ops::*;

/// a failing assertion should:
/// - panic in a script
/// - revert in a contract

/// @review trait bounds here will be helpful, ie: where T: Ord
/// switch this to return a Result when they land.
pub fn assert_eq<T>(a: T, b: T) -> bool {
    if a == b {
        true
    } else {
        false()
    }
}

pub fn assert_neq<T>(a: T, b: T) -> bool {
    if a != b {
        true
    } else {
        false
    }
}

pub fn require(a: bool) -> () {
    if a {
        return ();
    } else {
        revert(0);
    }
}

// would be nice to have the ability to make generic assertions about an expression
// pub fn assert()
// so I could do:
// assert(a == b)
// assert(a != b)
// asssert(a >= b), etc...
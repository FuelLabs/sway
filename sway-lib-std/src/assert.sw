library assert;

use ::revert::revert;
use ::context::call_frames;

pub struct CustomError<T> {
    Id: ContractId, // use to log the current contract id
    RevertValue: u64, // value to pass to `revert`
    AdditionalData: T // user can log out generic data to keep this flexible
}


/// Assert that a value is true
pub fn assert(a: bool) {
    if !a {
        revert(0);
    } else {
        ()
    }
}

/// A wrapper for `assert` that allows passing a custom revert value `v` if condition `c` is not true.
pub fn require<T>(c: bool, v: T) {
    if !c {
        let mut size = 32;
        if is_reference_type::<T>() {
            size = size_of::<T>();
        } else {
            let this = contract_id();
            asm(r1: v, r2: size, r3: this) {
                logd r3 zero r1 r2;
            };
        }
        revert(v)
    } else {
        ()
    }
}

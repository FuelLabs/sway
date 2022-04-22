script;
use std::chain::*;

fn main() {
    let x = if true {
        42u64
    } else {
        // this should be a type error even though everything aborts
        if true {
            return 42;
        } else {
            return true;
        };
        panic(0)
    };
}

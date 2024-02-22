script;

// This file tests the reported errors and warnings in various instances when a 'break'
// occurs in a non-statement position. This is allowed, but will often result in
// unreachable code or similar warning situations.

// Matching on `break` is unimplemented.

fn in_match_scrutinee_break() -> u64 {
    let mut i = 42;
    while i < 52 {
        match break {
            _ => return 5411,
        }
        i = i + 1;
    }

    i
}

// Matching on `continue` is unimplemented.

fn in_match_scrutinee_continue() -> u64 {
    let mut i = 32;
    while i < 43 {
        i = i + 1;
        match continue {
            _ => return 5411,
        }
    }

    i
}

fn main() {
    assert(42 == in_match_scrutinee_break());
    assert(42 == in_match_scrutinee_continue());
}

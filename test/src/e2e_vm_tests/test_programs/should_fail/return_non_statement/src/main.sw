script;

// The grammar treats 'return' as an expression rather than as a statement, but 'return' is only
// allowed to occur in statement positions.
// This file tests the reported error in various instances when a 'return' occurs in a
// non-statement position.

fn main() -> u64 {
    let a = [return 0u64];

    let a = (return 0u64, return 1u64);

    if return 0u64 {
        0
    };
    
    let a = return 0u64;
    
    let _ = match return 0u64 {
        _ => 0,
    };
    
    0
}
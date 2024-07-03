script;

enum Enum {
    A: (u64),
    B: (u64),
}

struct Struct {
    x: u64,
    y: u64,
    z: u64
}
 
// For testing side effects.
fn inc_i(ref mut i: u64) -> Struct {
    i = i + 11;
 
    Struct { x: 21, y: 21, z: 1 }
}

fn return_match_on_str_slice(param: str) -> u64 {
    match param {
        "get_a" => { 1u64 },
        "get_a_b" => { 2u64 },
        "get_b" => { 3u64 },
        _ => { 1000u64 },
    }
}

fn main() {
    assert(return_match_on_str_slice("") == 1000);
}

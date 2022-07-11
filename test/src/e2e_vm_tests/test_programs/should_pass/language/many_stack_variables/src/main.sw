script;

use std::constants::ZERO_B256;

struct Z {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
}

fn main() -> u64 {
    // Chosen names force these variables to show up last in the list of locals in IR so they will
    // be allocated last and require the highest offset to be computed

    // Test get_ptr large offset
    let zzz = Z {
        a: 1,
        b: 2,
        c: 3,
        d: 4,
    };

    // Test LW/SW with large offsets
    let z1 = 5;
    let z2 = 6;

    // Add enough stack variables to reach > 4096 words
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();
    foo();

    return zzz.a + zzz.b + zzz.c + zzz.d + z1 + z2 // get 16
}

fn foo() {
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
    let c = 0;
}

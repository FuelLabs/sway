script;

// Having > 256K NOPs from the BLOBs will force some relocation to keep control flow under the 1MB
// boundary.  But only one of these need to be moved to get it back under, and it should be the
// largest one.

fn main() -> u64 {
    asm() {
        blob i50000;
    }
    if t() {
        if f() {
            asm() {
                blob i51000;
            }
            111
        } else {
            // This one.
            asm() {
                blob i60000;
            }
            222
        }
    } else {
        if f() {
            asm() {
                blob i52000;
            }
            333
        } else {
            asm() {
                blob i53000;
            }
            444
        }
    }
}

fn f() -> bool {
    asm() {
        zero: bool
    }
}

fn t() -> bool {
    asm() {
        one: bool
    }
}

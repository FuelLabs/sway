library auth;

// this can be a generic option when options land
enum Caller {
    Some: b256,
    None: (),
}

/// Returns `true` if the caller is external.
pub fn caller_is_external() -> bool {
    asm(r1) {
        gm r1 i1;
        r1: bool
    }
}

pub fn caller() -> Caller {
    // if parent is not external
    if !caller_is_external() {
        // get the caller
        Caller::Some(asm(r1) {
            gmr1i2;
            r1: b256
        })
    } else {
        Caller::None
    }
}

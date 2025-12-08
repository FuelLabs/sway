script;

enum E {
    A: (),
    B: (),
    C: (),
}

#[inline(never)]
fn enum_varitants_unit(e: E) {
    __log(e);
}

fn main() -> E {
    enum_varitants_unit(E::A);
    // Expected output is only 8 bytes because all the variants are unit types 
    //
    //  0000000000000002  # E.tag

    E::C
}

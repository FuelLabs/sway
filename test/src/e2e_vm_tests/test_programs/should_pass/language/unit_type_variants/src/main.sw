script;

enum E {
    A: (),
    B: (),
    C: (),
}

fn main() -> E {
    // Expected output is only 8 bytes because all the variants are unit types 
    //
    //  0000000000000002  # E.tag

    E::C
}

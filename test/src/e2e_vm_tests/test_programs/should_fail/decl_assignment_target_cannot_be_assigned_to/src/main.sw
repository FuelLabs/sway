script;

abi Abi { }

struct S {
    x: u8,
}

trait Trait {}

fn function() { }

enum E {
    A: (),
}

type Type = u64;

pub fn play() {
    Abi = 0;

    Abi.x = 0;

    S = 0;

    S.x = 0;

    Trait = 0;

    Trait.x = 0;

    function = 0;

    function.x = 0;

    E = 0;

    E.x = 0;

    Type = 0;

    Type.x = 0;
}

script;

enum E {
    Eins: F,
}

enum F {
    Zwei: G,
}

enum G {
    Drei: E,
}

enum H {
    Vier: I,
}

enum I {
    Funf: H,
}

struct S {
    one: T,
}

struct T {
    two: S,
}

struct X {
    three: Y,
}

enum Y {
    four: Z,
}

struct Z {
    five: X,
}

fn main() {

}

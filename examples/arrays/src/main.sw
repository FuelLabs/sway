script;

struct Foo {
    f1: u32,
    f2: b256,
}

fn main() {
    // Array of integers with type ascription
    let array_of_integers: [u8; 5] = [1, 2, 3, 4, 5];

    // Array of strings
    let array_of_strings = ["Bob", "Jan", "Ron"];

    // Array of structs
    let array_of_structs: [Foo; 2] = [
        Foo {
            f1: 11,
            f2: 0x1111111111111111111111111111111111111111111111111111111111111111,
        },
        Foo {
            f1: 22,
            f2: 0x2222222222222222222222222222222222222222222222222222222222222222,
        },
    ];

    // Accessing an element of an array
    let mut array_of_bools: [bool; 2] = [true, false];
    assert(array_of_bools[0]);

    // Mutating the element of an array
    array_of_bools[1] = true;
    assert(array_of_bools[1]);
}

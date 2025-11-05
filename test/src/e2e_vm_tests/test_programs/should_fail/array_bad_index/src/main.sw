// This test proves that https://github.com/FuelLabs/sway/issues/5922 is fixed.
library;

struct S {
    x: u64,
}

enum E {
    X: u64
}

pub fn main() { 
    let mut array = [1, 2, 3];

    array[0u8] = 0;

    array[0u16] = 0;

    array[0u32] = 0;

    // Enough vertical space, so that the below line
    // does not appear in the output of the above error.

    array[0u64] = 0;


    array[true] = 0;

    array[()] = 0;

    array["test"] = 0;

    array[S { x: 0 }] = 0;
    
    array[E::X(0)] = 0;

    poke(array[0u8]);
    poke(array[0u16]);
    poke(array[0u32]);



    poke(array[0u64]);



    poke(array[true]);
    poke(array[()]);
    poke(array["test"]);
    poke(array[S { x: 0 }]);
    poke(array[E::X(0)]);

    poke(S { x: 0}.x);
}

#[inline(never)]
fn poke<T>(_x: T) { }
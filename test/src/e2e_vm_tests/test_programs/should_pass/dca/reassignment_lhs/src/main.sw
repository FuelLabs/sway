script;

const CONST: u64 = 0;

fn function_1() -> u64 {
    0
}

struct S1 {}

impl S1 {
    fn method(self) -> u64 {
        0
    }
}

fn function_2() -> u64 {
    0
}

struct S2 {}

impl S2 {
    fn method(self) -> u64 {
        0
    }
}

fn main() {
    // This test proves that https://github.com/FuelLabs/sway/issues/5920 is fixed.
    let mut array = [1, 2, 3];

    let i = 0;
    array[i] = 0;

    array[CONST] = 0;

    array[function_1()] = 0;

    array[S1 {}.method()] = 0;

    // This proves that LHS is properly analyzed in DCA also for dereferencing.
    let mut x = 0;

    *&mut x = 0;

    *&mut function_2() = 0;

    *&mut S2 {}.method() = 0;
}
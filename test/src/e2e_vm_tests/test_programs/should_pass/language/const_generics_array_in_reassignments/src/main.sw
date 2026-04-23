// This tests prove that https://github.com/FuelLabs/sway/issues/7600 is fixed.
library;

trait CopySelf<D> {
    fn copy_self(self, default: D) -> Self;
}

impl<T, const N: u64> CopySelf<T> for [T; N]
where
    T: AbiEncode + PartialEq,
{
    fn copy_self(self, default: T) -> Self {
        let mut array = [default; N];
        let mut i = 0;
        while i < N {
            array[i] = self[i];
            assert_eq(array[i], self[i]);
            i += 1;
        }
        array
    }
}

trait DefaultSelf<D> {
    fn default_self(ref mut self, default: D);
}

impl<T, const N: u64> DefaultSelf<T> for [T; N]
where
    T: AbiEncode + PartialEq,
{
    fn default_self(ref mut self, default: T) {
        let mut i = 0;
        while i < N {
            self[i] = default;
            // TODO: Uncomment this `assert_eq` once https://github.com/FuelLabs/sway/issues/7602 is fixed.
            // assert_eq(self[i], default);
            i += 1;
        }
    }
}

trait ConstructSelf<D> {
    fn construct_self(default: D) -> Self;
}

impl<T, const N: u64> ConstructSelf<T> for [T; N]
where
    T: AbiEncode + PartialEq,
{
    fn construct_self(default: T) -> Self {
        let mut array = [default; N];
        let mut i = 0;
        while i < N {
            array[i] = default;
            assert_eq(array[i], default);
            i += 1;
        }
        array
    }
}

fn simple_const_generic_array_reassignment<const N: u64>() -> [u8; N] {
    let mut array = [0u8; N];
    array[0] = 42;
    assert_eq(array[0], 42);
    array
}

fn simple_const_generic_nested_array_reassignment<const A: u64, const B: u64>() -> [[u64; A]; B] {
    let mut array = [[0u64; A]; B];
    let mut a = 0;
    while a < A {
        let mut b = 0;
        while b < B {
            array[a][b] = a + b;
            assert_eq(array[a][b], a + b);
            b += 1;
        }
        a += 1;
    }
    array
}

type ArrayU8 = [u8; 3];

#[test]
fn main() {
    let mut array = [1u8, 2, 3];
    assert_eq(array, array.copy_self(0));
    array.default_self(42);
    // TODO: Uncomment this `assert_eq` once https://github.com/FuelLabs/sway/issues/7603 is fixed.
    // assert_eq([42; 3], array);
    assert_eq([42u8; 3], array);

    let array = [42u16];
    assert_eq(array, array.copy_self(0));

    let array = [42u32; 3];
    assert_eq(array, array.copy_self(0));

    let array_of_array = [array; 3];
    assert_eq(array_of_array, array_of_array.copy_self(array));

    let array = simple_const_generic_array_reassignment::<1>();
    // TODO: Uncomment this `assert_eq` once https://github.com/FuelLabs/sway/issues/7603 is fixed.
    // assert_eq([42], array);
    assert_eq([42u8], array);

    let array = simple_const_generic_array_reassignment::<3>();
    // TODO: Uncomment this `assert_eq` once https://github.com/FuelLabs/sway/issues/7603 is fixed.
    // assert_eq([42, 0, 0], array);
    assert_eq([42u8, 0, 0], array);

    let array = simple_const_generic_nested_array_reassignment::<0, 0>();
    assert_eq([], array);

    let array = simple_const_generic_nested_array_reassignment::<0, 1>();
    assert_eq([[]], array);

    let array = simple_const_generic_nested_array_reassignment::<1, 1>();
    assert_eq([[0]], array);

    let array = simple_const_generic_nested_array_reassignment::<2, 2>();
    assert_eq([[0, 1], [1, 2]], array);

    // TODO: Uncomment this code once https://github.com/FuelLabs/sway/issues/7604 is fixed.
    // let array = ArrayU842::construct_self(42u8);
    // assert_eq([42u8, 42, 42], array);
}

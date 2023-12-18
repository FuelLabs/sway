script;

use core::ops::Eq;

struct Struct {
    x: u64,
}

impl core::ops::Eq for Struct {
    fn eq(self, other: Self) -> bool {
        self.x == other.x
    }
}

impl core::ops::Eq for [Struct; 3] {
    fn eq(self, other: Self) -> bool {
        self[0] == other[0] && self[1] == other[1] && self[2] == other[2]
    }
}

// TODO-IG: Add tests for other expressions that can be referenced and errors for those that cannot.

#[inline(always)]
fn if_expr<T>(input: u64, left: T, right: T) where T: Eq {
    let x = if input > 42 {
        left
    } else {
        right
    };

    let r_x = &x;
    let r_val = &if input > 42 {
        left
    } else {
        right
    };

    let r_x_ptr = asm(r: r_x) { r: raw_ptr };
    let r_val_ptr = asm(r: r_val) { r: raw_ptr };

    assert(r_x_ptr != r_val_ptr);

    let r_x_ptr_val = r_x_ptr.read::<T>();
    let r_x_val_val = r_val_ptr.read::<T>();

    assert(r_x_ptr_val == x);
    assert(r_x_val_val == x);
}

#[inline(never)]
fn if_expr_not_inlined<T>(input: u64, left: T, right: T) where T: Eq {
    if_expr(input, left, right)
}

#[inline(never)]
fn test_all_inlined(input: u64) {
    if_expr(input, 123, 321);
    if_expr(input, Struct { x: 123 }, Struct { x: 321 });
    if_expr(input, [Struct { x: 123 }, Struct { x: 123 }, Struct { x: 123 }], [Struct { x: 321 }, Struct { x: 321 }, Struct { x: 321 }]);
}

#[inline(never)]
fn test_not_inlined(input: u64) {
    if_expr_not_inlined(input, 123, 321);
    if_expr_not_inlined(input, Struct { x: 123 }, Struct { x: 321 });
    if_expr_not_inlined(input, [Struct { x: 123 }, Struct { x: 123 }, Struct { x: 123 }], [Struct { x: 321 }, Struct { x: 321 }, Struct { x: 321 }]);
}

fn main() -> u64 {
    test_all_inlined(42 - 1);
    test_all_inlined(42 + 1);

    test_not_inlined(42 - 1);
    test_not_inlined(42 + 1);

    42
}

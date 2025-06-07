script;

struct Struct {
    x: u64,
}

impl PartialEq for Struct {
    fn eq(self, other: Self) -> bool {
        self.x == other.x
    }
}
impl Eq for Struct {}

impl PartialEq for [Struct; 3] {
    fn eq(self, other: Self) -> bool {
        self[0] == other[0] && self[1] == other[1] && self[2] == other[2]
    }
}
impl Eq for [Struct; 3] {}

// TODO: (REFERENCES) Add tests for other expressions.

#[inline(always)]
fn if_expr<T>(input: u64, left: T, right: T)
where
    T: AbiEncode + Eq,
{
    let mut x = if input > 42 { left } else { right };

    let r_x = &x;

    let r_val = &if input > 42 { left } else { right };

    let r_mut_x = &mut x;

    let r_mut_val = &mut if input > 42 { left } else { right };

    assert_references(r_x, r_val, r_mut_x, r_mut_val, x);

    if *r_mut_x == left {
        assert_eq(x, left);
        *r_mut_x = right;
        assert_eq(x, right);
    } else {
        assert_eq(x, right);
        *r_mut_x = left;
        assert_eq(x, left);
    }

    if *r_mut_val == left {
        let current_x = x;
        *r_mut_val = right;
        assert_eq(x, current_x);
        assert_eq(*r_mut_val, right);
    } else {
        let current_x = x;
        *r_mut_val = left;
        assert_eq(x, current_x);
        assert_eq(*r_mut_val, left);
    }
}

fn assert_references<T>(
    r_x: &T,
    r_val: &T,
    r_mut_x: &mut T,
    r_mut_val: &mut T,
    x: T,
)
where
    T: Eq,
{
    let r_x_ptr = asm(r: r_x) {
        r: raw_ptr
    };
    let r_mut_x_ptr = asm(r: r_mut_x) {
        r: raw_ptr
    };
    let r_val_ptr = asm(r: r_val) {
        r: raw_ptr
    };
    let r_mut_val_ptr = asm(r: r_mut_val) {
        r: raw_ptr
    };

    assert(r_x_ptr == r_mut_x_ptr);
    assert(r_val_ptr != r_mut_val_ptr);
    assert(r_x_ptr != r_val_ptr);
    assert(r_mut_x_ptr != r_mut_val_ptr);

    let r_x_ptr_val = r_x_ptr.read::<T>();
    let r_mut_x_ptr_val = r_mut_x_ptr.read::<T>();
    let r_x_val_val = r_val_ptr.read::<T>();
    let r_mut_x_val_val = r_mut_val_ptr.read::<T>();

    assert(r_x_ptr_val == x);
    assert(r_mut_x_ptr_val == x);
    assert(r_x_val_val == x);
    assert(r_mut_x_val_val == x);

    assert(*r_x == x);
    assert(*r_mut_x == x);
    assert(*r_val == x);
    assert(*r_mut_val == x);
}

#[inline(never)]
fn if_expr_not_inlined<T>(input: u64, left: T, right: T)
where
    T: AbiEncode + Eq,
{
    if_expr(input, left, right)
}

#[inline(never)]
fn test_all_inlined(input: u64) {
    if_expr(input, 123, 321);
    if_expr(input, Struct { x: 123 }, Struct { x: 321 });
    if_expr(
        input,
        [Struct { x: 123 }, Struct { x: 123 }, Struct { x: 123 }],
        [Struct { x: 321 }, Struct { x: 321 }, Struct { x: 321 }],
    );
}

#[inline(never)]
fn test_not_inlined(input: u64) {
    if_expr_not_inlined(input, 123, 321);
    if_expr_not_inlined(input, Struct { x: 123 }, Struct { x: 321 });
    if_expr_not_inlined(
        input,
        [Struct { x: 123 }, Struct { x: 123 }, Struct { x: 123 }],
        [Struct { x: 321 }, Struct { x: 321 }, Struct { x: 321 }],
    );
}

fn main() -> u64 {
    test_all_inlined(42 - 1);
    test_all_inlined(42 + 1);

    test_not_inlined(42 - 1);
    test_not_inlined(42 + 1);

    42
}

script;

pub trait New {
    fn new() -> Self;
}

impl New for () {
    fn new() -> Self {
        ()
    }
}

impl New for [u64; 0] {
    fn new() -> Self {
        []
    }
}

impl PartialEq for [u64; 0] {
    fn eq(self, other: Self) -> bool {
        true
    }
}
impl Eq for [u64; 0] {}

struct EmptyStruct {}

impl New for EmptyStruct {
    fn new() -> Self {
        EmptyStruct {}
    }
}

impl PartialEq for EmptyStruct {
    fn eq(self, other: Self) -> bool {
        true
    }
}
impl Eq for EmptyStruct {}

#[inline(always)]
fn reference_zero_sized_local_var_and_value<T>()
where
    T: New + Eq,
{
    assert(__size_of::<T>() == 0);

    let mut x1 = T::new();
    let x2 = T::new();

    let x_ptr1 = asm(r: &x1) {
        r: raw_ptr
    };
    let res1 = x_ptr1.read::<T>();

    let x_ptr2 = __addr_of(x2);
    let res2 = x_ptr2.read::<T>();
    assert(res1 == res2);
}

#[inline(never)]
fn reference_zero_sized_local_var_and_value_not_inlined<T>()
where
    T: New + Eq,
{
    reference_zero_sized_local_var_and_value::<T>()
}

fn main() -> u64 {
    reference_zero_sized_local_var_and_value_not_inlined::<()>();
    reference_zero_sized_local_var_and_value_not_inlined::<EmptyStruct>();
    reference_zero_sized_local_var_and_value_not_inlined::<[u64; 0]>();

    reference_zero_sized_local_var_and_value::<()>();
    reference_zero_sized_local_var_and_value::<EmptyStruct>();
    reference_zero_sized_local_var_and_value::<[u64; 0]>();

    42
}

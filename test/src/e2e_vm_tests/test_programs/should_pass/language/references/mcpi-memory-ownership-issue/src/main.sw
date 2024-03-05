script;

pub trait New {
    fn new() -> Self;
}

impl New for [u64;0] {
    fn new() -> Self {
        []
    }
}

struct EmptyStruct { }

impl New for EmptyStruct {
    fn new() -> Self {
        EmptyStruct { }
    }
}

#[inline(always)]
fn reference_zero_sized_local_var_and_value<T>()
    where T: New
{
    assert(__size_of::<T>() == 0);

    let mut x = T::new();
    // let x = T::new(); // OK if the variable is not mutable.

    // Both examples fail. Taking the address from the reference, or using `__addr_of()`.
    // let x_ptr = asm(r: &x) { r: raw_ptr };
    // let _ = x_ptr.read::<T>();

    let x_ptr = __addr_of(x);
    let _ = x_ptr.read::<T>(); // Fails here with:
                               // Reason: PanicInstruction
                               // { reason: MemoryOwnership, instruction: MCPI { dst_addr: 0x11, src_addr: 0x3c, len: 0 } (bytes: 60 47 c0 00) }
}

#[inline(never)]
fn reference_zero_sized_local_var_and_value_not_inlined<T>()
    where T: New
{
    reference_zero_sized_local_var_and_value::<T>()
}

fn main() -> u64 {
    // OK for empty struct.
    // reference_zero_sized_local_var_and_value_not_inlined::<EmptyStruct>();

    // Fails for empty array when called over `_not_inlined`.
    reference_zero_sized_local_var_and_value_not_inlined::<[u64;0]>();

    // OK if called directly and being inlined.
    // reference_zero_sized_local_var_and_value::<[u64;0]>();

    42
}

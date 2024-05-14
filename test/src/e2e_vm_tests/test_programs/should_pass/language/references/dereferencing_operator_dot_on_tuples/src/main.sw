script;

mod impls;
use impls::*;
use core::ops::Eq;

impl<T> Eq for & &T
    where T: TestInstance + Eq
{
    fn eq(self, other: Self) -> bool {
        **self == **other
    }
}

impl<T> Eq for & & &T
    where T: TestInstance + Eq
{
    fn eq(self, other: Self) -> bool {
        ***self == ***other
    }
}

#[inline(always)]
fn dereference_tuple<T>()
    where T: TestInstance + Eq
{
    let mut x = (T::new(), T::different());

    let r_x = &x;
    let r_r_x = &r_x;
    let r_r_r_x = &r_r_x;

    let mut r_mut_x = &mut x;
    let mut r_mut_r_mut_x = &mut r_mut_x;
    let r_mut_r_mut_r_mut_x = &mut r_mut_r_mut_x;

    assert((*r_x).0 == T::new());
    assert((*r_x).0 == r_x.0);
    assert((*r_x).1 == T::different());
    assert((*r_x).1 == r_x.1);

    assert(r_x.0 == r_mut_x.0);
    assert(r_x.1 == r_mut_x.1);

    assert((**r_r_x).0 == T::new());
    assert((**r_r_x).0 == r_r_x.0);
    assert((**r_r_x).1 == T::different());
    assert((**r_r_x).1 == r_r_x.1);

    assert(r_r_x.0 == r_mut_r_mut_x.0);
    assert(r_r_x.1 == r_mut_r_mut_x.1);

    assert((***r_r_r_x).0 == T::new());
    assert((***r_r_r_x).0 == r_r_r_x.0);
    assert((***r_r_r_x).1 == T::different());
    assert((***r_r_r_x).1 == r_r_r_x.1);

    assert(r_r_r_x.0 == r_mut_r_mut_r_mut_x.0);
    assert(r_r_r_x.1 == r_mut_r_mut_r_mut_x.1);

    x.0 = T::different();
    x.1 = T::new();

    assert((*r_x).0 == T::different());
    assert((*r_x).0 == r_x.0);
    assert((*r_x).1 == T::new());
    assert((*r_x).1 == r_x.1);

    assert(r_x.0 == r_mut_x.0);
    assert(r_x.1 == r_mut_x.1);

    assert((**r_r_x).0 == T::different());
    assert((**r_r_x).0 == r_r_x.0);
    assert((**r_r_x).1 == T::new());
    assert((**r_r_x).1 == r_r_x.1);

    assert(r_r_x.0 == r_mut_r_mut_x.0);
    assert(r_r_x.1 == r_mut_r_mut_x.1);

    assert((***r_r_r_x).0 == T::different());
    assert((***r_r_r_x).0 == r_r_r_x.0);
    assert((***r_r_r_x).1 == T::new());
    assert((***r_r_r_x).1 == r_r_r_x.1);

    assert(r_r_r_x.0 == r_mut_r_mut_r_mut_x.0);
    assert(r_r_r_x.1 == r_mut_r_mut_r_mut_x.1);
}

#[inline(never)]
fn dereference_tuple_not_inlined<T>()
    where T: TestInstance + Eq
{
    dereference_tuple::<T>()
}

#[inline(always)]
fn dereference_tuple_of_refs<T>()
    where T: TestInstance + Eq
{
    let mut x1 = (T::new(), T::different());
    let mut x2 = (T::new(), T::different());

    let embed = (& & &x1, & &x2);
    let mut embed_mut = (&mut &mut &mut x1, &mut &mut x2);

    let r_embed = &embed;
    let r_r_embed = &r_embed;
    let r_r_r_embed = &r_r_embed;

    let mut r_mut_embed_mut = &mut embed_mut;
    let mut r_mut_r_mut_embed_mut = &mut r_mut_embed_mut;
    let r_mut_r_mut_r_mut_embed_mut = &mut r_mut_r_mut_embed_mut;

    assert(r_embed.0.0 == T::new());
    assert(r_embed.0.1 == T::different());
    assert(r_embed.1.0 == T::new());
    assert(r_embed.1.1 == T::different());

    assert(r_embed.0.0 == r_mut_embed_mut.0.0);
    assert(r_embed.0.1 == r_mut_embed_mut.0.1);
    assert(r_embed.1.0 == r_mut_embed_mut.1.0);
    assert(r_embed.1.1 == r_mut_embed_mut.1.1);

    assert(r_r_embed.0.0 == T::new());
    assert(r_r_embed.0.1 == T::different());
    assert(r_r_embed.1.0 == T::new());
    assert(r_r_embed.1.1 == T::different());

    assert(r_r_embed.0.0 == r_mut_r_mut_embed_mut.0.0);
    assert(r_r_embed.0.1 == r_mut_r_mut_embed_mut.0.1);
    assert(r_r_embed.1.0 == r_mut_r_mut_embed_mut.1.0);
    assert(r_r_embed.1.1 == r_mut_r_mut_embed_mut.1.1);

    assert(r_r_r_embed.0.0 == T::new());
    assert(r_r_r_embed.0.1 == T::different());
    assert(r_r_r_embed.1.0 == T::new());
    assert(r_r_r_embed.1.1 == T::different());

    assert(r_r_r_embed.0.0 == r_mut_r_mut_r_mut_embed_mut.0.0);
    assert(r_r_r_embed.0.1 == r_mut_r_mut_r_mut_embed_mut.0.1);
    assert(r_r_r_embed.1.0 == r_mut_r_mut_r_mut_embed_mut.1.0);
    assert(r_r_r_embed.1.1 == r_mut_r_mut_r_mut_embed_mut.1.1);

    x1.0 = T::different();
    x1.1 = T::new();

    x2.0 = T::different();
    x2.1 = T::new();

    assert(r_embed.0.0 == T::different());
    assert(r_embed.0.1 == T::new());
    assert(r_embed.1.0 == T::different());
    assert(r_embed.1.1 == T::new());

    assert(r_embed.0.0 == r_mut_embed_mut.0.0);
    assert(r_embed.0.1 == r_mut_embed_mut.0.1);
    assert(r_embed.1.0 == r_mut_embed_mut.1.0);
    assert(r_embed.1.1 == r_mut_embed_mut.1.1);

    assert(r_r_embed.0.0 == T::different());
    assert(r_r_embed.0.1 == T::new());
    assert(r_r_embed.1.0 == T::different());
    assert(r_r_embed.1.1 == T::new());

    assert(r_r_embed.0.0 == r_mut_r_mut_embed_mut.0.0);
    assert(r_r_embed.0.1 == r_mut_r_mut_embed_mut.0.1);
    assert(r_r_embed.1.0 == r_mut_r_mut_embed_mut.1.0);
    assert(r_r_embed.1.1 == r_mut_r_mut_embed_mut.1.1);

    assert(r_r_r_embed.0.0 == T::different());
    assert(r_r_r_embed.0.1 == T::new());
    assert(r_r_r_embed.1.0 == T::different());
    assert(r_r_r_embed.1.1 == T::new());

    assert(r_r_r_embed.0.0 == r_mut_r_mut_r_mut_embed_mut.0.0);
    assert(r_r_r_embed.0.1 == r_mut_r_mut_r_mut_embed_mut.0.1);
    assert(r_r_r_embed.1.0 == r_mut_r_mut_r_mut_embed_mut.1.0);
    assert(r_r_r_embed.1.1 == r_mut_r_mut_r_mut_embed_mut.1.1);

    let r = & & & & &(& & &T::new(), & &T::different());

    assert(r.0 == & & &T::new());
    assert(r.1 == & &T::different());

    let r = & & & & &(&mut &mut &mut T::new(), &mut &mut T::different());

    assert(r.0 == &mut &mut &mut T::new());
    assert(r.1 == &mut &mut T::different());
}

#[inline(never)]
fn dereference_tuple_of_refs_not_inlined<T>()
    where T: TestInstance + Eq
{
    dereference_tuple_of_refs::<T>()
}

#[inline(never)]
fn test_all_inlined() {
    dereference_tuple::<()>();
    dereference_tuple::<bool>();
    dereference_tuple::<u8>();
    dereference_tuple::<u16>();
    dereference_tuple::<u32>();
    dereference_tuple::<u64>();
    dereference_tuple::<u256>();
    dereference_tuple::<[u64;2]>();
    dereference_tuple::<[u64;0]>();
    dereference_tuple::<Struct>();
    dereference_tuple::<EmptyStruct>();
    dereference_tuple::<str>();
    dereference_tuple::<str[6]>();
    dereference_tuple::<Enum>();
    dereference_tuple::<(u8, u32)>();
    dereference_tuple::<b256>();
    dereference_tuple::<raw_ptr>();
    dereference_tuple::<raw_slice>();
    
    dereference_tuple_of_refs::<()>();
    dereference_tuple_of_refs::<bool>();
    dereference_tuple_of_refs::<u8>();
    dereference_tuple_of_refs::<u16>();
    dereference_tuple_of_refs::<u32>();
    dereference_tuple_of_refs::<u64>();
    dereference_tuple_of_refs::<u256>();
    dereference_tuple_of_refs::<[u64;2]>();
    dereference_tuple_of_refs::<[u64;0]>();
    dereference_tuple_of_refs::<Struct>();
    dereference_tuple_of_refs::<EmptyStruct>();
    dereference_tuple_of_refs::<str>();
    dereference_tuple_of_refs::<str[6]>();
    dereference_tuple_of_refs::<Enum>();
    dereference_tuple_of_refs::<(u8, u32)>();
    dereference_tuple_of_refs::<b256>();
    dereference_tuple_of_refs::<raw_ptr>();
    dereference_tuple_of_refs::<raw_slice>();
}

#[inline(never)]
fn test_not_inlined() {
    dereference_tuple_not_inlined::<()>();
    dereference_tuple_not_inlined::<bool>();
    dereference_tuple_not_inlined::<u8>();
    dereference_tuple_not_inlined::<u16>();
    dereference_tuple_not_inlined::<u32>();
    dereference_tuple_not_inlined::<u64>();
    dereference_tuple_not_inlined::<u256>();
    dereference_tuple_not_inlined::<[u64;2]>();
    dereference_tuple_not_inlined::<[u64;0]>();
    dereference_tuple_not_inlined::<Struct>();
    dereference_tuple_not_inlined::<EmptyStruct>();
    dereference_tuple_not_inlined::<str>();
    dereference_tuple_not_inlined::<str[6]>();
    dereference_tuple_not_inlined::<Enum>();
    dereference_tuple_not_inlined::<(u8, u32)>();
    dereference_tuple_not_inlined::<b256>();
    dereference_tuple_not_inlined::<raw_ptr>();
    dereference_tuple_not_inlined::<raw_slice>();
    
    dereference_tuple_of_refs_not_inlined::<()>();
    dereference_tuple_of_refs_not_inlined::<bool>();
    dereference_tuple_of_refs_not_inlined::<u8>();
    dereference_tuple_of_refs_not_inlined::<u16>();
    dereference_tuple_of_refs_not_inlined::<u32>();
    dereference_tuple_of_refs_not_inlined::<u64>();
    dereference_tuple_of_refs_not_inlined::<u256>();
    dereference_tuple_of_refs_not_inlined::<[u64;2]>();
    dereference_tuple_of_refs_not_inlined::<[u64;0]>();
    dereference_tuple_of_refs_not_inlined::<Struct>();
    dereference_tuple_of_refs_not_inlined::<EmptyStruct>();
    dereference_tuple_of_refs_not_inlined::<str>();
    dereference_tuple_of_refs_not_inlined::<str[6]>();
    dereference_tuple_of_refs_not_inlined::<Enum>();
    dereference_tuple_of_refs_not_inlined::<(u8, u32)>();
    dereference_tuple_of_refs_not_inlined::<b256>();
    dereference_tuple_of_refs_not_inlined::<raw_ptr>();
    dereference_tuple_of_refs_not_inlined::<raw_slice>();
}

fn main() -> u64 {
    test_all_inlined();
    test_not_inlined();

    42
}

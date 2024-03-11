script;

mod impls;
use impls::*;
use core::ops::Eq;

struct S<T>
    where T: TestInstance + Eq
{
    x: T,
    y: T
}

impl<T> TestInstance for S<T>
    where T: TestInstance + Eq
{
    fn new() -> Self {
        S { x: T::new(), y: T::new() }
    }
    fn different() -> Self {
        S { x: T::different(), y: T::different() }
    }
}

impl<T> Eq for S<T>
    where T: TestInstance + Eq
{
    fn eq(self, other: Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

struct EmbedsReferences<T>
    where T: TestInstance + Eq
{
    x: & & &T,
    y: & &T
}

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

struct EmbedsReferencesMut<T>
    where T: TestInstance + Eq
{
    x: &mut &mut &mut T,
    y: &mut &mut T
}

#[inline(always)]
fn dereference_struct<T>()
    where T: TestInstance + Eq
{
    let mut s = S { x: T::new(), y: T::different() };

    let r_s = &s;
    let r_r_s = &r_s;
    let r_r_r_s = &r_r_s;

    let mut r_mut_s = &mut s;
    let mut r_mut_r_mut_s = &mut r_mut_s;
    let r_mut_r_mut_r_mut_s = &mut r_mut_r_mut_s;

    assert((*r_s).x == T::new());
    assert((*r_s).x == r_s.x);
    assert((*r_s).y == T::different());
    assert((*r_s).y == r_s.y);

    assert(r_s.x == r_mut_s.x);
    assert(r_s.y == r_mut_s.y);

    assert((**r_r_s).x == T::new());
    assert((**r_r_s).x == r_r_s.x);
    assert((**r_r_s).y == T::different());
    assert((**r_r_s).y == r_r_s.y);

    assert(r_r_s.x == r_mut_r_mut_s.x);
    assert(r_r_s.y == r_mut_r_mut_s.y);

    assert((***r_r_r_s).x == T::new());
    assert((***r_r_r_s).x == r_r_r_s.x);
    assert((***r_r_r_s).y == T::different());
    assert((***r_r_r_s).y == r_r_r_s.y);

    assert(r_r_r_s.x == r_mut_r_mut_r_mut_s.x);
    assert(r_r_r_s.y == r_mut_r_mut_r_mut_s.y);

    s.x = T::different();
    s.y = T::new();

    assert((*r_s).x == T::different());
    assert((*r_s).x == r_s.x);
    assert((*r_s).y == T::new());
    assert((*r_s).y == r_s.y);

    assert(r_s.x == r_mut_s.x);
    assert(r_s.y == r_mut_s.y);

    assert((**r_r_s).x == T::different());
    assert((**r_r_s).x == r_r_s.x);
    assert((**r_r_s).y == T::new());
    assert((**r_r_s).y == r_r_s.y);

    assert(r_r_s.x == r_mut_r_mut_s.x);
    assert(r_r_s.y == r_mut_r_mut_s.y);

    assert((***r_r_r_s).x == T::different());
    assert((***r_r_r_s).x == r_r_r_s.x);
    assert((***r_r_r_s).y == T::new());
    assert((***r_r_r_s).y == r_r_r_s.y);

    assert(r_r_r_s.x == r_mut_r_mut_r_mut_s.x);
    assert(r_r_r_s.y == r_mut_r_mut_r_mut_s.y);
}

#[inline(never)]
fn dereference_struct_not_inlined<T>()
    where T: TestInstance + Eq
{
    dereference_struct::<T>()
}

#[inline(always)]
fn dereference_struct_of_refs<T>()
    where T: TestInstance + Eq
{
    let mut s1 = S { x: T::new(), y: T::different() };
    let mut s2 = S { x: T::new(), y: T::different() };

    let embed = EmbedsReferences { x: & & &s1, y: & &s2 };
    let mut embed_mut = EmbedsReferencesMut { x: &mut &mut &mut s1, y: &mut &mut s2 };

    let r_embed = &embed;
    let r_r_embed = &r_embed;
    let r_r_r_embed = &r_r_embed;

    let mut r_mut_embed_mut = &mut embed_mut;
    let mut r_mut_r_mut_embed_mut = &mut r_mut_embed_mut;
    let r_mut_r_mut_r_mut_embed_mut = &mut r_mut_r_mut_embed_mut;

    assert(r_embed.x.x == T::new());
    assert(r_embed.x.y == T::different());
    assert(r_embed.y.x == T::new());
    assert(r_embed.y.y == T::different());

    assert(r_embed.x.x == r_mut_embed_mut.x.x);
    assert(r_embed.x.y == r_mut_embed_mut.x.y);
    assert(r_embed.y.x == r_mut_embed_mut.y.x);
    assert(r_embed.y.y == r_mut_embed_mut.y.y);

    assert(r_r_embed.x.x == T::new());
    assert(r_r_embed.x.y == T::different());
    assert(r_r_embed.y.x == T::new());
    assert(r_r_embed.y.y == T::different());

    assert(r_r_embed.x.x == r_mut_r_mut_embed_mut.x.x);
    assert(r_r_embed.x.y == r_mut_r_mut_embed_mut.x.y);
    assert(r_r_embed.y.x == r_mut_r_mut_embed_mut.y.x);
    assert(r_r_embed.y.y == r_mut_r_mut_embed_mut.y.y);

    assert(r_r_r_embed.x.x == T::new());
    assert(r_r_r_embed.x.y == T::different());
    assert(r_r_r_embed.y.x == T::new());
    assert(r_r_r_embed.y.y == T::different());

    assert(r_r_r_embed.x.x == r_mut_r_mut_r_mut_embed_mut.x.x);
    assert(r_r_r_embed.x.y == r_mut_r_mut_r_mut_embed_mut.x.y);
    assert(r_r_r_embed.y.x == r_mut_r_mut_r_mut_embed_mut.y.x);
    assert(r_r_r_embed.y.y == r_mut_r_mut_r_mut_embed_mut.y.y);

    s1.x = T::different();
    s1.y = T::new();

    s2.x = T::different();
    s2.y = T::new();

    assert(r_embed.x.x == T::different());
    assert(r_embed.x.y == T::new());
    assert(r_embed.y.x == T::different());
    assert(r_embed.y.y == T::new());

    assert(r_embed.x.x == r_mut_embed_mut.x.x);
    assert(r_embed.x.y == r_mut_embed_mut.x.y);
    assert(r_embed.y.x == r_mut_embed_mut.y.x);
    assert(r_embed.y.y == r_mut_embed_mut.y.y);

    assert(r_r_embed.x.x == T::different());
    assert(r_r_embed.x.y == T::new());
    assert(r_r_embed.y.x == T::different());
    assert(r_r_embed.y.y == T::new());

    assert(r_r_embed.x.x == r_mut_r_mut_embed_mut.x.x);
    assert(r_r_embed.x.y == r_mut_r_mut_embed_mut.x.y);
    assert(r_r_embed.y.x == r_mut_r_mut_embed_mut.y.x);
    assert(r_r_embed.y.y == r_mut_r_mut_embed_mut.y.y);

    assert(r_r_r_embed.x.x == T::different());
    assert(r_r_r_embed.x.y == T::new());
    assert(r_r_r_embed.y.x == T::different());
    assert(r_r_r_embed.y.y == T::new());

    assert(r_r_r_embed.x.x == r_mut_r_mut_r_mut_embed_mut.x.x);
    assert(r_r_r_embed.x.y == r_mut_r_mut_r_mut_embed_mut.x.y);
    assert(r_r_r_embed.y.x == r_mut_r_mut_r_mut_embed_mut.y.x);
    assert(r_r_r_embed.y.y == r_mut_r_mut_r_mut_embed_mut.y.y);

    let r = & & & & &EmbedsReferences { x: & & &T::new(), y: & &T::different() };

    assert(r.x == & & &T::new());
    assert(r.y == & &T::different());

    let r = & & & & &EmbedsReferencesMut { x: &mut &mut &mut T::new(), y: &mut &mut T::different() };

    assert(r.x == &mut &mut &mut T::new());
    assert(r.y == &mut &mut T::different());
}

#[inline(never)]
fn dereference_struct_of_refs_not_inlined<T>()
    where T: TestInstance + Eq
{
    dereference_struct_of_refs::<T>()
}

#[inline(never)]
fn test_all_inlined() {
    dereference_struct::<()>();
    dereference_struct::<bool>();
    dereference_struct::<u8>();
    dereference_struct::<u16>();
    dereference_struct::<u32>();
    dereference_struct::<u64>();
    dereference_struct::<u256>();
    dereference_struct::<[u64;2]>();
    dereference_struct::<[u64;0]>();
    dereference_struct::<Struct>();
    dereference_struct::<EmptyStruct>();
    dereference_struct::<str>();
    dereference_struct::<str[6]>();
    dereference_struct::<Enum>();
    dereference_struct::<(u8, u32)>();
    dereference_struct::<b256>();
    dereference_struct::<raw_ptr>();
    dereference_struct::<raw_slice>();

    dereference_struct_of_refs::<()>();
    dereference_struct_of_refs::<bool>();
    dereference_struct_of_refs::<u8>();
    dereference_struct_of_refs::<u16>();
    dereference_struct_of_refs::<u32>();
    dereference_struct_of_refs::<u64>();
    dereference_struct_of_refs::<u256>();
    dereference_struct_of_refs::<[u64;2]>();
    dereference_struct_of_refs::<[u64;0]>();
    dereference_struct_of_refs::<Struct>();
    dereference_struct_of_refs::<EmptyStruct>();
    dereference_struct_of_refs::<str>();
    dereference_struct_of_refs::<str[6]>();
    dereference_struct_of_refs::<Enum>();
    dereference_struct_of_refs::<(u8, u32)>();
    dereference_struct_of_refs::<b256>();
    dereference_struct_of_refs::<raw_ptr>();
    dereference_struct_of_refs::<raw_slice>();
}

#[inline(never)]
fn test_not_inlined() {
    dereference_struct_not_inlined::<()>();
    dereference_struct_not_inlined::<bool>();
    dereference_struct_not_inlined::<u8>();
    dereference_struct_not_inlined::<u16>();
    dereference_struct_not_inlined::<u32>();
    dereference_struct_not_inlined::<u64>();
    dereference_struct_not_inlined::<u256>();
    dereference_struct_not_inlined::<[u64;2]>();
    dereference_struct_not_inlined::<[u64;0]>();
    dereference_struct_not_inlined::<Struct>();
    dereference_struct_not_inlined::<EmptyStruct>();
    dereference_struct_not_inlined::<str>();
    dereference_struct_not_inlined::<str[6]>();
    dereference_struct_not_inlined::<Enum>();
    dereference_struct_not_inlined::<(u8, u32)>();
    dereference_struct_not_inlined::<b256>();
    dereference_struct_not_inlined::<raw_ptr>();
    dereference_struct_not_inlined::<raw_slice>();

    dereference_struct_of_refs_not_inlined::<()>();
    dereference_struct_of_refs_not_inlined::<bool>();
    dereference_struct_of_refs_not_inlined::<u8>();
    dereference_struct_of_refs_not_inlined::<u16>();
    dereference_struct_of_refs_not_inlined::<u32>();
    dereference_struct_of_refs_not_inlined::<u64>();
    dereference_struct_of_refs_not_inlined::<u256>();
    dereference_struct_of_refs_not_inlined::<[u64;2]>();
    dereference_struct_of_refs_not_inlined::<[u64;0]>();
    dereference_struct_of_refs_not_inlined::<Struct>();
    dereference_struct_of_refs_not_inlined::<EmptyStruct>();
    dereference_struct_of_refs_not_inlined::<str>();
    dereference_struct_of_refs_not_inlined::<str[6]>();
    dereference_struct_of_refs_not_inlined::<Enum>();
    dereference_struct_of_refs_not_inlined::<(u8, u32)>();
    dereference_struct_of_refs_not_inlined::<b256>();
    dereference_struct_of_refs_not_inlined::<raw_ptr>();
    dereference_struct_of_refs_not_inlined::<raw_slice>();
}

fn main() -> u64 {
    test_all_inlined();
    test_not_inlined();

    42
}

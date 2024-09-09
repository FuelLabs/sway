library;

impl &mut u64 {
    fn ref_mut_u64_method(self) { }
}

impl &mut &mut u64 {
    fn ref_mut_ref_mut_u64_method(self) { }
}

struct S {}

trait Trait<T> { }

// TODO: Add checks for conflicting implementations of the `Trait<T>` once https://github.com/FuelLabs/sway/issues/5686 is fixed.

impl<A> Trait<A> for &u64 { }
impl<B> Trait<B> for &u64 { }

impl<A> Trait<A> for & &u64 { }
impl<B> Trait<B> for & &u64 { }

impl<A> Trait<A> for &mut &mut u64 { }
impl<B> Trait<B> for &mut &mut u64 { }

pub fn test() {
    let x = 123u64;
    let r_x = &x;
    let r_r_x = & &x;

    r_x.ref_mut_u64_method();

    r_r_x.ref_mut_ref_mut_u64_method();
}
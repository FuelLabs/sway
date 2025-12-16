script;

struct A {
    x: u64,
}

impl PartialEq for A {
    fn eq(self, other: Self) -> bool {
        self.x == other.x
    }
}
impl Eq for A {}

struct S<T>
where
    T: Eq,
{
    r_t: &T,
    r_r_t: & &T,
    r_mut_t: &mut T,
    r_mut_r_mut_t: &mut &mut T,
}

impl<T> PartialEq for S<T>
where
    T: Eq,
{
    fn eq(self, other: Self) -> bool {
        let self_r_t_ptr = asm(r: self.r_t) {
            r: raw_ptr
        };
        let self_r_r_t_ptr = asm(r: self.r_r_t) {
            r: raw_ptr
        };
        let self_r_mut_t_ptr = asm(r: self.r_mut_t) {
            r: raw_ptr
        };
        let self_r_mut_r_mut_t_ptr = asm(r: self.r_mut_r_mut_t) {
            r: raw_ptr
        };

        let other_r_t_ptr = asm(r: other.r_t) {
            r: raw_ptr
        };
        let other_r_r_t_ptr = asm(r: other.r_r_t) {
            r: raw_ptr
        };
        let other_r_mut_t_ptr = asm(r: other.r_mut_t) {
            r: raw_ptr
        };
        let other_r_mut_r_mut_t_ptr = asm(r: other.r_mut_r_mut_t) {
            r: raw_ptr
        };

        assert(self_r_t_ptr.read::<T>() == *self.r_t);
        assert(other_r_t_ptr.read::<T>() == *other.r_t);
        assert(self_r_r_t_ptr.read::<raw_ptr>().read::<T>() == **self.r_r_t);
        assert(other_r_r_t_ptr.read::<raw_ptr>().read::<T>() == **other.r_r_t);
        assert(self_r_mut_t_ptr.read::<T>() == *self.r_mut_t);
        assert(other_r_mut_t_ptr.read::<T>() == *other.r_mut_t);
        assert(self_r_mut_r_mut_t_ptr.read::<raw_ptr>().read::<T>() == **self.r_mut_r_mut_t);
        assert(other_r_mut_r_mut_t_ptr.read::<raw_ptr>().read::<T>() == **other.r_mut_r_mut_t);

        self_r_t_ptr
            .read::<T>() == other_r_t_ptr
            .read::<T>()
        && self_r_r_t_ptr
            .read::<raw_ptr>()
            .read::<T>() == other_r_r_t_ptr
            .read::<raw_ptr>()
            .read::<T>()
        && self_r_mut_t_ptr
            .read::<T>() == other_r_mut_t_ptr
            .read::<T>()
            && self_r_mut_r_mut_t_ptr
                .read::<raw_ptr>()
                .read::<T>() == other_r_mut_r_mut_t_ptr
                .read::<raw_ptr>()
                .read::<T>()
    }
}
impl<T> Eq for S<T>
where
    T: Eq,
{}

fn test<T>(s: S<T>, v: T)
where
    T: Eq,
{
    let s_r_t_ptr = asm(r: s.r_t) {
        r: raw_ptr
    };
    let s_r_r_t_ptr = asm(r: s.r_r_t) {
        r: raw_ptr
    };
    let s_r_mut_t_ptr = asm(r: s.r_mut_t) {
        r: raw_ptr
    };
    let s_r_mut_r_mut_t_ptr = asm(r: s.r_mut_r_mut_t) {
        r: raw_ptr
    };

    assert(s_r_t_ptr.read::<T>() == v);
    assert(s_r_r_t_ptr.read::<raw_ptr>().read::<T>() == v);
    assert(s_r_mut_t_ptr.read::<T>() == v);
    assert(s_r_mut_r_mut_t_ptr.read::<raw_ptr>().read::<T>() == v);

    assert(*s.r_t == v);
    assert(**s.r_r_t == v);
    assert(*s.r_mut_t == v);
    assert(**s.r_mut_r_mut_t == v);
}

fn main() -> u64 {
    let mut x = 123u8;

    let mut s_x = S {
        r_t: &x,
        r_r_t: & &x,
        r_mut_t: &mut x,
        r_mut_r_mut_t: &mut &mut x,
    };
    test(s_x, x);

    let s_s_x = S {
        r_t: &s_x,
        r_r_t: & &s_x,
        r_mut_t: &mut s_x,
        r_mut_r_mut_t: &mut &mut s_x,
    };
    test(s_s_x, s_x);

    let mut a = A { x: 321 };

    let s_a = S {
        r_t: &a,
        r_r_t: & &a,
        r_mut_t: &mut a,
        r_mut_r_mut_t: &mut &mut a,
    };
    test(s_a, a);

    42
}

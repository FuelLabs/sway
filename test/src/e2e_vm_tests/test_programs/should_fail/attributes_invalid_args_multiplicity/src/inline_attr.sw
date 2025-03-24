library;

struct S { }

impl S {
    #[inline(always)]
    fn ok_1() { }

    #[inline]
    #[inline()]
    #[inline(always, never)]
    fn not_ok() { }
}
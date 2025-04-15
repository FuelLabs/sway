library;

struct S { }

impl S {
    #[inline(always)]
    fn ok() { }

    #[inline(never)]
    #[inline(always)]
    #[inline(always), inline(never)]
    fn not_ok() { }
}
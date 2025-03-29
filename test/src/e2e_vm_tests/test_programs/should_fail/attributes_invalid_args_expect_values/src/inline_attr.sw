library;

struct S { }

impl S {
    #[inline(always = false)]
    fn not_ok() { }
}
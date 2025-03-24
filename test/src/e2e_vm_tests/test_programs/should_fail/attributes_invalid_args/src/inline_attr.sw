library;

struct S { }

impl S {
    #[inline(always)]
    fn ok_1() { }
    
    #[inline(never)]
    fn ok_1() { }

    #[inline(alwys)]
    #[inline(unknown_arg)]
    fn not_ok() { }
}
library;

struct S { }

impl S {
    #[trace(always)]
    fn ok_1() {
        panic "Panics for tracing purposes.";
    }
    
    #[trace(never)]
    fn ok_1() {
        panic "Panics for tracing purposes.";
    }

    #[trace(alwys)]
    #[trace(unknown_arg)]
    fn not_ok() {
        panic "Panics for tracing purposes.";
    }
}
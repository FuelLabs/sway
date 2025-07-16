library;

struct S { }

impl S {
    #[trace(always)]
    fn ok() {
        panic "Panics for tracing purposes.";
    }

    #[trace(never)]
    #[trace(always)]
    #[trace(always), trace(never)]
    fn not_ok() {
        panic "Panics for tracing purposes.";
    }
}
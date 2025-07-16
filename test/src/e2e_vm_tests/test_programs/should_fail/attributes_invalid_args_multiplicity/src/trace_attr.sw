library;

struct S { }

impl S {
    #[trace(always)]
    fn ok_1() {
        panic "Panics for tracing purposes.";
    }

    #[trace]
    #[trace()]
    #[trace(always, never)]
    fn not_ok() {
        panic "Panics for tracing purposes.";
    }
}
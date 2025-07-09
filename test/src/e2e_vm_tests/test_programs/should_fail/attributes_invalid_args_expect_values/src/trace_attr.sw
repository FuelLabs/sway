library;

struct S { }

impl S {
    #[trace(always = false)]
    fn not_ok() {
        panic "Panics for tracing purposes.";
    }
}
//! Tests that trait implementations can be declared inside
//! of function bodies.
script;

struct S {}
struct R {}

trait Twenty {
    fn twenty(self) -> u64;
}

fn main() -> u64 {
    // Implementing inherent methods in a function body.
    impl S {
        fn ten(self) -> u64 {
            10
        }
    }

    // Implementing a trait in a function body.
    impl Twenty for S {
        fn twenty(self) -> u64 {
            20
        }
    }

    // Implementing inherent methods in a nested block expression.
    let twelve = {
        impl R {
            fn twelve(self) -> u64 {
                12
            }
        }
        let r = R {};
        r.twelve()
    };

    let s = S {};
    s.ten() + s.twenty() + twelve
}

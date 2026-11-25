library;

trait T {
    fn tf(self) -> u64;
}

struct S {}

fn main() -> u64 {
    impl T for S {
        fn tf(self) -> u64 {
            1
        }
    }

    impl T for S {
        fn tf(self) -> u64 {
            2
        }
    }

    42
}

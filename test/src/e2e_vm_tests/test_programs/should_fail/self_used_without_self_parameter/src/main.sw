library;

struct S {}

impl S {
    fn use_self_method() {
        let _ = self.x();
    }

    fn use_self_value() {
        let _ = self;
    }

    fn x(self) -> u64 {
        0
    }
}

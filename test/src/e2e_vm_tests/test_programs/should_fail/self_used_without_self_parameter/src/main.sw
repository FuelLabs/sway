library;

struct S {}

impl S {
    fn use_self() {
        let _ = self.x();
        let _ = self;
    }

    fn x(self) -> u64 {
        0
    }
}

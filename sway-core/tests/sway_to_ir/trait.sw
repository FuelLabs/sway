script;

trait Pred {
    fn pred(self) -> bool;
} {
    fn pred_or(self, other: Self) -> bool {
        self.pred() || other.pred()
    }
}

struct Foo {
    a: bool,
}

impl Pred for Foo {
    fn pred(self) -> bool {
        self.a
    }
}

fn main() -> bool {
    let foo = Foo {
        a: true
    };
    let bar = Foo {
        a: false
    };
    foo.pred_or(bar)
}

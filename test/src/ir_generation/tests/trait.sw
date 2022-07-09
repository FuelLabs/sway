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

// check:  ptr { bool } bar
// check:  ptr { bool } foo

// check: get_ptr ptr { bool } foo, ptr { bool }, 0
// check: get_ptr ptr { bool } bar, ptr { bool }, 0

// check: $(foo_ptr=$VAL) = get_ptr ptr { bool } foo, ptr { bool }, 0
// check: $(bar_ptr=$VAL) = get_ptr ptr { bool } bar, ptr { bool }, 0
// check: $(res=$VAL) = call $(pred_or=$ID)($foo_ptr, $bar_ptr)
// check: ret bool $res

// check: fn $pred_or(self $MD: { bool }, other $MD: { bool }) -> bool
// check: $(self_pred=$VAL) = call $ID(self)
// check: cbr $self_pred, $(block1=$ID), $(block0=$ID)

// check: $block0:
// check: $VAL = phi(entry: $self_pred)
// check: $(other_pred=$VAL) = call $ID(other)
// check: br $block1

// check: $block1:
// check: $(res=$VAL) = phi(entry: $self_pred, $block0: $other_pred)
// check: ret bool $res

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

// check:  { bool } bar
// check:  { bool } foo

// check: get_local { bool } foo
// check: get_local { bool } bar

// check: $(foo_var=$VAL) = get_local { bool } foo
// check: $(bar_var=$VAL) = get_local { bool } bar
// check: $(res=$VAL) = call $(pred_or=$ID)($foo_var, $bar_var)
// check: ret bool $res

// check: fn $pred_or(self $MD: { bool }, other $MD: { bool }) -> bool
// check: $(self_pred=$VAL) = call $ID(self)
// check: cbr $self_pred, $(block1=$ID)($self_pred), $(block0=$ID)()

// check: $block0():
// check: $(other_pred=$VAL) = call $ID(other)
// check: br $block1

// check: $block1($(res=$VAL): bool):
// check: ret bool $res

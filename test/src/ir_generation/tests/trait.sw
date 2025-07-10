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

// check: get_local __ptr { bool }, foo
// check: get_local __ptr { bool }, bar

// check: $(foo_ptr=$VAL) = get_local __ptr { bool }, foo
// check: $(foo_val=$VAL) = load $foo_ptr
// check: $(bar_ptr=$VAL) = get_local __ptr { bool }, bar
// check: $(bar_val=$VAL) = load $bar_ptr
// check: $(res=$VAL) = call $(pred_or=$ID)($foo_val, $bar_val)
// check: ret bool $res

// check: fn $pred_or(self $MD: { bool }, other $MD: { bool }) -> bool
// check: store self to $VAL
// check: store other to $VAL
// check: $(self_ptr=$VAL) = get_local __ptr { bool }, self_
// check: $(self_val=$VAL) = load $self_ptr
// check: $(self_pred=$VAL) = call $ID($self_val)
// check: cbr $self_pred, $(block1=$ID)($self_pred), $(block0=$ID)()

// check: $block0():
// check: $(other_ptr=$VAL) = get_local __ptr { bool }, other_
// check: $(other_val=$VAL) = load $other_ptr
// check: $(other_pred=$VAL) = call $ID($other_val)
// check: br $block1

// check: $block1($(res=$VAL): bool):
// check: ret bool $res

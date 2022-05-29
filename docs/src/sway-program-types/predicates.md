# Predicates

From the perspective of Sway, predicates are programs that return a Boolean value and which represent ownership of some resource upon execution to true. They have no access to contract storage. Here is a trivial predicate, which always evaluates to true:

```sway
predicate;

// All predicates require a main function which returns a Boolean value.
fn main() -> bool {
    true
}
```

## Debugging Predicates

Because they don't have any side effects (they are _pure_), predicates cannot create receipts. Therefore, they cannot have logging or create a stack backtrace. This means that there is no naive way to debug them aside from using a single-stepping debugger (which is a [work-in-progress](https://github.com/FuelLabs/fuel-debugger/pull/1)).

As a workaround, the predicate can be written, tested, and debugged first as a `script`, and then changed back into a `predicate`.

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

- Because they don't have any side effects (they are _pure_), predicates *cannot* create receipts (which are transactions), and thus cannot have logging or create a backtrace. This means that there is no native way to debug them aside from using a single-stepping debugger.
- As a workaround, the predicate can be written, tested, and debugged first as a script, and then changed back into a `predicate`.

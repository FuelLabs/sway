# Predicates

From the perspective of Sway, predicates are programs that return a Boolean value and which represent ownership of some resource upon execution to true. They have no access to contract storage. Here is a trivial predicate, which always evaluates to true:

```sway
predicate;

// All predicates require a main function which returns a Boolean value.
fn main() -> bool {
    true
}
```

## Debugging predicates

- Because they don't have any side-effects (they are _pure functions_), predicates *cannot* create receipts (which are transactions), and thus cannot have logging or create a backtrace. Which means no native way to debug aside from single-stepping debuggers.
- Given the above, we suggest that to debug and write _predicates_, you should actually write it  first  in a _script_, and, once it works as expected, move it inside a _predicate_.

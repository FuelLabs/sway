# Predicates

From the perspective of Sway, predicates are programs that return a Boolean value and which represent ownership of some resource upon execution to true. They have no access to contract storage. Here is a trivial predicate, which always evaluates to true:

```sway
predicate;

// All predicates require a main function which returns a Boolean value.
fn main() -> bool {
    true
}
```

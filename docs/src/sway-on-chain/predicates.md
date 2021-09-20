# Predicates

From the perspective of Sway, predicates are programs which return a boolean value and which represent ownership of some resource upon execution to true. They have no access to contract storage. Here is a trivial predicate, which always evaluates to true:

```sway
predicate;

fn main() -> bool {
  true
}
```

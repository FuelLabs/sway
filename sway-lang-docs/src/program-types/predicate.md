# Predicates

A predicate is a program which represents ownership of some resource upon execution to the Boolean value of `true`. They do not have access to contract storage. 

Here is a trivial predicate, which always evaluates to true:

```sway
predicate;

// All predicates require a main function which returns a Boolean value.
fn main() -> bool {
    true
}
```

## Debugging Predicates

A predicate does not have any side effects because it is pure and thus it cannot create receipts. Since there are no receipts they cannot use logging nor create a stack backtrace for debugging. This means that there is no way to debug them aside from using a single-stepping [debugger](https://github.com/FuelLabs/fuel-debugger).

As a workaround, the predicate can be written, tested, and debugged first as a `script`, and then changed back into a `predicate`.

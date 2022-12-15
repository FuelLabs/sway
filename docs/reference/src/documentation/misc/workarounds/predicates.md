# Predicates

A [predicate](../../language/program-types/predicate.md) does not have any side effects because it is pure and thus it cannot create [receipts](https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/abi.md#receipt).

Since there are no receipts they cannot use logging nor create a stack backtrace for debugging. This means that there is no way to debug them aside from using a single-stepping [debugger](https://github.com/FuelLabs/fuel-debugger).

As a workaround, the predicate can be written, tested, and debugged first as a [`script`](../../language/program-types/script.md), and then changed back into a `predicate`.

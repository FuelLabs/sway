# Assertions

An assertion is a condition which must evaluate to the [Boolean](../../language/built-ins/boolean.md) value of `true` and its purpose is to prevent undesirable computation when the condition is evaluated to `false`.

For example, a function may only work if the condition `argument < 5` is `true`. We can use an assertion to enforce this condition by:

- Forcing a revert in the program when `5 <= argument`
- Handling the exception with additional code

Handling exceptions may be done through [if expressions](../../language/control-flow/if-expressions.md) therefore the following sections will take a look at how we can make the virtual machine revert (safely crash).

- [`assert`](assert.md): Checks if a `condition` is `true` otherwise reverts
- [`require`](require.md): Checks if a `condition` is `true` otherwise logs a `value` and reverts
- [`revert`](revert.md): Reverts the virtual machine with the provided exit code
- [`assert_eq`](assert-eq.md): Checks if `a` and `b` are equal otherwise reverts
- [`assert_ne`](assert-ne.md): Checks if `a` and `b` are not equal otherwise reverts

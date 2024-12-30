# Re-entrancy

Re-entrancy occurs when a contract makes a call back into the contract that called it, e.g. `Contract A` calls `Contract B` but then `Contract B` makes a call back into `Contract A`.

To mitigate security concerns there are two approaches that are commonly used:

- [Implement a guard](#re-entrancy-guard): detect when a re-entrancy occurs
- [Defensive programming](#checks-effects-interactions-pattern): perform calls after all state changes have been made

## Re-entrancy Guard

Sway provides a stateless [re-entrancy](https://fuellabs.github.io/sway-libs/book/reentrancy/index.html) guard, which reverts at run-time when re-entrancy is detected.

To use the guard we must import it.

```sway
{{#include ../../code/operations/re_entrency/src/main.sw:import}}
```

Then call it in a contract function.

```sway
{{#include ../../code/operations/re_entrency/src/main.sw:guard}}
```

## Checks-Effects-Interactions Pattern

The pattern states that all state (storage) changes should be made before a call is made.

```sway
{{#include ../../code/operations/re_entrency/src/main.sw:check}}
```

# Fizzbuzz

The following example implements the fizzbuzz game.

The rules are:

- A number divisible by `3` returns `Fizz`
- A number divisible by `5` returns `Buzz`
- A number which is divisible by `3` & `5` returns `Fizzbuzz`
- Any other number entered is returned back to the user

## State

Let's define an [`enum`](../language/built-ins/enums.md) which contains the state of the game.

```sway
{{#include ../../code/examples/fizzbuzz/src/lib.sw:state}}
```

## Implementation

We can write a [`function`](../language/functions/index.md) which takes an `input` and checks its divisibility. Depending on the result a different `State` will be returned.

```sway
{{#include ../../code/examples/fizzbuzz/src/lib.sw:fizzbuzz}}
```
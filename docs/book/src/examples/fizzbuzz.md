# FizzBuzz

This example is not the traditional [FizzBuzz](https://en.wikipedia.org/wiki/Fizz_buzz#Programming); instead it is the smart contract version! A script can call the `fizzbuzz` ABI method of this contract with some `u64` value and receive back its fizzbuzzability as an `enum`.

The format for custom structs and enums such as `FizzBuzzResult` will be automatically included in the ABI JSON so that off-chain code can handle the encoded form of the returned data.

```sway
{{#include ../../../../examples/fizzbuzz/src/main.sw}}
```

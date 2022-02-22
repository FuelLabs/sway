# FizzBuzz

This example is not the traditional fizzbuzz, instead it is the smart contract version! A script can call this contract with some u64 value
and receive back its fizzbuzzability as an enum. Note that the deserialization scheme for the fizzbuzz enum will be included in the ABI descriptor
so the caller knows what to do with the bytes.

```sway
{{#include ../../../examples/fizzbuzz/src/main.sw}}
```

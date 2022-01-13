# FizzBuzz

This example is not the traditional fizzbuzz, instead it is the smart contract version! A script can call this contract with some u64 value
and receive back its fizzbuzzability as an enum. Note that the deserialization scheme for the fizzbuzz enum will be included in the ABI descriptor
so the caller knows what to do with the bytes.

```sway
contract;

enum FizzBuzzResult {
    Fizz: (),
    Buzz: (),
    FizzBuzz: (),
    Other: u64,
}

abi FizzBuzz {
    fn fizzbuzz(gas: u64, coins: u64, asset_id: b256, input: u64) -> FizzBuzzResult;
}

impl FizzBuzz for Contract {
    fn fizzbuzz(gas: u64, coins: u64, asset_id: b256, input: u64) -> FizzBuzzResult {
        if input % 15 == 0 {
            FizzBuzzResult::FizzBuzz
        } else if input % 3 == 0 {
            FizzBuzzResult::Fizz
        } else if input % 5 == 0 {
            FizzBuzzResult::Buzz
        } else {
            FizzBuzzResult::Other(input)
        }
    }
}

```

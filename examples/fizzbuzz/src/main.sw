contract;

use std::contract_id::ContractId;

enum FizzBuzzResult {
    Fizz: (),
    Buzz: (),
    FizzBuzz: (),
    Other: u64,
}

abi FizzBuzz {
    fn fizzbuzz(gas: u64, coins: u64, asset_id: ContractId, input: u64) -> FizzBuzzResult;
}

impl FizzBuzz for Contract {
    fn fizzbuzz(gas: u64, coins: u64, asset_id: ContractId, input: u64) -> FizzBuzzResult {
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

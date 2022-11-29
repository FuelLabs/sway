contract;

enum FizzBuzzResult {
    Fizz: (),
    Buzz: (),
    FizzBuzz: (),
    Other: u64,
}

abi FizzBuzz {
    #[storage(read)]
    fn fizzbuzz(input: u64, input2: u32) -> FizzBuzzResult;
    #[storage(read, write)]
    fn fizzbuzz2(input: u64, input3: FizzBuzzResult) -> FizzBuzzResult;
    #[storage(write, read)]
    fn fizzbuzz3(input: u64, input3: FizzBuzzResult) -> FizzBuzzResult;
    fn fizzbuzz4(input: u64) -> FizzBuzzResult;
}

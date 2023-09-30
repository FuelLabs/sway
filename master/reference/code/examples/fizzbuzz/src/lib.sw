library;

// ANCHOR: state
enum State {
    Fizz: (),
    Buzz: (),
    FizzBuzz: (),
    Other: u64,
}
// ANCHOR_END: state
// ANCHOR: fizzbuzz
fn fizzbuzz(input: u64) -> State {
    if input % 15 == 0 {
        State::FizzBuzz
    } else if input % 3 == 0 {
        State::Fizz
    } else if input % 5 == 0 {
        State::Buzz
    } else {
        State::Other(input)
    }
}
// ANCHOR_END: fizzbuzz

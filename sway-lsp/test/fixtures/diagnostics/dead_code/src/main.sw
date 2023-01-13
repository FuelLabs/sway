contract;

abi MyContract {
    fn test_function() -> bool;
}

impl MyContract for Contract {
    fn test_function() -> bool {
        true
    }
}

fn not_used1() -> bool {
    return true;
}
/// Comments about unused code
fn not_used2(input: u64) -> u64 {
    return input + 1;
}

enum NotUsed {
    A: (),
    B: (),
}

const NOT_USED_NUM = 15;
const NOT_USED_WITH_TYPE: bool = true;

struct not_used_struct {
    a: bool,
}

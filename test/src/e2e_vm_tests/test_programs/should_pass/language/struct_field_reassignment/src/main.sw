script;
// this file tests struct field reassignments
fn main() -> u64 {
    let mut _data = Data {
        value: NumberOrString::Number(20),
        address: 0b00001111,
    };

    _data.value = NumberOrString::String( "sway");
    return 0;
}

enum NumberOrString {
    Number: u64,
    String: str[4],
}

struct Data {
    value: NumberOrString,
    address: u8,
}

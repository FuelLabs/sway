contract;

/// Enum representing either a number or a string
///
/// # Examples
///
/// `NumberOrString::Number(42)`
/// `NumberOrString::String("foo")`
enum NumberOrString {
    /// The `Number` variant in `NumberOrString`
    Number: u64,
    /// The `String` variant in `NumberOrString`
    String: str[4],
}

/// Struct holding:
///
/// 1. A `value` of type `NumberOrString`
/// 2. An `address` of type `u64`
struct Data {
    /// The `value` field in `Data`
    value: NumberOrString,
    /// The `address` field in `Data`
    address: u64,
}

/// This is the `FooABI` abi
abi FooABI {
    /// This is the `main` method on the `FooABI` abi
    fn main() -> u64;
}

/// Storage fields for the contract
storage {
    /// A `u64` field
    field_a: u64 = 0,
    /// An `str` field
    field_b: str[4] = "aaaa",
}

/// The implementation of the `FooABI` abi
impl FooABI for Contract {
    /// The main function that does all the things!
    fn main() -> u64 {
        let mut data = Data {
            value: NumberOrString::Number(20),
            address: 1337,
        };

        return 20;
    }
}

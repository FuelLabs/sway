script;

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
/// 2. An `address` of type `byte`
struct Data {
	// e /// The `value` field in `Data`
	value: NumberOrString,
	// e /// The `address` field in `Data`
	address: byte,
}

/// The main function that does all the things!
fn main() -> u64 {
  let mut data = Data { 
                  value: NumberOrString::Number(20),
                  address: 0b00001111,
                };

  return 20;
}
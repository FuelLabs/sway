script;

// ANCHOR: type_alias 
type Kilometers = u64;
// ANCHOR_END: type_alias 

struct MyStruct<T, U> {
    x: T,
    y: U,
}
// ANCHOR: long_type_use
fn foo_long(array: [MyStruct<u64, b256>; 5]) -> [MyStruct<u64, b256>; 5] {
    array
}
// ANCHOR_END: long_type_use

// ANCHOR: long_type_use_shorter
type MyArray = [MyStruct<u64, b256>; 5];

fn foo_shorter(array: MyArray) -> MyArray {
    array
}
// ANCHOR_END: long_type_use_shorter

fn main() {
    // ANCHOR: addition 
    let x: u64 = 5;
    let y: Kilometers = 5;
    assert(x + y == 10);
    // ANCHOR_END: addition 
}

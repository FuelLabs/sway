contract;

mod r#panic;

// This proves that https://github.com/FuelLabs/sway/issues/7134 is fixed.
enum r#enum {
    r#bool: bool,
}

struct r#struct {
    r#u64: u64,
}

// This proves that https://github.com/FuelLabs/sway/issues/7135 is fixed.
#[fallback]
fn r#let() {}

abi r#abi {
    fn r#return() -> u64;
}

impl r#abi for Contract {
    fn r#return() -> u64 {
        main()
    }
}

#[test]
fn r#test() {
    let caller = abi(r#abi, CONTRACT_ID);
    assert_eq(caller.r#return(), 42);
    assert_eq(::r#panic::call_panic(), 42);
    assert_eq(::r#panic::r#panic(42), 42);
}

fn r#fn(x: u64) -> u64 {
    x
}

fn simple_function() -> u64 {
    42
}

fn main() -> u64 {
    // A non-keyword identifier can be used as a raw identifier, interchangeably.
    let mut r#not_an_identifier = 0;
    r#not_an_identifier = 24;
    assert_eq(r#not_an_identifier, 24);
    not_an_identifier = 42;
    assert_eq(not_an_identifier, 42);
    assert_eq(r#not_an_identifier, 42);

    let simple_identifier = 24;
    assert_eq(r#simple_identifier, 24);
    let r#simple_identifier = 42;
    assert_eq(simple_identifier, 42);

    assert_eq(r#simple_function(), 42);

    // Accessing raw identifiers that represent keywords.
    let mut r#script = 0;
    r#script = 42;
    assert_eq(r#script, 42);

    let mut r#contract = 0;
    r#contract = 42;
    assert_eq(r#contract, 42);

    let mut r#predicate = 0;
    r#predicate = 42;
    assert_eq(r#predicate, 42);

    let mut r#library = 0;
    r#library = 42;
    assert_eq(r#library, 42);

    let mut r#mod = 0;
    r#mod = 42;
    assert_eq(r#mod, 42);

    let mut r#pub = 0;
    r#pub = 42;
    assert_eq(r#pub, 42);

    let mut r#use = 0;
    r#use = 42;
    assert_eq(r#use, 42);

    let mut r#as = 0;
    r#as = 42;
    assert_eq(r#as, 42);

    let mut r#struct = 0;
    r#struct = 42;
    assert_eq(r#struct, 42);

    let mut r#enum = 0;
    r#enum = 42;
    assert_eq(r#enum, 42);

    let mut r#self = 0;
    r#self = 42;
    assert_eq(r#self, 42);

    let mut r#fn = 0;
    r#fn = 42;
    assert_eq(r#fn, 42);
    // TODO: Uncomment once https://github.com/FuelLabs/sway/issues/7136 is fixed.
    // let v = r#fn(r#fn);
    // assert_eq(v, 42);

    let mut r#trait = 0;
    r#trait = 42;
    assert_eq(r#trait, 42);

    let mut r#impl = 0;
    r#impl = 42;
    assert_eq(r#impl, 42);

    let mut r#for = 0;
    r#for = 42;
    assert_eq(r#for, 42);

    let mut r#abi = 0;
    r#abi = 42;
    assert_eq(r#abi, 42);

    let mut r#const = 0;
    r#const = 42;
    assert_eq(r#const, 42);

    let mut r#storage = 0;
    r#storage = 42;
    assert_eq(r#storage, 42);

    let mut r#str = 0;
    r#str = 42;
    assert_eq(r#str, 42);

    let mut r#asm = 0;
    r#asm = 42;
    assert_eq(r#asm, 42);

    let mut r#return = 0;
    r#return = 42;
    assert_eq(r#return, 42);

    let mut r#if = 0;
    r#if = 42;
    assert_eq(r#if, 42);

    let mut r#else = 0;
    r#else = 42;
    assert_eq(r#else, 42);

    let mut r#match = 0;
    r#match = 42;
    assert_eq(r#match, 42);

    let mut r#mut = 0;
    r#mut = 42;
    assert_eq(r#mut, 42);

    let mut r#let = 0;
    r#let = 42;
    assert_eq(r#let, 42);

    let mut r#while = 0;
    r#while = 42;
    assert_eq(r#while, 42);

    let mut r#where = 0;
    r#where = 42;
    assert_eq(r#where, 42);

    let mut r#ref = 0;
    r#ref = 42;
    assert_eq(r#ref, 42);

    let mut r#deref = 0;
    r#deref = 42;
    assert_eq(r#deref, 42);

    let mut r#true = 0;
    r#true = 42;
    assert_eq(r#true, 42);

    let mut r#false = 0;
    r#false = 42;
    assert_eq(r#false, 42);

    let mut r#panic = 0;
    r#panic = 42;
    assert_eq(r#panic, 42);

    // Using structs and enums with raw identifiers.
    // TODO: Uncomment once https://github.com/FuelLabs/sway/issues/7136 is fixed.
    // let v = match r#enum::r#bool(false) {
    //     r#enum::r#bool(true) => {
    //         24
    //     },
    //     r#enum::r#bool(false) => {
    //         42
    //     },
    // };
    // assert_eq(v, 42);

    // let v = r#struct { 
    //     r#u64: 42,
    // };
    // assert_eq(v.r#u64, 42);

    // let v = match (r#struct { r#u64: 42 }) { 
    //     r#struct { r#u64 } => {
    //         r#u64
    //     },
    // };
    // assert_eq(v, 42);

    // let v = r#fn(42);
    // assert_eq(v, 42);

    42
}

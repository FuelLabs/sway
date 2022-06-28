script;

enum SomeEnum {
    B: bool,
}

fn main() -> u64 {
    let mut r#script = 0;
    let mut r#contract = 0;
    let mut r#predicate = 0;
    let mut r#library = 0;
    let mut r#dep = 0;
    let mut r#pub = 0;
    let mut r#use = 0;
    let mut r#as = 0;
    let mut r#struct = 0;
    let mut r#enum = 0;
    let mut r#self = 0;
    let mut r#fn = 0;
    let mut r#trait = 0;
    let mut r#impl = 0;
    let mut r#for = 0;
    let mut r#abi = 0;
    let mut r#const = 0;
    let mut r#storage = 0;
    let mut r#str = 0;
    let mut r#asm = 0;
    let mut r#return = 0;
    let mut r#if = 0;
    let mut r#else = 0;
    let mut r#match = 0;
    let mut r#mut = 0;
    let mut r#let = 0;
    let mut r#while = 0;
    let mut r#where = 0;
    let mut r#ref = 0;
    let mut r#deref = 0;
    let mut r#true = 0;
    let mut r#false = 0;

    let e = SomeEnum::B(false);
    let v = match e {
        SomeEnum::B(true) => {
            1
        },
        SomeEnum::B(false) => {
            0
        },
    };

    0
}

script;

fn foo() -> b256 {
    __generate_uid()
}

fn main() {
    let key1 = __generate_uid();
    let key2 = foo();
    let key3 = foo();
}

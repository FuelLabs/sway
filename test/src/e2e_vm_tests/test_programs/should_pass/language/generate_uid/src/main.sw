script;

use std::assert::assert;

fn bar() -> (b256, b256) {
    (foo(), foo())
}

fn foo() -> b256 {
    __generate_uid()
}

fn main() -> b256 {
    let key1 = __generate_uid();
    let key2 = __generate_uid();
    let key3 = foo();
    let key4 = foo();
    let (key5, key6) = bar();

    assert(key1 != key2);
    assert(key1 != key3);
    assert(key1 != key4);
    assert(key1 != key5);
    assert(key1 != key6);
 
    assert(key2 != key3);
    assert(key2 != key4);
    assert(key2 != key5);
    assert(key2 != key6);

    assert(key3 != key4);
    assert(key3 != key5);
    assert(key3 != key6);
 
    assert(key4 != key5);
    assert(key4 != key6);
    
    assert(key5 != key6);

    key1
}

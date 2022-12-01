script;

struct Foo {
    value: u64
}

fn main() -> u64 {
    let mut my_array: [Foo; 1] = [Foo{value: 10}];
    my_array[0] = Foo{value: 20};
    my_array[0].value
}

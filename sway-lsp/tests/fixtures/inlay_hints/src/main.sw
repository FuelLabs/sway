script;

const CONSTANT: u64 = 42;

enum MyEnum {
    A: u64,
}
struct MyStruct {
    a: u64,
}
fn my_function(foo: u64, bar: u64, long_argument_name: u64) -> u64 {
    foo + bar + long_argument_name
}
fn identity<T>(x: T) -> T {
    x
}
fn two_generics<A, B>(_a: A, b: B) -> B {
    b
}
fn three_generics<A, B, C>(a: A, b: B, _c: C) -> B {
    let _a: A = a;
    b
}

fn main() {
    let _x = my_function(1, 2, 3);
    let foo = 1;
    let _y = my_function(foo, 2, 3);
    let bar = 2;
    let _function_call = identity(my_function(1, bar, 3));
    let _z = my_function(foo, bar, 3);
    let long_argument_name = 3;
    let _w = my_function(foo, bar, long_argument_name);
    let _a: bool = identity(true);
    let _b: u32 = identity(10u32);
    let _c: u64 = identity(42);
    let _e: str = identity("foo");
    let _f: u64 = two_generics(true, 10);
    let _g: str = three_generics(true, "foo", 10);
    let _const = identity(CONSTANT);
    let _tuple = identity((1, 2, 3));
    let _array = identity([1, 2, 3]);
    let _enum = identity(MyEnum::A(1));
    let s = MyStruct { a: 1 };
    let _struct_field_access = identity(s.a);
    let t = (0, 1, 9);
    let _tuple_elem_access = identity(t.2);
    let a = [1, 2, 3];
    let _array_index = identity(a[1]);
}


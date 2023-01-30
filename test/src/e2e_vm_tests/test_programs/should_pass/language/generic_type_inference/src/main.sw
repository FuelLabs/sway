script;

dep utils;

use utils::*;

struct CustomType {
    name: str[3],
}

enum MyResult<T, E> {
    Ok: T,
    Err: E,
}

struct SomeStruct<T> {
    a: T,
}

fn simple_vec_test() {
    let mut vec1 = Vec::new();
    let mut vec2 = Vec::new();

    vec2.push(54u32);
    vec1.push(SomeStruct { a: 42 });

    assert(vec1.get(0).unwrap().a == 42);
    assert(vec2.get(0).unwrap() == 54u32);
}

fn complex_vec_test() {
    let mut exp_vec_in_a_vec_in_a_struct_in_a_vec = Vec::new();
    let mut inner_vec_1 = Vec::new();
    let inner_inner_vec_1 = vec_from([0, 1, 2]);

    inner_vec_1.push(inner_inner_vec_1);
    exp_vec_in_a_vec_in_a_struct_in_a_vec.push(SomeStruct { a: inner_vec_1 });

    assert(inner_vec_1.get(0).unwrap().get(1).unwrap() == 1);
    assert(exp_vec_in_a_vec_in_a_struct_in_a_vec.get(0).unwrap().a.get(0).unwrap().get(2).unwrap() == 2);
}

fn simple_option_generics_test() {
    assert(get_an_option::<u64>().is_none());
}

fn test_assert_eq_u64() {
    let a = 42;
    let b = 40 + 2;
    assert_eq(a, b);
}

fn test_try_from() {
    let x = u64::try_from(7);
    assert(x.unwrap() == 42);
}

fn main() {
    sell_product();
    simple_vec_test();
    complex_vec_test();
    simple_option_generics_test();
    test_assert_eq_u64();
    test_try_from();
}

fn sell_product() -> MyResult<bool, CustomType> {
    if false {
        return MyResult::Err(CustomType {
            name: "foo"
        });
    };

    return MyResult::Ok(false);
}

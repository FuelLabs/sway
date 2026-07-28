script;

type ArrayU8Len2 = [u8; 2];
type ArrayNestedLen3 = [ArrayU8Len2; 3];

struct MyStruct {
    f_array: ArrayU8Len2,
    f_nested: ArrayNestedLen3,
}

fn main() -> u64 {
    let a = MyStruct {
        f_array: [1u8, 2u8],
        f_nested: [[1u8, 2u8], [3u8, 4u8], [5u8, 6u8]],
    };
    let b = MyStruct {
        f_array: [1u8, 2u8],
        f_nested: [[1u8, 2u8], [3u8, 4u8], [5u8, 6u8]],
    };

    assert(a.f_array == b.f_array);
    assert(a.f_nested == b.f_nested);

    let mut v: Vec<ArrayU8Len2> = Vec::new();
    v.push([10u8, 20u8]);
    let got: ArrayU8Len2 = v.get(0).unwrap();
    assert(got[0] == 10u8);
    assert(got[1] == 20u8);

    1
}

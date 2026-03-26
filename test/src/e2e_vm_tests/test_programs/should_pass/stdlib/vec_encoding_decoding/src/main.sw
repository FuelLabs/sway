script;

fn main(trivial: Vec<u64>, non_trivial: Vec<u32>) -> (Vec<u64>, Vec<u32>) {
    assert_eq(trivial.len(), 3);
    assert_eq(trivial.get(0).unwrap_or(0), 124);
    assert_eq(trivial.get(1).unwrap_or(0), 124);
    assert_eq(trivial.get(2).unwrap_or(0), 124);

    let mut trivial = Vec::from(trivial.as_raw_slice());
    trivial.push(124);
    trivial.push(124);
    trivial.push(124);

    assert_eq(non_trivial.len(), 3);
    assert_eq(non_trivial.get(0).unwrap_or(0), 124);
    assert_eq(non_trivial.get(1).unwrap_or(0), 124);
    assert_eq(non_trivial.get(2).unwrap_or(0), 124);

    let mut non_trivial = Vec::from(non_trivial.as_raw_slice());
    non_trivial.push(124);
    non_trivial.push(124);
    non_trivial.push(124);

    (trivial, non_trivial)
}

#[test]
fn vec_trivial() {
    let vec = create_vec_trivial(10);
    let encoded_decoded: Vec<u64> = abi_decode(encode(vec));
    assert_eq(encoded_decoded, vec);

    let encoded_as_alias = encode_allow_alias(&encoded_decoded);
    let encoded_decoded: Vec<u64> = abi_decode(encoded_as_alias);
    assert_eq(encoded_decoded, vec);

    log(encoded_decoded);
}

#[test]
fn nested_vec_trivial() {
    let vec = create_nested_vec_trivial(10);
    let encoded_decoded: Vec<Vec<u64>> = abi_decode(encode(vec));
    assert_eq(encoded_decoded, vec);

    let encoded_as_alias = encode_allow_alias(&encoded_decoded);
    let encoded_decoded: Vec<Vec<u64>> = abi_decode(encoded_as_alias);
    assert_eq(encoded_decoded, vec);

    log(encoded_decoded);
}

#[test]
fn vec_non_trivial() {
    let vec = create_vec_trivial(10);
    let encoded_decoded: Vec<u64> = abi_decode(encode(vec));
    assert_eq(encoded_decoded, vec);

    let encoded_as_alias = encode_allow_alias(&encoded_decoded);
    let encoded_decoded: Vec<u64> = abi_decode(encoded_as_alias);
    assert_eq(encoded_decoded, vec);

    log(encoded_decoded);
}

#[test]
fn nested_vec_non_trivial() {
    let vec = create_nested_vec_non_trivial(10);
    let encoded_decoded: Vec<Vec<u32>> = abi_decode(encode(vec));
    assert_eq(encoded_decoded, vec);

    let encoded_as_alias = encode_allow_alias(&encoded_decoded);
    let encoded_decoded: Vec<Vec<u32>> = abi_decode(encoded_as_alias);
    assert_eq(encoded_decoded, vec);

    log(encoded_decoded);
}

#[allow(dead_code)]
fn create_vec_trivial(n: u64) -> Vec<u64> {
    let mut vec = Vec::<u64>::new();
    let mut i: u64 = 0;
    while i < n {
        vec.push(i);
        i += 1;
    }
    vec
}

#[allow(dead_code)]
fn create_nested_vec_trivial(n: u64) -> Vec<Vec<u64>> {
    let mut vec = Vec::<Vec<u64>>::new();
    let mut i: u64 = 0;
    while i < n {
        vec.push(create_vec_trivial(i));
        i += 1;
    }
    vec
}

#[allow(dead_code)]
fn create_vec_non_trivial(n: u32) -> Vec<u32> {
    let mut vec = Vec::<u32>::new();
    let mut i: u32 = 0;
    while i < n {
        vec.push(i);
        i += 1;
    }
    vec
}

#[allow(dead_code)]
fn create_nested_vec_non_trivial(n: u32) -> Vec<Vec<u32>> {
    let mut vec = Vec::<Vec<u32>>::new();
    let mut i: u32 = 0;
    while i < n {
        vec.push(create_vec_non_trivial(i));
        i += 1;
    }
    vec
}

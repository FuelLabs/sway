script;

struct S<T> {
    a: T
}

enum E<T> {
    a: T
}

pub fn main() {
    let m = decode_first_param::<str>();

    if m == "b" {
        let a: raw_slice = decode_first_param::<raw_slice>();
        __contract_ret(a.ptr(), a.len::<u8>());
    }

    if m == "a" {
        let a: raw_slice = decode_first_param::<raw_slice>();
        __contract_ret(a.ptr(), a.len::<u8>());
    }

    if m == "a" {
        let a: raw_slice = decode_first_param::<raw_slice>();
        __contract_ret(a.ptr(), a.len::<u8>());
    }

    if m == "a" {
        let a: raw_slice = decode_first_param::<raw_slice>();
        __contract_ret(a.ptr(), a.len::<u8>());
    }

    if m == "a" {
        let a: raw_slice = decode_first_param::<raw_slice>();
        __contract_ret(a.ptr(), a.len::<u8>());
    }

    if m == "a" {
        let a: raw_slice = decode_first_param::<raw_slice>();
        __contract_ret(a.ptr(), a.len::<u8>());
    }

    if m == "a" {
        let a: raw_slice = decode_first_param::<raw_slice>();
        __contract_ret(a.ptr(), a.len::<u8>());
    }

    if m == "a" {
        let a: raw_slice = decode_first_param::<raw_slice>();
        __contract_ret(a.ptr(), a.len::<u8>());
    }
}

contract;

abi MyContract {
    fn test_function_a() -> bool;
    fn test_function_b() -> bool;
    fn test_function_c() -> bool;
    fn test_function_d() -> bool;
    #[storage(read)]
    fn test_function_e() -> bool;
}

impl MyContract for Contract {
    fn test_function_a() -> bool {
        do_impure_stuff_a(true)
    }
    fn test_function_b() -> bool {
        do_impure_stuff_b()
    }
    fn test_function_c() -> bool {
        do_impure_stuff_c()
    }
    fn test_function_d() -> bool {
        do_impure_stuff_d()
    }
    #[storage(read)]
    fn test_function_e() -> bool {
        do_pure_stuff_e()
    }
}

// -------------------------------------------------------------------------------------------------

fn do_impure_stuff_a(choice: bool) -> bool {
    if choice {
        let s = do_more_impure_stuff_a();
        false
    } else {
        true
    }
}

struct S {
    a: u64,
}

fn do_more_impure_stuff_a() -> S {
    let a = read_storage_word();
    S { a }
}

// -------------------------------------------------------------------------------------------------

fn do_impure_stuff_b() -> bool {
    do_more_impure_stuff_b()
}

fn do_more_impure_stuff_b() -> bool {
    let a = read_storage_b256();
    true
}

// -------------------------------------------------------------------------------------------------

fn do_impure_stuff_c() -> bool {
    while true {
        do_more_impure_stuff_c();
    }
    true
}

fn do_more_impure_stuff_c() {
    write_storage_word();
}

// -------------------------------------------------------------------------------------------------

enum E {
    a: (),
    b: bool,
}

fn do_impure_stuff_d() -> bool {
    let a = E::b(do_more_impure_stuff_d());
    true
}

fn do_more_impure_stuff_d() -> bool {
    write_storage_b256();
    false
}

// -------------------------------------------------------------------------------------------------

#[storage(read)]
fn do_pure_stuff_e() -> bool {
    true
}

// -------------------------------------------------------------------------------------------------

const KEY: b256 = 0xfefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefe;

fn read_storage_word() -> u64 {
    asm (key: KEY, res, a, b, c) {
        move a b;
        srw res key;
        addi b c i1;
        res: u64
    }
}

fn read_storage_b256() -> b256 {
    let res = 0x0000000000000000000000000000000000000000000000000000000000000000;
    asm (key: KEY, buf: res, a, b, c) {
        modi a b i10;
        lt a b c;
        srwq buf key;
        and a b c;
    }
    res
}

fn write_storage_word() {
    asm (key: KEY, val: 42) {
        sww key val;
    }
}

fn write_storage_b256() {
    let val = 0xbabababababababababababababababababababababababababababababababa;
    asm (key: KEY, val: val) {
        swwq key val;
    }
}

// -------------------------------------------------------------------------------------------------

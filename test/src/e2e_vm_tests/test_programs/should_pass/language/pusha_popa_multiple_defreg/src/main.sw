contract;

abi IncorrectPushaPopa {
    #[storage(read)]
    fn incorrect_pusha_popa() -> ();
}

impl IncorrectPushaPopa for Contract {
    #[storage(read)]
    fn incorrect_pusha_popa() -> () {
        setup();
        ()
    }
}

#[storage(read)]
fn setup() -> () {
    let a: u64 = 1;
    let b: u64 = 1;
    let c: u64 = 1;
    //call a few times to avoid inline
    store_read();
    let r = asm(r, a: a, b: b, c: c, d: store_read()) {
        movi r i0;  
        add r a b;  // r = a + b = 2 
        add r r c;  // r = a + b + c = 3        c value is overwritten by store_read, so we get 2 instead
        add r r d;  // r = a + b + c + d = 3    d returns 0
        r
    };
    assert(r == 3);
    ()
}

#[storage(read)]
fn store_read() -> u64 {
    let a = asm(slot, a, b, c) {
        movi c i32;
        aloc c;
        move slot hp;
        srw a b slot i0;   // somehow make b allocate to $r3
        movi c i0;
        add a a slot;
        sub a a slot;
        add a a b;
        add a a c;
        a
    };
    a - a   // return 0 and make sure a is not dced
}

#[test]
fn incorrect_pusha_popa() -> () {
    let c = abi(IncorrectPushaPopa, CONTRACT_ID);
    c.incorrect_pusha_popa();
    ()
}

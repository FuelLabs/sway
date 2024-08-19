contract;

trait A {
    #[storage(write)]
    fn f(self) -> bool;
}

impl A for bool {
    #[storage(write)]
    fn f(self) -> bool {
        let _ = __state_store_word(b256::zero(), 0);
        true
    }
}

abi Abi {
    #[storage(read, write)]
    fn test() -> bool;
}

impl Abi for Contract {
    #[storage(read, write)]
    fn test() -> bool {
        g()
    }
}

#[storage(read)]
pub fn g() -> bool {
    true.f()
}

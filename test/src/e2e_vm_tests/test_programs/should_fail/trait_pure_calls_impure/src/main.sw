contract;

trait A {
    #[storage(read)]
    fn f(self) -> bool;
} {
    fn g(self) -> bool {
        self.f()
    }
}

struct S {}

impl A for S {
    #[cfg(experimental_aligned_and_dynamic_storage = false)]
    #[storage(read)]
    fn f(self) -> bool {
        let _ = __state_load_word(b256::zero());
        true
    }

    #[cfg(experimental_aligned_and_dynamic_storage = true)]
    #[storage(read)]
    fn f(self) -> bool {
        let _ = __state_load_word(b256::zero(), 0);
        true
    }
}

abi Abi {
    #[storage(read)]
    fn test() -> bool;
}

impl Abi for Contract {
    #[storage(read)]
    fn test() -> bool {
        S {}.g()
    }
}
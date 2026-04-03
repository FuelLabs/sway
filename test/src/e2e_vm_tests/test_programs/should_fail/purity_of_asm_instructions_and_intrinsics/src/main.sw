contract;

abi Abi {
    fn test();
}

struct S { }

impl S {
    #[cfg(experimental_aligned_and_dynamic_storage = false)]
    fn read_intrinsics(self) -> Self {
        let ptr = asm (p: 0) { p: raw_ptr };
        let _ = __state_load_word(b256::zero());
        let _ = __state_load_quad(b256::zero(), ptr, 1);

        self
    }

    #[cfg(experimental_aligned_and_dynamic_storage = true)]
    fn read_intrinsics(self) -> Self {
        let ptr = asm (p: 0) { p: raw_ptr };
        let _ = __state_load_word(b256::zero(), 0);
        let _ = __state_load_quad(b256::zero(), ptr, 1);

        self
    }

    #[storage(read)]
    fn write_intrinsics(self) -> Self {
        let ptr = asm (p: 0) { p: raw_ptr };
        let _ = __state_store_word(b256::zero(), 0);
        let _ = __state_store_quad(b256::zero(), ptr, 1);

        self
    }

    #[storage(read)]
    fn clear_intrinsic(self) -> Self {
        let _ = __state_clear(b256::zero(), 1);

        self
    }
}

impl Abi for Contract {
    fn test() {
        read_asm_instructions();
        write_asm_instructions();
        clear_asm_instruction();

        let s = S {};
        let _ = s.read_intrinsics();
        let _ = s.write_intrinsics();
        let _ = s.clear_intrinsic();
    }
}

fn read_asm_instructions() {
    asm(r1, r2, r3: 0) {
        srw r1 r2 r3 i0;
    }

    asm(r1: 0, r2, r3: 0, r4: 0) {
        srwq r1 r2 r3 r4;
    }
}

#[storage(read)]
fn write_asm_instructions() {
    asm(r1: 0, r2, r3: 0) {
        sww r1 r2 r3;
    }

    asm(r1: 0, r2, r3: 0, r4: 0) {
        swwq r1 r2 r3 r4;
    }
}

#[storage(read)]
fn clear_asm_instruction() {
    asm(r1: 0, r2, r3: 0) {
        scwq r1 r2 r3;
    }
}
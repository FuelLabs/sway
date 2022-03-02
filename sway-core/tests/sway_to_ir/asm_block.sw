script;

fn get_global_gas() -> u64 {
    // This is just reading the $ggas register.  Need to make sure that's what the codegen does.
    asm() {
        ggas
    }
}

fn main() -> u64 {
    get_global_gas();
    asm(r1) {
        bhei r1;
        r1: u64
    }
}

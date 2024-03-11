contract;

abi ConstantId {
    const ID: u32 = 0;
}

#[fallback]
fn fallback_1() {
}

#[fallback]
fn fallback_2() {
}

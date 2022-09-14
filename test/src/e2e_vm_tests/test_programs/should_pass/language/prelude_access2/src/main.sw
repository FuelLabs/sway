script;

struct A {
    f1: Address,
    f2: ContractId,
    f3: Identity,
    f4: Vec<u8>,
}

fn foo() {
    assert(true);
    require(true, 0);
    revert(0);
}

fn main() {
}

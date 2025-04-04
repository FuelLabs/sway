script;

abi Abi1 {
    fn foo() -> b256;
}

abi Abi2{
    fn bar() -> u64;
}

pub fn main() {
    let contract_1 = abi(Abi1, b256::min());
    const INVALID_CONST: b256 = contract_1.foo();
    let contract_2 = abi(Abi2, INVALID_CONST);
    let _ = contract_2.bar();
}

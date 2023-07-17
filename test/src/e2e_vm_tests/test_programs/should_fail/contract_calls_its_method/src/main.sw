contract;

abi ContractCallsItsOwnMethod {
    fn method1();
    fn method2();
}

impl ContractCallsItsOwnMethod for Contract {
    fn method1() {
    }
    fn method2() {
        Self::method1()
    }
}

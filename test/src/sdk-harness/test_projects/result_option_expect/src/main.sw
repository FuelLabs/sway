contract;

abi MyContract {
    fn option_test_should_revert();
    fn option_test_should_not_revert();
    fn result_test_should_revert();
    fn result_test_should_not_revert();
}

enum Error {
    Test: (),
}

impl AbiEncode for Error {
    fn is_encode_trivial() -> bool { false }
    fn abi_encode(self, buffer: Buffer) -> Buffer { buffer }
}

impl MyContract for Contract {
    fn option_test_should_revert() {
        let op: Option<u64> = None;
        let _ = op.expect(Error::Test);
    }

    fn option_test_should_not_revert() {
        let op = Some(0);
        let _ = op.expect(Error::Test);
    }

    fn result_test_should_revert() {
        let res: Result<u64, u64> = Err(0);
        let _ = res.expect(Error::Test);
    }

    fn result_test_should_not_revert() {
        let res: Result<u64, u64> = Ok(0);
        let _ = res.expect(Error::Test);
    }
}

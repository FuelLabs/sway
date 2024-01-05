contract;

abi MyContract {
    fn option_test(should_revert: bool);
    fn result_test(should_revert: bool);
}

enum Error {
    Test: (),
}

impl MyContract for Contract {
    fn option_test(should_revert: bool) {
        if should_revert {
            let op: Option<u64> = None;
            let _ = op.expect(Error::Test);
        } else {
            let op = Some(0);
            let _ = op.expect(Error::Test);
        }
    }

    fn result_test(should_revert: bool) {
        if should_revert {
            let res: Result<u64, u64> = Err(0);
            let _ = res.expect(Error::Test);
        } else {
            let res: Result<u64, u64> = Ok(0);
            let _ = res.expect(Error::Test);
        }
    }
}

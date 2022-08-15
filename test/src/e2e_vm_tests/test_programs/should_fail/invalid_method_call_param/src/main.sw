script;

abi MyContract {
    fn test_function();
}

fn main() {
    let caller = abi(MyContract, 0x0000000000000000000000000000000000000000000000000000000000000000);
    caller.test_function {
        wrong_call_param: 0 // Invalid call parameter
    } ();
}
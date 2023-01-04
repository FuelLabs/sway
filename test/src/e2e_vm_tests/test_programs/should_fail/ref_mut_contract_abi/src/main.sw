contract;

abi MyAbi {
    fn foo(ref mut x: u64);
    fn test_function() -> bool;
}

impl MyAbi for Contract {
    fn foo(ref mut x: u64) {

    }

    fn test_function() -> bool {
        bar(ref Vec::new());
        true
    }
}

fn bar(ref mut vec: Vec<u64>) {
    vec.push(1);
}


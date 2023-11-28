script;

trait AbiEncode {
    fn abi_encode(self);
}

struct S {

}

fn main() {
    let s = S{};
    s.abi_encode();
}

// check: script
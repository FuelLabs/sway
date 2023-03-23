script;

mod lib;

fn noop1(x: lib::MyIdentity1) -> u64 {
    if let lib::MyIdentity2::A(x) = x {
        x
    } else {
        0
    }
}

fn main() {
    let _a = noop1(lib::MyEnum1::A(12));
}

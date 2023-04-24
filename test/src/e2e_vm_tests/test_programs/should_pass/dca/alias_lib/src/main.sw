script;

mod lib;

fn noop1(x: lib::MyIdentity1) -> u64 {
    if let lib::MyIdentity2::A(x) = x {
        x
    } else {
        0
    }
}

fn noop2(x: lib::MyStruct2) -> u64 {
    x.A
}

fn main() {
    let _a = noop1(lib::MyEnum1::A(12));
    let _b = noop2(lib::MyStruct1 { A: 12 });
}

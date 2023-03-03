script;

#[allow(foo)]
fn f1() {}

#[allow]
fn f2() {}

#[allow(dead_code, dead_code)]
fn f3() {}

fn main() {
    f1();
    f2();
    f3();
}

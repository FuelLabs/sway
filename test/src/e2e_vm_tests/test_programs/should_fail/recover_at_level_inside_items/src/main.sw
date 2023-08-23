script;

trait T1 {
    garbage
    fn f1(self);
    / more garbage
    fn f2(self);
    fn f3(self)
    fn f4(self);
}

struct S1 {

}

impl T1 for S1 {
    fn f1(self) {}
    fn f2(self) {}
    fn f3(self) {}
    fn f4(self) {}
}

fn main() {
    let s1 = S1 {};
    s1.f1();
    s1.f2();
    s1.f3();
    s1.f4();
}

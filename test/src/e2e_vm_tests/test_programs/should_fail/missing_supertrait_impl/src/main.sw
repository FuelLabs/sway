library;

trait A {
    fn a();
}

trait B: A {
    fn b();
}

trait C {
    fn c();
}

trait D: C + B  {
    fn d();
}

struct X { x: u64 }
impl A for X {
    fn a() { }
}
impl B for X {
    fn b() { }
}
impl C for X {
    fn c() { }
}
impl D for X {
    fn d() { }
}

struct Y { y: u64 }
// This code shouldn't compile because the implementation of `A` below is completely missing:
//impl A for Y {
//    fn a() { }
//}
impl B for Y {
    fn b() { }
}
impl C for Y {
    fn c() { }
}
impl D for Y {
    fn d() { }
}

script;

#[deprecated]
struct A {
    #[deprecated]
    a: u64,
    b: u64,
}

impl A {
    #[deprecated]
    fn fun(self) {}
}

#[deprecated]
enum B {
    A: (),
    #[deprecated]
    B: (),
}


#[deprecated]
fn depr(_a: A) {}

fn fun(_a: A) {}

#[deprecated]
fn depr_b(_b: B) {}

// TODO: support for traits, abis and their methods
pub fn main() {
    let a = A { a: 0, b: 0 };
    let b = B::A;
    depr(a);
    depr(A { a: 0, b: 0  });
    depr_b(b);
    depr_b(B::A);
    fun(a);
    fun(A { a: 0, b: 0 });
    let _ = a.a;
    let _ = a.b;
    let _ = B::A;
    let _ = B::B;
    a.fun();
}

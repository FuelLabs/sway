script;

trait From2<T> {
    fn into2(self) -> T;
}

struct S1 {
    s1: u64
}

struct S2 {
    s2: u64
}

impl From2<S1> for u64 {
    fn into2(self) -> S1 {
        S1{s1: self}
    }
}

impl From2<S2> for u64 {
    fn into2(self) -> S2 {
        S2{s2: self}
    }
}

fn main() -> bool {    
    let _s1: S1  = <u64 as From2<S1>>::into2(42);

    let _s2: S2  = <u64 as From2<S2>>::into2(42);

    true   
}
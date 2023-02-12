script;
use core::ops::Ord;

fn main() -> bool {
    let x = Option::Some(10); 
    let y = Option::Some(true);

    let z = Option::<u32>::Some(10);
    let z = Option::Some::<u32>(10);

    let n = Option::<u32>::None;
    let n = Option::None::<u32>;

 //   x == Option::Some(10)
   true
}

enum Option<T> {
    Some: T,
    None: ()
}


/*
TODO: make this work
impl Ord<T> for Option<T> where T: Ord {
    fn gt(self, other: Self) -> bool {
        asm(r1: self, r2: other, r3) {
            gt r3 r1 r2;
            r3: bool
        }
    }
    fn lt(self, other: Self) -> bool {
        asm(r1: self, r2: other, r3) {
            lt r3 r1 r2;
            r3: bool
        }
    }
    fn eq(self, other: Self) -> bool {
        asm(r1: self, r2: other, r3) {
            eq r3 r1 r2;
            r3: bool
        }
    }
}
*/

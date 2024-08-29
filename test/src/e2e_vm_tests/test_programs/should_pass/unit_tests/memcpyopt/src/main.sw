script;

pub fn main() {

}

pub struct S {
    ptr: raw_ptr,
}

impl S {
    #[inline(never)]
    pub fn new() -> Self {
        let ptr = asm(size) {
            movi size i16;
            aloc size;
            hp: raw_ptr
        };
        S {ptr: ptr}
    }

    #[inline(never)]
    pub fn set(self, idx: u64, val: u64) -> () {
        assert(idx < 2);
        let ptr = self.ptr.add::<u64>(idx);
        ptr.write::<u64>(val);
    }

    #[inline(never)]
    pub fn get(self, idx: u64) -> u64 {
        assert(idx < 2);
        let ptr = self.ptr.add::<u64>(idx);
        ptr.read::<u64>()
    }
}

#[inline(never)]
fn side_effect(ref mut a: [S;2]) -> u64 {
    let mut b = S::new();
    b.set(0,5);
    b.set(1,6);
    a[1] = b;
    1
}

#[test]
fn test() -> () {
    let mut v1 = S::new();
    let mut v2 = S::new();
    v1.set(0,1);
    v1.set(1,2);
    v2.set(0,3);
    v2.set(1,4);
    let mut a: [S;2] = [v1, v2];
    let b = a[1].get(side_effect(a)); //ir is shown for this line
    assert(b == 4);
    ()
}

#[test]
fn foo() -> u64 {
   let mut x = 43;
   x = x;
   assert(x == 43);
   x
}

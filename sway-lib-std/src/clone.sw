//! The clone trait, for explicit duplication.
library;

/// A common trait for the ability to explicitly duplicate an object.
pub trait Clone {
    /// Clone self into a new value of the same type.
    fn clone(self) -> Self;
}

impl Clone for u8 {
    fn clone(self) -> Self {
        self
    }
}

impl Clone for u16 {
    fn clone(self) -> Self {
        self
    }
}

impl Clone for u32 {
    fn clone(self) -> Self {
        self
    }
}

impl Clone for u64 {
    fn clone(self) -> Self {
        self
    }
}

impl Clone for u256 {
    fn clone(self) -> Self {
        self
    }
}

#[cfg(experimental_const_generics = true)]
impl<T, const N: u64> Clone for [T; N]
where
    T: Clone,
{
    fn clone(self) -> Self {
        let first: T = *__elem_at(&self, 0);
        let mut new_array = [first.clone(); N];

        let mut i = 1;
        while __lt(i, N) {
            let item: T = *__elem_at(&self, i);
            let new_item: &mut T = __elem_at(&mut new_array, i);
            *new_item = item.clone();
            i = __add(i, 1);
        }

        new_array
    }
}

#[cfg(experimental_const_generics = true)]
impl<const N: u64> Clone for str[N] {
    fn clone(self) -> Self {
        let new = [0u8; N];
        asm(dest: new, len: N, src: self) {
            mcp dest src len;
            dest: str[N]
        }
    }
}

#[cfg(experimental_const_generics = true)]
#[test]
fn ok_string_array_clone() {
    use ::assert::*;
    use ::debug::*;

    let a = __to_str_array("abc");
    let b = a.clone();

    let _ = __dbg((a, b));

    assert(a == __to_str_array("abc"));
    assert(b == __to_str_array("abc"));
    assert(a == b);
}

#[cfg(experimental_const_generics = true)]
#[test]
fn ok_array_clone() {
    use ::assert::*;
    use ::debug::*;

    let a = [1, 2, 3];
    let b = a.clone();

    let _ = __dbg((a, b));

    assert(a == [1, 2, 3]);
    assert(b == [1, 2, 3]);
    assert(a == b);
}

library;

use ::data_structures::{SomeEnum, SomeStruct};
use core::ops::Eq;

#[cfg(experimental_partial_eq = false)]
impl Eq for SomeEnum<u32> {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (SomeEnum::A(val), SomeEnum::A(other_val)) => {
                val == other_val
            },
        }
    }
}
#[cfg(experimental_partial_eq = true)]
impl PartialEq for SomeEnum<u32> {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (SomeEnum::A(val), SomeEnum::A(other_val)) => {
                val == other_val
            },
        }
    }
}
#[cfg(experimental_partial_eq = true)]
impl Eq for SomeEnum<u32> {}

#[cfg(experimental_partial_eq = false)]
impl Eq for SomeStruct<u32> {
    fn eq(self, other: Self) -> bool {
        self.a == other.a
    }
}
#[cfg(experimental_partial_eq = true)]
impl PartialEq for SomeStruct<u32> {
    fn eq(self, other: Self) -> bool {
        self.a == other.a
    }
}
#[cfg(experimental_partial_eq = true)]
impl Eq for SomeStruct<u32> {}

#[cfg(experimental_partial_eq = true)]
impl Eq for Vec<SomeStruct<u32>> {}

#[cfg(experimental_partial_eq = true)]
impl Eq for Vec<SomeEnum<u32>> {
}

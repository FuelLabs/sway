library;
use std::convert::{From, Into};
use std::primitive_conversions::u64::*;

pub struct U128_2 {
    upper: u64,
    lower: u64,
}

impl U128_2 {
    fn associated_fn(){}
}
impl From<u8> for U128_2 {
    fn from(val: u8) -> Self {
        Self {
            upper: 0,
            lower: val.into(),
        }
    }
}

pub fn test_u128_from_u8() {
    let u8_1: u8 = 0u8;
    let u8_2: u8 = 255u8;

    //Symbol U128_2 is found here
    U128_2::associated_fn();

    //Symbol U128_2 is found inside fully qualified path
    let _u128_1 = <U128_2 as From<u8>>::from(u8_1);
    let _u128_2 = <U128_2 as From<u8>>::from(u8_2);
}
library utils;

use core::ops::Eq;
use std::logging::log;
use std::revert::revert;

// pub fn vec_from(vals: [u32; 3]) -> Vec<u32> {
//     let mut vec = Vec::new();
//     vec.push(vals[0]);
//     vec.push(vals[1]);
//     vec.push(vals[2]);
//     vec
// }

// pub fn get_an_option<T>() -> Option<T> {
//     Option::None
// }

pub fn assert_eq_2<T>(v1: T) where T: Eq {

}

// pub fn assert_eq<T>(v1: T, v2: T) where T: Eq {
//     if (v1 != v2) {
//         log(v1);
//         log(v2);
//         revert(0xffff_ffff_ffff_0004);
//     }
// }

script;

use std::assert::assert;
use std::result::*;
use std::u128::*;
use std::ufp64::*;
use std::math::*;
use std::logging::log;

fn main() -> bool {
    let one = ~UFP64::from_uint(1);
    let mut res = ~UFP64::exp(one);
    assert(res.value == 11674811894);

    let two = ~UFP64::from_uint(2);
    res = ~UFP64::exp(two);
    assert(res.value == 31700949040);

    let four = ~UFP64::from_uint(4);
    res = ~UFP64::exp(four);
    log(res.value);
    assert(res.value == 222506572928);

    let seven = ~UFP64::from_uint(7);
    res = ~UFP64::exp(seven);
    log(res.value);
    assert(res.value == 2819944203710);

    let ten = ~UFP64::from_uint(10);
    res = ~UFP64::exp(ten);
    log(res.value);
    assert(res.value == 20833521987056);
    
    true
}

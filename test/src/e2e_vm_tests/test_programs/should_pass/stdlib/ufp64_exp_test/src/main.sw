script;

use std::{
    assert::assert,
    ufp64::UFP64,
};

fn main() -> bool {
    let one = ~UFP64::from_uint(1);
    let mut res = ~UFP64::exp(one);
    assert(res.value == 11674811894);

    let two = ~UFP64::from_uint(2);
    res = ~UFP64::exp(two);
    assert(res.value == 31700949040);

    let four = ~UFP64::from_uint(4);
    res = ~UFP64::exp(four);
    assert(res.value == 222506572928);

    let seven = ~UFP64::from_uint(7);
    res = ~UFP64::exp(seven);
    assert(res.value == 2819944203710);

    let ten = ~UFP64::from_uint(10);
    res = ~UFP64::exp(ten);
    assert(res.value == 20833521987056);
    
    true
}

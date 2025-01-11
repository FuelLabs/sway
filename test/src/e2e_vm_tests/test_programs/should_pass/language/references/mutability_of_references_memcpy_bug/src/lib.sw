library;

#[inline(never)]
pub fn unit(b: u256) -> u256 {
    b
}

#[inline(never)]
pub fn weird(_b: u256) {
    let mut b = _b; // b = _b = 2
    
    log(b);

    let mut b_tmp = b; // b_tmp = b = 2
    
    b = 0x03u256; // b = a = 1
    
    assert(unit(b_tmp) == 0x02u256);
}

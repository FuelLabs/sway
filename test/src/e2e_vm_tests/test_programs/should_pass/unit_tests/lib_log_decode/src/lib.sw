library;

use std::flags::disable_panic_on_overflow;

#[test]
fn test_fn() {
	let a = 10;
	log(a);
	let b = 30;
	log(b);
	assert_eq(a, 10)
}

#[test]
fn math_u16_overflow_mul() {
    disable_panic_on_overflow();
    
    let a = (u16::max() / 2 ) + 1;
    let b = a * 2;

    log(b);
    assert(b == 0_u16)
}

#[test]
fn math_u32_overflow_mul() {
    disable_panic_on_overflow();
    
    let a = (u32::max() / 2 ) + 1;
    let b = a * 2;
    
    log(b);
    assert(b == 0_u32)
}
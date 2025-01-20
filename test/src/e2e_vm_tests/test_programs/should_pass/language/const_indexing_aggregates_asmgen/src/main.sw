script;

pub fn main() -> bool {
    true
}

#[test]
fn side_effect_register_not_cleared() -> u64 {
    let a = asm(a, b) {
        movi b i16;     // b = 16
        aloc b;         // buf1 = [0;16]
        movi b i0;      // b = 0
        sw hp b i0;     // buf1[0:8] = b = 0
        movi a i0;      // a = 0
        add a hp a;     // a = &buf1
        movi b i16;     // b = 16
        aloc b;         // buf2 = [0;16]
        movi b i1;      // b = 1
        sw hp b i0;     // buf2[0:8] = b = 1
        lw a a i0;      // expected : a = buf1[0:8] = 0         real : a = buf2[0:8] = 1
        a
    };
    assert(a == 0);
    a
}

#[test]
fn incorrect_bailout() -> u64 {
    let a = asm(a, b) {
        movi a i32;		// a = 32
        aloc a;			// hp = buf[0;32]

        movi a i1;		// a = 1
        addi b hp i23;	        // b = &buf[23]
        sb b a i0;		// buf[23] = a = 1
        addi a hp i9;	        // a = &buf[9]
        addi b hp i1;	        // b = &buf[1]
        sb b a i7;		// buf[1:9] = a = &buf[9] avoid using sw, which is buggy itself
        srli a a i8;
        sb b a i6;
        srli a a i8;
        sb b a i5;
        srli a a i8;
        sb b a i4;
        srli a a i8;
        sb b a i3;
        srli a a i8;
        sb b a i2;
        srli a a i8;
        sb b a i1;
        srli a a i8;
        sb b a i0;

        addi a hp i1;	        // a = &buf[1]
        lw a a i0;		// a = buf[1:9] = &buf[9]
        addi a a i15;	        // expected : a = &buf[24]              real : a = &buf[16]
        lw a a i0;		// expected : a = buf[24:32] = 0        real : a = bug[16:24] = 1
        a: u64
    };
    assert(a == 0);
    a
}

#[test]
fn sw_missing_alignment_check() -> u64 {
    let a = asm(a, b) {
        movi a i24;     // a = 24
        aloc a;         // hp = buf[0;24]

        movi a i1;      // a = 1
        sb hp a i16;    // buf[16] = a = 1

        movi a i0;      // a = 0
        addi b hp i1;   // b = &buf[1]
        sw b a i1;      // expected : buf[9:17] = a = 0     real : buf[8:16] = a = 0

        lb a hp i16;    // a = &buf[16]
        a: u64
    };

    assert(a == 0);
    a
}

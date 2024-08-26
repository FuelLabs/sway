library;

pub fn test() {
    asm() {
    };

    asm(r1: 5, r2: 5, r3, r4) {
        add r3 r1 r2;
        add r4 r2 r2;
    };

    // These cases prove that https://github.com/FuelLabs/sway/issues/6354 is fixed.
    poke(asm() { });

    let arg_u8 = 11u8;

    poke(asm(a: arg_u8, b: arg_u8, res) {
        add res a b;
    });

    let x = asm(a: arg_u8, b: arg_u8, res) {
        add res a b;
    };
    
    poke(x);

    // Return the unit result of ASM block as function result.
    asm(r1: 5, r2: 5, r3) {
        add r3 r1 r2;
    }
}

#[inline(never)]
fn poke<T>(_x: T) { }

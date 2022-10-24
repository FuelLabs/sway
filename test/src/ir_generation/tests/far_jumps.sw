script;

fn main() -> bool {
    a() && b() && b()
}

fn a() -> bool {
    asm (res) {
        // Introduce a 'NOP blob' to push the address of b() out into the danger zone.
        blob i262200;
        movi res i1;
        res: bool
    }
}

fn t() -> bool {
    asm() {
        one: bool
    }
}

fn b() -> bool {
    // Create complex control flow.
    while t() {
        t() && t();
    }
    t()
}

// ::check-ir::

// check: fn main() -> bool
// check: call a$()
// check: call b$()
// check: call b$()

// The blob must be before b().
// check: blob

// We want both `cbr`s and `br`s in b().
// check: fn b$()
// check: call t$()
// check: cbr
// check: br

// ::check-asm::

// regex: REG=\$r\d+

// We want to see the blob and then no JNZI or JNEI.
// check: blob
// not: jnzi
// not: jnei

// The maximum offset available with 18 bits is 262144.  So JNZI can't jump to an address larger
// than this.  Below those larger addresses are stored in the data section, and they're matched with
// i2622xx (specifically i2622$()), giving a bit of leeway in case slight changes to code gen move
// things around slightly.

// Calling t() - save $reta from the data section, but we're still able to call directly with JI.
// check: lw   $$$$reta data_0
// check: ji   i2622$()

// Some local control flow using addresses from the data section.
// check: lw   $$$$tmp data_1
// nextln: jne  $REG $$zero $$$$tmp

// Calling t() again.
// check: lw   $$$$reta data_2
// check: ji   i2622$()

// check: lw   $$$$tmp data_3
// nextln: jne  $REG $$zero $$$$tmp

// check: lw   $$$$reta data_4
// check: ji   i2622$()

// check: lw   $$$$reta data_5
// check: ji   i2622$()

// check: data_0 .word 2622$()
// check: data_1 .word 2622$()
// check: data_2 .word 2622$()
// check: data_3 .word 2622$()
// check: data_4 .word 2622$()
// check: data_5 .word 2622$()

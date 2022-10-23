contract;

struct S {
    x: u64,
    y: b256,
}

abi Test {
    fn get_u64(val: u64) -> u64;
    fn get_b256(val: b256) -> b256;
    fn get_s(val1: u64, val2: b256) -> S;
}

impl Test for Contract {
    fn get_u64(val: u64) -> u64 {
        val
    }

    fn get_b256(val: b256) -> b256 {
        val
    }

    fn get_s(val1: u64, val2: b256) -> S {
        S {
            x: val1,
            y: val2,
        }
    }
}

// ::check-ir::

// check: contract {
// check: fn get_u64<9890aef4>($ID $MD: u64) -> u64
// check: fn get_b256<42123b96>($ID $MD: b256) -> b256
// check: fn get_s<fc62d029>($ID $MD: u64, $ID $MD: b256) -> { u64, b256 }

// ::check-asm::

// regex: REG=\$r\d+

// Get the called selector.
// check: lw   $(sel_reg=$REG) $$fp i73

// Check selector at data_2 2559618804 (0x9890aef4)
// check: lw   $(data_2_reg=$REG) data_2
// check: eq   $(eq_reg=$REG) $sel_reg $data_2_reg
// check: jnzi $eq_reg

// Check selector at data_3 1108491158 (0x42123b96)
// check: lw   $(data_3_reg=$REG) data_3
// check: eq   $(eq_reg=$REG) $sel_reg $data_3_reg
// check: jnzi $eq_reg

// Check selector at data_4 4234334249 (0xfc62d029)
// check: lw   $(data_4_reg=$REG) data_4
// check: eq   $(eq_reg=$REG) $sel_reg $data_4_reg
// check: jnzi $eq_reg

// Revert on no match.
// check: movi $$$$tmp i123
// check: rvrt $$$$tmp

// Each function will read from $fp for the args.
// check: lw   $REG $$fp i74

// check: .data:
// check: data_2 .word 2559618804
// check: data_3 .word 1108491158
// check: data_4 .word 4234334249

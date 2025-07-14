// target-fuelvm

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
// check: fn get_b256<42123b96>($ID: ptr b256) -> ptr b256,
// check: fn get_s<fc62d029>($ID $MD: u64, $ID: ptr b256) -> ptr { u64, b256 }
// check: fn get_u64<9890aef4>($ID $MD: u64) -> u64

// ::check-asm::

// regex: REG=\$r\d+
// regex: ID=[_[:alpha:]][_0-9[:alpha:]]*
// regex: IMM=i\d+

// Get the called selector.
// check: lw   $(sel_reg=$REG) $$fp i73

// Check selector for get_b256()
// check: load $(get_b256_sel_reg=$REG) $(get_b256_sel_data=$ID)
// check: eq   $(eq_reg=$REG) $sel_reg $get_b256_sel_reg
// check: jnzf $eq_reg $$zero $IMM

// Check selector for get_s()
// check: load $(get_s_sel_reg=$REG) $(get_s_sel_data=$ID)
// check: eq   $(eq_reg=$REG) $sel_reg $get_s_sel_reg
// check: jnzf $eq_reg $$zero $IMM

// Check selector for get_u64()
// check: load $(get_u64_sel_reg=$REG) $(get_u64_sel_data=$ID)
// check: eq   $(eq_reg=$REG) $sel_reg $get_u64_sel_reg
// check: jnzf $eq_reg $$zero $IMM

// Revert on no match.
// check: movi $$$$tmp i123
// check: rvrt $$$$tmp

// Each function will read from $fp for the args.
// check: lw   $REG $$fp i74

// check: .data:
// check: $get_b256_sel_data .halfword 1108491158
// check: $get_s_sel_data .halfword 4234334249
// check: $get_u64_sel_data .halfword 2559618804

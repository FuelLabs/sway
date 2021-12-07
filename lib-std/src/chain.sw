library chain;
dep chain/auth;

// When generics land, these will be generic.
pub fn log_u64(val: u64) {
    asm(r1: val) {
        log r1 zero zero zero;
    }
}

pub fn log_u32(val: u32) {
    asm(r1: val) {
        log r1 zero zero zero;
    }
}

pub fn log_u16(val: u16) {
    asm(r1: val) {
        log r1 zero zero zero;
    }
}

pub fn log_u8(val: u8) {
    asm(r1: val) {
        log r1 zero zero zero;
    }
}

/// Context-dependent:
/// will panic if used in a predicate
/// will revert if used in a contract
pub fn panic(code: u64) {
    asm(r1: code) {
        rvrt r1;
    }
}


// The transaction starts at:
// 32 + MAX_INPUTS*(32+8).
// Everything when serialized is padded to word length, so if there are 4 fields preceding script data then it's 4 words.
// Couldn't you just add script length (at a constant offset from the start of the transaction,
// also constant) to the start of the script (also a constant)? Adding seems more intuitive then subtracting.
//
// start: SCRIPT_LENGTH + SCRIPT_START
// end:   start + SCRIPT_DATA_LEN
// where:
// SCRIPT_DATA_LEN = mem[TX_START + 4 words, 32 bytes)         // 352 + 32       == 384
// SCRIPT_LENGTH   = mem[TX_START +  3 words, 24 bytes] as u16 // 352 + 24       == 376
// TX_START        = 32 + MAX_INPUTS * (32 + 8)                // 32 + 8 * (40)  == 352 
// MAX_INPUTS      = 8
// SCRIPT_START    = $is 

// TODO some safety checks on the input data? We are going to assume it is the right type for now.
pub fn get_script_data<T>() -> T{
    asm(script_data_len, to_return, script_data_ptr, script_len, script_len_ptr: 376, script_data_len_ptr: 384) {
        lw script_len script_len_ptr;
        lw script_data_len script_data_len_ptr;
        // get the start of the script data
        // script_len + script_start
        add script_data_ptr script_len is;
        // allocate memory to copy script data into
        mv to_return sp;
        cfe script_data_len;
        // copy script data into above buffer
        mcp to_return script_data_ptr script_data_len;
        to_return: T
    }
}

/// Assert that a value is true
pub fn assert(a: bool) {
    if !a {
        panic(0);
    } else {
        ()
    }
}

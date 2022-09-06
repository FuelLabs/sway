contract;

use std::context::{*, call_frames::*, registers::*};

abi Registers {
    /*
    fn get_program_counter() -> raw_ptr;
    fn get_stack_start_ptr() -> raw_ptr;
    fn get_stack_ptr() -> raw_ptr;
    fn get_frame_ptr() -> raw_ptr;
    fn get_heap_ptr() -> raw_ptr;
    fn get_instrs_start() -> raw_ptr;
    */
    fn get_overflow() -> u64;
    fn get_error() -> u64;
    fn get_global_gas() -> u64;
    fn get_context_gas() -> u64;
    fn get_balance() -> u64;
    fn get_return_value() -> u64;
    fn get_return_length() -> u64;
    fn get_flags() -> u64;
}

impl Registers for Contract {
    /*
    fn get_program_counter() -> raw_ptr {
        program_counter()
    }

    fn get_stack_start_ptr() -> raw_ptr {
        stack_start_ptr()
    }

    fn get_stack_ptr() -> raw_ptr {
        stack_ptr()
    }

    fn get_frame_ptr() -> raw_ptr {
        frame_ptr()
    }

    fn get_heap_ptr() -> raw_ptr {
        heap_ptr()
    }

    fn get_instrs_start() -> raw_ptr {
        instrs_start()
    }
    */

    fn get_error() -> u64 {
        error()
    }

    fn get_global_gas() -> u64 {
        global_gas()
    }

    fn get_context_gas() -> u64 {
        context_gas()
    }

    fn get_balance() -> u64 {
        msg_amount()
    }

    fn get_overflow() -> u64 {
        overflow()
    }

    fn get_return_value() -> u64 {
        return_value()
    }

    fn get_return_length() -> u64 {
        return_length()
    }

    fn get_flags() -> u64 {
        flags()
    }
}

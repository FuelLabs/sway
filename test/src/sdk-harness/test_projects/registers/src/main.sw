contract;

use std::context::{*, call_frames::*, registers::*};

abi Registers {
    fn get_overflow() -> u64;
    fn get_program_counter() -> u64;
    fn get_stack_start_ptr() -> u64;
    fn get_stack_ptr() -> u64;
    fn get_frame_ptr() -> u64;
    fn get_heap_ptr() -> u64;
    fn get_error() -> u64;
    fn get_global_gas() -> u64;
    fn get_context_gas() -> u64;
    fn get_balance() -> u64;
    fn get_instrs_start() -> u64;
    fn get_return_value() -> u64;
    fn get_return_length() -> u64;
    fn get_flags() -> u64;
}

impl Registers for Contract {
    fn get_overflow() -> u64 {
        overflow()
    }

    fn get_program_counter() -> u64 {
        program_counter().addr()
    }

    fn get_stack_start_ptr() -> u64 {
        stack_start_ptr().addr()
    }

    fn get_stack_ptr() -> u64 {
        stack_ptr().addr()
    }

    fn get_frame_ptr() -> u64 {
        frame_ptr().addr()
    }

    fn get_heap_ptr() -> u64 {
        heap_ptr().addr()
    }

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

    fn get_instrs_start() -> u64 {
        instrs_start().addr()
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

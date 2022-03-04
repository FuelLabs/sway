contract;

use std::registers::*;

abi Registers {
    fn get_overflow(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64;
    fn get_program_counter(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64;
    fn get_stack_start_ptr(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64;
    fn get_stack_ptr(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64;
    fn get_frame_ptr(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64;
    fn get_heap_ptr(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64;
    fn get_error(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64;
    fn get_global_gas(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64;
    fn get_context_gas(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64;
    fn get_balance(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64;
    fn get_instrs_start(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64;
    fn get_return_value(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64;
    fn get_return_length(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64;
    fn get_flags(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64;
}

impl Registers for Contract {
    fn get_overflow(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64 {
        overflow()
    }

    fn get_program_counter(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64 {
        program_counter()
    }

    fn get_stack_start_ptr(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64 {
        stack_start_ptr()
    }

    fn get_stack_ptr(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64 {
        stack_ptr()
    }

    fn get_frame_ptr(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64 {
        frame_ptr()
    }

    fn get_heap_ptr(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64 {
        heap_ptr()
    }

    fn get_error(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64 {
        error()
    }

    fn get_global_gas(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64 {
        global_gas()
    }

    fn get_context_gas(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64 {
        context_gas()
    }

    fn get_balance(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64 {
        balance()
    }

    fn get_instrs_start(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64 {
        instrs_start()
    }

    fn get_return_value(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64 {
        return_value()
    }

    fn get_return_length(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64 {
        return_length()
    }

    fn get_flags(gas_: u64, amount_: u64, color_: b256, input: ()) -> u64 {
        flags()
    }
}

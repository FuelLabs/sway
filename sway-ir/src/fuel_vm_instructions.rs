use crate::{Context, DebugWithContext, Register, Type, Value};

#[derive(Debug, Clone, DebugWithContext)]
pub enum FuelVmInstruction {
    /// Generate a unique integer value
    GetStorageKey,
    Gtf {
        index: Value,
        tx_field_id: u64,
    },
    /// Logs a value along with an identifier.
    Log {
        log_val: Value,
        log_ty: Type,
        log_id: Value,
    },
    /// Reads a special register in the VM.
    ReadRegister(Register),
    /// Revert VM execution.
    Revert(Value),
    /// - Sends a message to an output via the `smo` FuelVM instruction. The first operand must be
    /// a struct with the first field being a `B256` representing the recipient. The rest of the
    /// struct is the message data being sent.
    /// - Assumes the existence of an `OutputMessage` at `output_index`
    /// - `message_size`, `output_index`, and `coins` must be of type `U64`.
    Smo {
        recipient_and_message: Value,
        message_size: Value,
        output_index: Value,
        coins: Value,
    },
    /// Clears `number_of_slots` storage slots (`b256` each) starting at key `key`.
    StateClear {
        key: Value,
        number_of_slots: Value,
    },
    /// Reads `number_of_slots` slots (`b256` each) from storage starting at key `key` and stores
    /// them in memory starting at address `load_val`.
    StateLoadQuadWord {
        load_val: Value,
        key: Value,
        number_of_slots: Value,
    },
    /// Reads and returns single word from a storage slot.
    StateLoadWord(Value),
    /// Stores `number_of_slots` slots (`b256` each) starting at address `stored_val` in memory into
    /// storage starting at key `key`. `key` must be a `b256`.
    StateStoreQuadWord {
        stored_val: Value,
        key: Value,
        number_of_slots: Value,
    },
    /// Writes a single word to a storage slot. `key` must be a `b256` and the type of `stored_val`
    /// must be a `u64`.
    StateStoreWord {
        stored_val: Value,
        key: Value,
    },
}

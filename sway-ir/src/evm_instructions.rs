use crate::Value;

#[derive(Debug, Clone)]
pub enum EVMInstruction {
    /// Halts execution
    Stop,
    /// Addition operation
    Add,
    /// Multiplication operation
    Mul,
    /// Subtraction operation
    Sub,
    /// Integer division operation
    Div,
    /// Signed integer division operation (truncated)
    SDiv,
    /// Modulo remainder operation
    Mod,
    /// Signed modulo remainder operation
    SMod,
    /// Modulo addition operation
    AddMod,
    /// Modulo multiplication operation
    MulMod,
    /// Exponential operation
    Exp,
    /// Extend length of two’s complement signed integer
    SignExtend,
    /// Less-than comparison
    Lt,
    /// Greater-than comparison
    Gt,
    /// Signed less-than comparison
    SLt,
    /// Signed greater-than comparison
    SGt,
    /// Equality comparison
    Eq,
    /// Zero comparison
    IsZero,
    /// Bitwise AND operation
    And,
    /// Bitwise OR operation
    Or,
    /// Bitwise XOR operation
    Xor,
    /// Bitwise NOT operation
    Not,
    /// Retrieve single byte from word
    Byte,
    /// Left shift operation
    Shl,
    /// Right shift operation
    Shr,
    /// Arithmetic (signed) right shift operation
    Sar,
    /// Compute Keccak-256 hash
    SHA3,
    /// Get address of currently executing account
    Address,
    /// Get balance of the given account
    Balance,
    /// Get execution origination address
    Origin,
    /// Get caller address
    Caller,
    /// Get deposited value by the instruction/transaction responsible for this execution
    CallValue,
    /// Get input data of current environment
    CallDataLoad,
    /// Get size of input data in current environment
    CallDataSize,
    /// Copy input data in current environment to memory
    CallDataCopy,
    /// Get size of code running in current environment
    CodeSize,
    /// Copy code running in current environment to memory
    CodeCopy,
    /// Get price of gas in current environment
    GasPrice,
    /// Get size of an account’s code
    ExtCodeSize,
    /// Copy an account’s code to memory
    ExtCodeCopy,
    /// Get size of output data from the previous call from the current environment
    ReturnDataSize,
    /// Copy output data from the previous call to memory
    ReturnDataCopy,
    /// Get hash of an account’s code
    ExtCodeHash,
    /// Get the hash of one of the 256 most recent complete blocks
    BlockHash,
    /// Get the block’s beneficiary address
    Coinbase,
    /// Get the block’s timestamp
    Timestamp,
    /// Get the block’s number
    Number,
    /// Get the previous block’s RANDAO mix
    PrevRANDAO,
    /// Get the block’s gas limit
    GasLimit,
    /// Get the chain ID
    ChainId,
    /// Get balance of currently executing account
    SelfBalance,
    /// Get the base fee
    BaseFee,
    /// Remove item from stack
    Pop,
    /// Load word from memory
    MLoad,
    /// Save word to memory
    MStore,
    /// Save byte to memory
    MStore8,
    /// Load word from storage
    SLoad,
    /// Save word to storage
    SStore,
    /// Alter the program counter
    Jump,
    /// Conditionally alter the program counter
    JumpI,
    /// Get the value of the program counter prior to the increment corresponding to this instruction
    PC,
    /// Get the size of active memory in bytes
    MSize,
    /// Get the amount of available gas, including the corresponding reduction for the cost of this instruction
    Gas,
    /// Mark a valid destination for jumps
    JumpDest,
    /// Place 1 byte item on stack
    Push1(Value),
    /// Place 2 byte item on stack
    Push2(Value),
    /// Place 3 byte item on stack
    Push3(Value),
    /// Place 4 byte item on stack
    Push4(Value),
    /// Place 5 byte item on stack
    Push5(Value),
    /// Place 6 byte item on stack
    Push6(Value),
    /// Place 7 byte item on stack
    Push7(Value),
    /// Place 8 byte item on stack
    Push8(Value),
    /// Place 9 byte item on stack
    Push9(Value),
    /// Place 10 byte item on stack
    Push10(Value),
    /// Place 11 byte item on stack
    Push11(Value),
    /// Place 12 byte item on stack
    Push12(Value),
    /// Place 13 byte item on stack
    Push13(Value),
    /// Place 14 byte item on stack
    Push14(Value),
    /// Place 15 byte item on stack
    Push15(Value),
    /// Place 16 byte item on stack
    Push16(Value),
    /// Place 17 byte item on stack
    Push17(Value),
    /// Place 18 byte item on stack
    Push18(Value),
    /// Place 19 byte item on stack
    Push19(Value),
    /// Place 20 byte item on stack
    Push20(Value),
    /// Place 21 byte item on stack
    Push21(Value),
    /// Place 22 byte item on stack
    Push22(Value),
    /// Place 23 byte item on stack
    Push23(Value),
    /// Place 24 byte item on stack
    Push24(Value),
    /// Place 25 byte item on stack
    Push25(Value),
    /// Place 26 byte item on stack
    Push26(Value),
    /// Place 27 byte item on stack
    Push27(Value),
    /// Place 28 byte item on stack
    Push28(Value),
    /// Place 29 byte item on stack
    Push29(Value),
    /// Place 30 byte item on stack
    Push30(Value),
    /// Place 31 byte item on stack
    Push31(Value),
    /// Place 32 byte (full word) item on stack
    Push32(Value),
    /// Duplicate 1st stack item
    Dup1,
    /// Duplicate 2nd stack item
    Dup2,
    /// Duplicate 3rd stack item
    Dup3,
    /// Duplicate 4th stack item
    Dup4,
    /// Duplicate 5th stack item
    Dup5,
    /// Duplicate 6th stack item
    Dup6,
    /// Duplicate 7th stack item
    Dup7,
    /// Duplicate 8th stack item
    Dup8,
    /// Duplicate 9th stack item
    Dup9,
    /// Duplicate 10th stack item
    Dup10,
    /// Duplicate 11th stack item
    Dup11,
    /// Duplicate 12th stack item
    Dup12,
    /// Duplicate 13th stack item
    Dup13,
    /// Duplicate 14th stack item
    Dup14,
    /// Duplicate 15th stack item
    Dup15,
    /// Duplicate 16th stack item
    Dup16,
    /// Exchange 1st and 2nd stack items
    Swap1,
    /// Exchange 1st and 3rd stack items
    Swap2,
    /// Exchange 1st and 4th stack items
    Swap3,
    /// Exchange 1st and 5th stack items
    Swap4,
    /// Exchange 1st and 6th stack items
    Swap5,
    /// Exchange 1st and 7th stack items
    Swap6,
    /// Exchange 1st and 8th stack items
    Swap7,
    /// Exchange 1st and 9th stack items
    Swap8,
    /// Exchange 1st and 10th stack items
    Swap9,
    /// Exchange 1st and 11th stack items
    Swap10,
    /// Exchange 1st and 12th stack items
    Swap11,
    /// Exchange 1st and 13th stack items
    Swap12,
    /// Exchange 1st and 14th stack items
    Swap13,
    /// Exchange 1st and 15th stack items
    Swap14,
    /// Exchange 1st and 16th stack items
    Swap15,
    /// Exchange 1st and 17th stack items
    Swap16,
    /// Append log record with no topics
    Log0,
    /// Append log record with one topic
    Log1,
    /// Append log record with two topics
    Log2,
    /// Append log record with three topics
    Log3,
    /// Append log record with four topics
    Log4,
    /// Create a new account with associated code
    Create,
    /// Message-call into an account
    Call,
    /// Message-call into this account with alternative account’s code
    CallCode,
    /// Halt execution returning output data
    Return,
    /// Message-call into this account with an alternative account’s code, but persisting the current values for sender and value
    DelegateCall,
    /// Create a new account with associated code at a predictable address
    Create2,
    /// Static message-call into an account
    StaticCall,
    /// Halt execution reverting state changes but returning data and remaining gas
    Revert,
    /// Designated invalid instruction
    Invalid,
    /// Halt execution and register account for later deletion
    SelfDestruct,
}

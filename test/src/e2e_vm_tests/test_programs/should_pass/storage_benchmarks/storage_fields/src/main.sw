contract;

use stored_types::*;

storage {
    t_bool: bool = false,
    t_u8: u8 = 0,
    t_u16: u16 = 0,
    t_u32: u32 = 0,
    t_u64: u64 = 0,
    t_u256: u256 = 0u256,
    t_struct24: Struct24 = STRUCT24_DEFAULT,
    t_struct32: Struct32 = STRUCT32_DEFAULT,
    t_struct40: Struct40 = STRUCT40_DEFAULT,
    t_struct48: Struct48 = STRUCT48_DEFAULT,
    t_struct56: Struct56 = STRUCT56_DEFAULT,
    t_struct72: Struct72 = STRUCT72_DEFAULT,
    t_struct88: Struct88 = STRUCT88_DEFAULT,
    t_struct96: Struct96 = STRUCT96_DEFAULT,
    t_struct184: Struct184 = STRUCT184_DEFAULT,
    t_struct200: Struct200 = STRUCT200_DEFAULT,
    t_struct224: Struct224 = STRUCT224_DEFAULT,
    t_struct552: Struct552 = STRUCT552_DEFAULT,
}

impl Contract {
    // To measure the baseline cost of a contract method call.
    fn baseline() { }

    // bool
    #[storage(read)]
    fn bool_read() {
        let _ = storage.t_bool.try_read();
    }

    #[storage(write)]
    fn bool_write() {
        let _ = storage.t_bool.write(false);
    }

    #[storage(write)]
    fn bool_clear() {
        let _ = storage.t_bool.clear();
    }

    // u8
    #[storage(read)]
    fn u8_read() {
        let _ = storage.t_u8.try_read();
    }

    #[storage(write)]
    fn u8_write() {
        let _ = storage.t_u8.write(0);
    }

    #[storage(write)]
    fn u8_clear() {
        let _ = storage.t_u8.clear();
    }

    // u16
    #[storage(read)]
    fn u16_read() {
        let _ = storage.t_u16.try_read();
    }

    #[storage(write)]
    fn u16_write() {
        let _ = storage.t_u16.write(0);
    }

    #[storage(write)]
    fn u16_clear() {
        let _ = storage.t_u16.clear();
    }

    // u32
    #[storage(read)]
    fn u32_read() {
        let _ = storage.t_u32.try_read();
    }

    #[storage(write)]
    fn u32_write() {
        let _ = storage.t_u32.write(0);
    }

    #[storage(write)]
    fn u32_clear() {
        let _ = storage.t_u32.clear();
    }

    // u64
    #[storage(read)]
    fn u64_read() {
        let _ = storage.t_u64.try_read();
    }

    #[storage(write)]
    fn u64_write() {
        let _ = storage.t_u64.write(0);
    }

    #[storage(write)]
    fn u64_clear() {
        let _ = storage.t_u64.clear();
    }

    // u256
    #[storage(read)]
    fn u256_read() {
        let _ = storage.t_u256.try_read();
    }

    #[storage(write)]
    fn u256_write() {
        let _ = storage.t_u256.write(0u256);
    }

    #[storage(write)]
    fn u256_clear() {
        let _ = storage.t_u256.clear();
    }

    // Struct24
    #[storage(read)]
    fn struct24_read() {
        let _ = storage.t_struct24.try_read();
    }

    #[storage(write)]
    fn struct24_write() {
        let _ = storage.t_struct24.write(STRUCT24_DEFAULT);
    }

    #[storage(write)]
    fn struct24_clear() {
        let _ = storage.t_struct24.clear();
    }

    // Struct32
    #[storage(read)]
    fn struct32_read() {
        let _ = storage.t_struct32.try_read();
    }

    #[storage(write)]
    fn struct32_write() {
        let _ = storage.t_struct32.write(STRUCT32_DEFAULT);
    }

    #[storage(write)]
    fn struct32_clear() {
        let _ = storage.t_struct32.clear();
    }

    // Struct40
    #[storage(read)]
    fn struct40_read() {
        let _ = storage.t_struct40.try_read();
    }

    #[storage(write)]
    fn struct40_write() {
        let _ = storage.t_struct40.write(STRUCT40_DEFAULT);
    }

    #[storage(write)]
    fn struct40_clear() {
        let _ = storage.t_struct40.clear();
    }

    // Struct48
    #[storage(read)]
    fn struct48_read() {
        let _ = storage.t_struct48.try_read();
    }

    #[storage(write)]
    fn struct48_write() {
        let _ = storage.t_struct48.write(STRUCT48_DEFAULT);
    }

    #[storage(write)]
    fn struct48_clear() {
        let _ = storage.t_struct48.clear();
    }

    // Struct56
    #[storage(read)]
    fn struct56_read() {
        let _ = storage.t_struct56.try_read();
    }

    #[storage(write)]
    fn struct56_write() {
        let _ = storage.t_struct56.write(STRUCT56_DEFAULT);
    }

    #[storage(write)]
    fn struct56_clear() {
        let _ = storage.t_struct56.clear();
    }

    // Struct72
    #[storage(read)]
    fn struct72_read() {
        let _ = storage.t_struct72.try_read();
    }

    #[storage(write)]
    fn struct72_write() {
        let _ = storage.t_struct72.write(STRUCT72_DEFAULT);
    }

    #[storage(write)]
    fn struct72_clear() {
        let _ = storage.t_struct72.clear();
    }

    // Struct88
    #[storage(read)]
    fn struct88_read() {
        let _ = storage.t_struct88.try_read();
    }

    #[storage(write)]
    fn struct88_write() {
        let _ = storage.t_struct88.write(STRUCT88_DEFAULT);
    }

    #[storage(write)]
    fn struct88_clear() {
        let _ = storage.t_struct88.clear();
    }

    // Struct96
    #[storage(read)]
    fn struct96_read() {
        let _ = storage.t_struct96.try_read();
    }

    #[storage(write)]
    fn struct96_write() {
        let _ = storage.t_struct96.write(STRUCT96_DEFAULT);
    }

    #[storage(write)]
    fn struct96_clear() {
        let _ = storage.t_struct96.clear();
    }

    // Struct184
    #[storage(read)]
    fn struct184_read() {
        let _ = storage.t_struct184.try_read();
    }

    #[storage(write)]
    fn struct184_write() {
        let _ = storage.t_struct184.write(STRUCT184_DEFAULT);
    }

    #[storage(write)]
    fn struct184_clear() {
        let _ = storage.t_struct184.clear();
    }

    // Struct200
    #[storage(read)]
    fn struct200_read() {
        let _ = storage.t_struct200.try_read();
    }

    #[storage(write)]
    fn struct200_write() {
        let _ = storage.t_struct200.write(STRUCT200_DEFAULT);
    }

    #[storage(write)]
    fn struct200_clear() {
        let _ = storage.t_struct200.clear();
    }

    // Struct224
    #[storage(read)]
    fn struct224_read() {
        let _ = storage.t_struct224.try_read();
    }

    #[storage(write)]
    fn struct224_write() {
        let _ = storage.t_struct224.write(STRUCT224_DEFAULT);
    }

    #[storage(write)]
    fn struct224_clear() {
        let _ = storage.t_struct224.clear();
    }

    // Struct552
    #[storage(read)]
    fn struct552_read() {
        let _ = storage.t_struct552.try_read();
    }

    #[storage(write)]
    fn struct552_write() {
        let _ = storage.t_struct552.write(STRUCT552_DEFAULT);
    }

    #[storage(write)]
    fn struct552_clear() {
        let _ = storage.t_struct552.clear();
    }
}

// === Baseline ===

#[test]
fn bench_baseline() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.baseline();
}

// === bool ===

#[test]
fn bench_bool_read() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.bool_read();
}

#[test]
fn bench_bool_write() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.bool_write();
}

#[test]
fn bench_bool_clear() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.bool_clear();
}

// === u8 ===

#[test]
fn bench_u8_read() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.u8_read();
}

#[test]
fn bench_u8_write() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.u8_write();
}

#[test]
fn bench_u8_clear() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.u8_clear();
}

// === u16 ===

#[test]
fn bench_u16_read() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.u16_read();
}

#[test]
fn bench_u16_write() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.u16_write();
}

#[test]
fn bench_u16_clear() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.u16_clear();
}

// === u32 ===

#[test]
fn bench_u32_read() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.u32_read();
}

#[test]
fn bench_u32_write() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.u32_write();
}

#[test]
fn bench_u32_clear() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.u32_clear();
}

// === u64 ===

#[test]
fn bench_u64_read() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.u64_read();
}

#[test]
fn bench_u64_write() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.u64_write();
}

#[test]
fn bench_u64_clear() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.u64_clear();
}

// === u256 ===

#[test]
fn bench_u256_read() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.u256_read();
}

#[test]
fn bench_u256_write() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.u256_write();
}

#[test]
fn bench_u256_clear() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.u256_clear();
}

// === Struct24 (24 bytes) ===

#[test]
fn bench_struct24_read() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct24_read();
}

#[test]
fn bench_struct24_write() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct24_write();
}

#[test]
fn bench_struct24_clear() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct24_clear();
}

// === Struct32 (32 bytes) ===

#[test]
fn bench_struct32_read() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct32_read();
}

#[test]
fn bench_struct32_write() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct32_write();
}

#[test]
fn bench_struct32_clear() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct32_clear();
}

// === Struct40 (40 bytes) ===

#[test]
fn bench_struct40_read() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct40_read();
}

#[test]
fn bench_struct40_write() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct40_write();
}

#[test]
fn bench_struct40_clear() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct40_clear();
}

// === Struct48 (48 bytes) ===

#[test]
fn bench_struct48_read() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct48_read();
}

#[test]
fn bench_struct48_write() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct48_write();
}

#[test]
fn bench_struct48_clear() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct48_clear();
}

// === Struct56 (56 bytes) ===

#[test]
fn bench_struct56_read() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct56_read();
}

#[test]
fn bench_struct56_write() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct56_write();
}

#[test]
fn bench_struct56_clear() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct56_clear();
}

// === Struct72 (72 bytes) ===

#[test]
fn bench_struct72_read() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct72_read();
}

#[test]
fn bench_struct72_write() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct72_write();
}

#[test]
fn bench_struct72_clear() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct72_clear();
}

// === Struct88 (88 bytes) ===

#[test]
fn bench_struct88_read() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct88_read();
}

#[test]
fn bench_struct88_write() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct88_write();
}

#[test]
fn bench_struct88_clear() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct88_clear();
}

// === Struct96 (96 bytes) ===

#[test]
fn bench_struct96_read() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct96_read();
}

#[test]
fn bench_struct96_write() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct96_write();
}

#[test]
fn bench_struct96_clear() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct96_clear();
}

// === Struct184 (184 bytes) ===

#[test]
fn bench_struct184_read() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct184_read();
}

#[test]
fn bench_struct184_write() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct184_write();
}

#[test]
fn bench_struct184_clear() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct184_clear();
}

// === Struct200 (200 bytes) ===

#[test]
fn bench_struct200_read() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct200_read();
}

#[test]
fn bench_struct200_write() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct200_write();
}

#[test]
fn bench_struct200_clear() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct200_clear();
}

// === Struct224 (224 bytes) ===

#[test]
fn bench_struct224_read() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct224_read();
}

#[test]
fn bench_struct224_write() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct224_write();
}

#[test]
fn bench_struct224_clear() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct224_clear();
}

// === Struct552 (552 bytes) ===

#[test]
fn bench_struct552_read() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct552_read();
}

#[test]
fn bench_struct552_write() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct552_write();
}

#[test]
fn bench_struct552_clear() {
    let caller = abi(StorageFieldsAbi, CONTRACT_ID);
    caller.struct552_clear();
}

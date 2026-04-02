contract;

use stored_types::*;

storage {
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

    // Struct24 { a: u64, b: u64, c: u64 }
    // Partial: u64 (8 of 24)
    #[storage(read)]
    fn struct24_read_u64() {
        let _ = storage.t_struct24.b.try_read();
    }

    #[storage(write)]
    fn struct24_write_u64() {
        let _ = storage.t_struct24.b.write(0);
    }

    // Struct32 { a: u64, b: u64, c: u64, d: u64 }
    // Partial: u64 (8 of 32)
    #[storage(read)]
    fn struct32_read_u64() {
        let _ = storage.t_struct32.b.try_read();
    }

    #[storage(write)]
    fn struct32_write_u64() {
        let _ = storage.t_struct32.b.write(0);
    }

    // Struct40 { a: u64, b: u64, c: u64, d: u64, e: u64 }
    // Partial: u64 (8 of 40)
    #[storage(read)]
    fn struct40_read_u64() {
        let _ = storage.t_struct40.b.try_read();
    }

    #[storage(write)]
    fn struct40_write_u64() {
        let _ = storage.t_struct40.b.write(0);
    }

    // Struct48 { a: Struct24, b: Struct24 }
    // Partial: Struct24 (24 of 48)
    #[storage(read)]
    fn struct48_read_struct24() {
        let _ = storage.t_struct48.b.try_read();
    }

    #[storage(write)]
    fn struct48_write_struct24() {
        let _ = storage.t_struct48.b.write(STRUCT24_DEFAULT);
    }

    // Partial: Struct24.u64 (8 of 48)
    #[storage(read)]
    fn struct48_read_struct24_u64() {
        let _ = storage.t_struct48.b.b.try_read();
    }

    #[storage(write)]
    fn struct48_write_struct24_u64() {
        let _ = storage.t_struct48.b.b.write(0);
    }

    // Struct56 { a: Struct24, b: Struct32 }
    // Partial: Struct24 (24 of 56)
    #[storage(read)]
    fn struct56_read_struct24() {
        let _ = storage.t_struct56.a.try_read();
    }

    #[storage(write)]
    fn struct56_write_struct24() {
        let _ = storage.t_struct56.a.write(STRUCT24_DEFAULT);
    }

    // Partial: Struct32 (32 of 56)
    #[storage(read)]
    fn struct56_read_struct32() {
        let _ = storage.t_struct56.b.try_read();
    }

    #[storage(write)]
    fn struct56_write_struct32() {
        let _ = storage.t_struct56.b.write(STRUCT32_DEFAULT);
    }

    // Partial: Struct24.u64 (8 of 56)
    #[storage(read)]
    fn struct56_read_struct24_u64() {
        let _ = storage.t_struct56.a.b.try_read();
    }

    #[storage(write)]
    fn struct56_write_struct24_u64() {
        let _ = storage.t_struct56.a.b.write(0);
    }

    // Partial: Struct32.u64 (8 of 56)
    #[storage(read)]
    fn struct56_read_struct32_u64() {
        let _ = storage.t_struct56.b.b.try_read();
    }

    #[storage(write)]
    fn struct56_write_struct32_u64() {
        let _ = storage.t_struct56.b.b.write(0);
    }

    // Struct72 { a: Struct32, b: Struct40 }
    // Partial: Struct32 (32 of 72)
    #[storage(read)]
    fn struct72_read_struct32() {
        let _ = storage.t_struct72.a.try_read();
    }

    #[storage(write)]
    fn struct72_write_struct32() {
        let _ = storage.t_struct72.a.write(STRUCT32_DEFAULT);
    }

    // Partial: Struct40 (40 of 72)
    #[storage(read)]
    fn struct72_read_struct40() {
        let _ = storage.t_struct72.b.try_read();
    }

    #[storage(write)]
    fn struct72_write_struct40() {
        let _ = storage.t_struct72.b.write(STRUCT40_DEFAULT);
    }

    // Partial: Struct32.u64 (8 of 72)
    #[storage(read)]
    fn struct72_read_struct32_u64() {
        let _ = storage.t_struct72.a.b.try_read();
    }

    #[storage(write)]
    fn struct72_write_struct32_u64() {
        let _ = storage.t_struct72.a.b.write(0);
    }

    // Partial: Struct40.u64 (8 of 72)
    #[storage(read)]
    fn struct72_read_struct40_u64() {
        let _ = storage.t_struct72.b.b.try_read();
    }

    #[storage(write)]
    fn struct72_write_struct40_u64() {
        let _ = storage.t_struct72.b.b.write(0);
    }

    // Struct88 { a: Struct40, b: Struct40, c: u64 }
    // Partial: Struct40 (40 of 88)
    #[storage(read)]
    fn struct88_read_struct40() {
        let _ = storage.t_struct88.b.try_read();
    }

    #[storage(write)]
    fn struct88_write_struct40() {
        let _ = storage.t_struct88.b.write(STRUCT40_DEFAULT);
    }

    // Partial: u64 (8 of 88)
    #[storage(read)]
    fn struct88_read_u64() {
        let _ = storage.t_struct88.c.try_read();
    }

    #[storage(write)]
    fn struct88_write_u64() {
        let _ = storage.t_struct88.c.write(0);
    }

    // Partial: Struct40.u64 (8 of 88)
    #[storage(read)]
    fn struct88_read_struct40_u64() {
        let _ = storage.t_struct88.b.b.try_read();
    }

    #[storage(write)]
    fn struct88_write_struct40_u64() {
        let _ = storage.t_struct88.b.b.write(0);
    }

    // Struct96 { a: Struct48, b: Struct48 }
    // Partial: Struct48 (48 of 96)
    #[storage(read)]
    fn struct96_read_struct48() {
        let _ = storage.t_struct96.b.try_read();
    }

    #[storage(write)]
    fn struct96_write_struct48() {
        let _ = storage.t_struct96.b.write(STRUCT48_DEFAULT);
    }

    // Partial: Struct48.Struct24 (24 of 96)
    #[storage(read)]
    fn struct96_read_struct48_struct24() {
        let _ = storage.t_struct96.b.b.try_read();
    }

    #[storage(write)]
    fn struct96_write_struct48_struct24() {
        let _ = storage.t_struct96.b.b.write(STRUCT24_DEFAULT);
    }

    // Partial: Struct48.Struct24.u64 (8 of 96)
    #[storage(read)]
    fn struct96_read_struct48_struct24_u64() {
        let _ = storage.t_struct96.b.b.b.try_read();
    }

    #[storage(write)]
    fn struct96_write_struct48_struct24_u64() {
        let _ = storage.t_struct96.b.b.b.write(0);
    }

    // Struct184 { a: Struct96, b: Struct88 }
    // Partial: Struct96 (96 of 184)
    #[storage(read)]
    fn struct184_read_struct96() {
        let _ = storage.t_struct184.a.try_read();
    }

    #[storage(write)]
    fn struct184_write_struct96() {
        let _ = storage.t_struct184.a.write(STRUCT96_DEFAULT);
    }

    // Partial: Struct88 (88 of 184)
    #[storage(read)]
    fn struct184_read_struct88() {
        let _ = storage.t_struct184.b.try_read();
    }

    #[storage(write)]
    fn struct184_write_struct88() {
        let _ = storage.t_struct184.b.write(STRUCT88_DEFAULT);
    }

    // Partial: Struct96.Struct48 (48 of 184)
    #[storage(read)]
    fn struct184_read_struct96_struct48() {
        let _ = storage.t_struct184.a.b.try_read();
    }

    #[storage(write)]
    fn struct184_write_struct96_struct48() {
        let _ = storage.t_struct184.a.b.write(STRUCT48_DEFAULT);
    }

    // Partial: Struct88.Struct40 (40 of 184)
    #[storage(read)]
    fn struct184_read_struct88_struct40() {
        let _ = storage.t_struct184.b.b.try_read();
    }

    #[storage(write)]
    fn struct184_write_struct88_struct40() {
        let _ = storage.t_struct184.b.b.write(STRUCT40_DEFAULT);
    }

    // Partial: Struct96.Struct48.Struct24 (24 of 184)
    #[storage(read)]
    fn struct184_read_struct96_struct48_struct24() {
        let _ = storage.t_struct184.a.b.b.try_read();
    }

    #[storage(write)]
    fn struct184_write_struct96_struct48_struct24() {
        let _ = storage.t_struct184.a.b.b.write(STRUCT24_DEFAULT);
    }

    // Partial: Struct88.Struct40.u64 (8 of 184)
    #[storage(read)]
    fn struct184_read_struct88_struct40_u64() {
        let _ = storage.t_struct184.b.b.b.try_read();
    }

    #[storage(write)]
    fn struct184_write_struct88_struct40_u64() {
        let _ = storage.t_struct184.b.b.b.write(0);
    }

    // Partial: Struct96.Struct48.Struct24.u64 (8 of 184)
    #[storage(read)]
    fn struct184_read_struct96_struct48_struct24_u64() {
        let _ = storage.t_struct184.a.b.b.b.try_read();
    }

    #[storage(write)]
    fn struct184_write_struct96_struct48_struct24_u64() {
        let _ = storage.t_struct184.a.b.b.b.write(0);
    }

    // Struct200 { a: Struct96, b: Struct96, c: u64 }
    // Partial: Struct96 (96 of 200)
    #[storage(read)]
    fn struct200_read_struct96() {
        let _ = storage.t_struct200.b.try_read();
    }

    #[storage(write)]
    fn struct200_write_struct96() {
        let _ = storage.t_struct200.b.write(STRUCT96_DEFAULT);
    }

    // Partial: u64 (8 of 200)
    #[storage(read)]
    fn struct200_read_u64() {
        let _ = storage.t_struct200.c.try_read();
    }

    #[storage(write)]
    fn struct200_write_u64() {
        let _ = storage.t_struct200.c.write(0);
    }

    // Partial: Struct96.Struct48 (48 of 200)
    #[storage(read)]
    fn struct200_read_struct96_struct48() {
        let _ = storage.t_struct200.b.b.try_read();
    }

    #[storage(write)]
    fn struct200_write_struct96_struct48() {
        let _ = storage.t_struct200.b.b.write(STRUCT48_DEFAULT);
    }

    // Partial: Struct96.Struct48.Struct24 (24 of 200)
    #[storage(read)]
    fn struct200_read_struct96_struct48_struct24() {
        let _ = storage.t_struct200.b.b.b.try_read();
    }

    #[storage(write)]
    fn struct200_write_struct96_struct48_struct24() {
        let _ = storage.t_struct200.b.b.b.write(STRUCT24_DEFAULT);
    }

    // Partial: Struct96.Struct48.Struct24.u64 (8 of 200)
    #[storage(read)]
    fn struct200_read_struct96_struct48_struct24_u64() {
        let _ = storage.t_struct200.b.b.b.b.try_read();
    }

    #[storage(write)]
    fn struct200_write_struct96_struct48_struct24_u64() {
        let _ = storage.t_struct200.b.b.b.b.write(0);
    }

    // Struct224 { a: Struct96, b: Struct96, c: Struct32 }
    // Partial: Struct96 (96 of 224)
    #[storage(read)]
    fn struct224_read_struct96() {
        let _ = storage.t_struct224.b.try_read();
    }

    #[storage(write)]
    fn struct224_write_struct96() {
        let _ = storage.t_struct224.b.write(STRUCT96_DEFAULT);
    }

    // Partial: Struct32 (32 of 224)
    #[storage(read)]
    fn struct224_read_struct32() {
        let _ = storage.t_struct224.c.try_read();
    }

    #[storage(write)]
    fn struct224_write_struct32() {
        let _ = storage.t_struct224.c.write(STRUCT32_DEFAULT);
    }

    // Partial: Struct96.Struct48 (48 of 224)
    #[storage(read)]
    fn struct224_read_struct96_struct48() {
        let _ = storage.t_struct224.b.b.try_read();
    }

    #[storage(write)]
    fn struct224_write_struct96_struct48() {
        let _ = storage.t_struct224.b.b.write(STRUCT48_DEFAULT);
    }

    // Partial: Struct32.u64 (8 of 224)
    #[storage(read)]
    fn struct224_read_struct32_u64() {
        let _ = storage.t_struct224.c.b.try_read();
    }

    #[storage(write)]
    fn struct224_write_struct32_u64() {
        let _ = storage.t_struct224.c.b.write(0);
    }

    // Partial: Struct96.Struct48.Struct24 (24 of 224)
    #[storage(read)]
    fn struct224_read_struct96_struct48_struct24() {
        let _ = storage.t_struct224.b.b.b.try_read();
    }

    #[storage(write)]
    fn struct224_write_struct96_struct48_struct24() {
        let _ = storage.t_struct224.b.b.b.write(STRUCT24_DEFAULT);
    }

    // Partial: Struct96.Struct48.Struct24.u64 (8 of 224)
    #[storage(read)]
    fn struct224_read_struct96_struct48_struct24_u64() {
        let _ = storage.t_struct224.b.b.b.b.try_read();
    }

    #[storage(write)]
    fn struct224_write_struct96_struct48_struct24_u64() {
        let _ = storage.t_struct224.b.b.b.b.write(0);
    }

    // Struct552 { a: Struct224, b: Struct224, c: Struct96, d: u64 }
    // Partial: Struct224 (224 of 552)
    #[storage(read)]
    fn struct552_read_struct224() {
        let _ = storage.t_struct552.b.try_read();
    }

    #[storage(write)]
    fn struct552_write_struct224() {
        let _ = storage.t_struct552.b.write(STRUCT224_DEFAULT);
    }

    // Partial: Struct96 (96 of 552)
    #[storage(read)]
    fn struct552_read_struct96() {
        let _ = storage.t_struct552.c.try_read();
    }

    #[storage(write)]
    fn struct552_write_struct96() {
        let _ = storage.t_struct552.c.write(STRUCT96_DEFAULT);
    }

    // Partial: u64 (8 of 552)
    #[storage(read)]
    fn struct552_read_u64() {
        let _ = storage.t_struct552.d.try_read();
    }

    #[storage(write)]
    fn struct552_write_u64() {
        let _ = storage.t_struct552.d.write(0);
    }

    // Partial: Struct224.Struct96 (96 of 552)
    #[storage(read)]
    fn struct552_read_struct224_struct96() {
        let _ = storage.t_struct552.b.b.try_read();
    }

    #[storage(write)]
    fn struct552_write_struct224_struct96() {
        let _ = storage.t_struct552.b.b.write(STRUCT96_DEFAULT);
    }

    // Partial: Struct224.Struct32 (32 of 552)
    #[storage(read)]
    fn struct552_read_struct224_struct32() {
        let _ = storage.t_struct552.b.c.try_read();
    }

    #[storage(write)]
    fn struct552_write_struct224_struct32() {
        let _ = storage.t_struct552.b.c.write(STRUCT32_DEFAULT);
    }

    // Partial: Struct224.Struct96.Struct48 (48 of 552)
    #[storage(read)]
    fn struct552_read_struct224_struct96_struct48() {
        let _ = storage.t_struct552.b.b.b.try_read();
    }

    #[storage(write)]
    fn struct552_write_struct224_struct96_struct48() {
        let _ = storage.t_struct552.b.b.b.write(STRUCT48_DEFAULT);
    }

    // Partial: Struct224.Struct96.Struct48.Struct24 (24 of 552)
    #[storage(read)]
    fn struct552_read_struct224_struct96_struct48_struct24() {
        let _ = storage.t_struct552.b.b.b.b.try_read();
    }

    #[storage(write)]
    fn struct552_write_struct224_struct96_struct48_struct24() {
        let _ = storage.t_struct552.b.b.b.b.write(STRUCT24_DEFAULT);
    }

    // Partial: Struct224.Struct96.Struct48.Struct24.u64 (8 of 552)
    #[storage(read)]
    fn struct552_read_struct224_struct96_struct48_struct24_u64() {
        let _ = storage.t_struct552.b.b.b.b.b.try_read();
    }

    #[storage(write)]
    fn struct552_write_struct224_struct96_struct48_struct24_u64() {
        let _ = storage.t_struct552.b.b.b.b.b.write(0);
    }
}

// === Baseline ===

#[test]
fn bench_baseline() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.baseline();
}

// === Struct24 (24 bytes) ===

#[test]
fn bench_struct24_read_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct24_read_u64();
}

#[test]
fn bench_struct24_write_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct24_write_u64();
}

// === Struct32 (32 bytes) ===

#[test]
fn bench_struct32_read_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct32_read_u64();
}

#[test]
fn bench_struct32_write_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct32_write_u64();
}

// === Struct40 (40 bytes) ===

#[test]
fn bench_struct40_read_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct40_read_u64();
}

#[test]
fn bench_struct40_write_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct40_write_u64();
}

// === Struct48 (48 bytes) ===

#[test]
fn bench_struct48_read_struct24() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct48_read_struct24();
}

#[test]
fn bench_struct48_write_struct24() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct48_write_struct24();
}

#[test]
fn bench_struct48_read_struct24_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct48_read_struct24_u64();
}

#[test]
fn bench_struct48_write_struct24_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct48_write_struct24_u64();
}

// === Struct56 (56 bytes) ===

#[test]
fn bench_struct56_read_struct24() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct56_read_struct24();
}

#[test]
fn bench_struct56_write_struct24() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct56_write_struct24();
}

#[test]
fn bench_struct56_read_struct32() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct56_read_struct32();
}

#[test]
fn bench_struct56_write_struct32() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct56_write_struct32();
}

#[test]
fn bench_struct56_read_struct24_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct56_read_struct24_u64();
}

#[test]
fn bench_struct56_write_struct24_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct56_write_struct24_u64();
}

#[test]
fn bench_struct56_read_struct32_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct56_read_struct32_u64();
}

#[test]
fn bench_struct56_write_struct32_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct56_write_struct32_u64();
}

// === Struct72 (72 bytes) ===

#[test]
fn bench_struct72_read_struct32() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct72_read_struct32();
}

#[test]
fn bench_struct72_write_struct32() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct72_write_struct32();
}

#[test]
fn bench_struct72_read_struct40() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct72_read_struct40();
}

#[test]
fn bench_struct72_write_struct40() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct72_write_struct40();
}

#[test]
fn bench_struct72_read_struct32_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct72_read_struct32_u64();
}

#[test]
fn bench_struct72_write_struct32_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct72_write_struct32_u64();
}

#[test]
fn bench_struct72_read_struct40_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct72_read_struct40_u64();
}

#[test]
fn bench_struct72_write_struct40_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct72_write_struct40_u64();
}

// === Struct88 (88 bytes) ===

#[test]
fn bench_struct88_read_struct40() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct88_read_struct40();
}

#[test]
fn bench_struct88_write_struct40() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct88_write_struct40();
}

#[test]
fn bench_struct88_read_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct88_read_u64();
}

#[test]
fn bench_struct88_write_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct88_write_u64();
}

#[test]
fn bench_struct88_read_struct40_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct88_read_struct40_u64();
}

#[test]
fn bench_struct88_write_struct40_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct88_write_struct40_u64();
}

// === Struct96 (96 bytes) ===

#[test]
fn bench_struct96_read_struct48() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct96_read_struct48();
}

#[test]
fn bench_struct96_write_struct48() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct96_write_struct48();
}

#[test]
fn bench_struct96_read_struct48_struct24() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct96_read_struct48_struct24();
}

#[test]
fn bench_struct96_write_struct48_struct24() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct96_write_struct48_struct24();
}

#[test]
fn bench_struct96_read_struct48_struct24_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct96_read_struct48_struct24_u64();
}

#[test]
fn bench_struct96_write_struct48_struct24_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct96_write_struct48_struct24_u64();
}

// === Struct184 (184 bytes) ===

#[test]
fn bench_struct184_read_struct96() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct184_read_struct96();
}

#[test]
fn bench_struct184_write_struct96() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct184_write_struct96();
}

#[test]
fn bench_struct184_read_struct88() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct184_read_struct88();
}

#[test]
fn bench_struct184_write_struct88() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct184_write_struct88();
}

#[test]
fn bench_struct184_read_struct96_struct48() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct184_read_struct96_struct48();
}

#[test]
fn bench_struct184_write_struct96_struct48() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct184_write_struct96_struct48();
}

#[test]
fn bench_struct184_read_struct88_struct40() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct184_read_struct88_struct40();
}

#[test]
fn bench_struct184_write_struct88_struct40() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct184_write_struct88_struct40();
}

#[test]
fn bench_struct184_read_struct96_struct48_struct24() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct184_read_struct96_struct48_struct24();
}

#[test]
fn bench_struct184_write_struct96_struct48_struct24() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct184_write_struct96_struct48_struct24();
}

#[test]
fn bench_struct184_read_struct88_struct40_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct184_read_struct88_struct40_u64();
}

#[test]
fn bench_struct184_write_struct88_struct40_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct184_write_struct88_struct40_u64();
}

#[test]
fn bench_struct184_read_struct96_struct48_struct24_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct184_read_struct96_struct48_struct24_u64();
}

#[test]
fn bench_struct184_write_struct96_struct48_struct24_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct184_write_struct96_struct48_struct24_u64();
}

// === Struct200 (200 bytes) ===

#[test]
fn bench_struct200_read_struct96() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct200_read_struct96();
}

#[test]
fn bench_struct200_write_struct96() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct200_write_struct96();
}

#[test]
fn bench_struct200_read_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct200_read_u64();
}

#[test]
fn bench_struct200_write_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct200_write_u64();
}

#[test]
fn bench_struct200_read_struct96_struct48() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct200_read_struct96_struct48();
}

#[test]
fn bench_struct200_write_struct96_struct48() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct200_write_struct96_struct48();
}

#[test]
fn bench_struct200_read_struct96_struct48_struct24() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct200_read_struct96_struct48_struct24();
}

#[test]
fn bench_struct200_write_struct96_struct48_struct24() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct200_write_struct96_struct48_struct24();
}

#[test]
fn bench_struct200_read_struct96_struct48_struct24_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct200_read_struct96_struct48_struct24_u64();
}

#[test]
fn bench_struct200_write_struct96_struct48_struct24_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct200_write_struct96_struct48_struct24_u64();
}

// === Struct224 (224 bytes) ===

#[test]
fn bench_struct224_read_struct96() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct224_read_struct96();
}

#[test]
fn bench_struct224_write_struct96() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct224_write_struct96();
}

#[test]
fn bench_struct224_read_struct32() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct224_read_struct32();
}

#[test]
fn bench_struct224_write_struct32() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct224_write_struct32();
}

#[test]
fn bench_struct224_read_struct96_struct48() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct224_read_struct96_struct48();
}

#[test]
fn bench_struct224_write_struct96_struct48() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct224_write_struct96_struct48();
}

#[test]
fn bench_struct224_read_struct32_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct224_read_struct32_u64();
}

#[test]
fn bench_struct224_write_struct32_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct224_write_struct32_u64();
}

#[test]
fn bench_struct224_read_struct96_struct48_struct24() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct224_read_struct96_struct48_struct24();
}

#[test]
fn bench_struct224_write_struct96_struct48_struct24() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct224_write_struct96_struct48_struct24();
}

#[test]
fn bench_struct224_read_struct96_struct48_struct24_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct224_read_struct96_struct48_struct24_u64();
}

#[test]
fn bench_struct224_write_struct96_struct48_struct24_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct224_write_struct96_struct48_struct24_u64();
}

// === Struct552 (552 bytes) ===

#[test]
fn bench_struct552_read_struct224() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct552_read_struct224();
}

#[test]
fn bench_struct552_write_struct224() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct552_write_struct224();
}

#[test]
fn bench_struct552_read_struct96() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct552_read_struct96();
}

#[test]
fn bench_struct552_write_struct96() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct552_write_struct96();
}

#[test]
fn bench_struct552_read_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct552_read_u64();
}

#[test]
fn bench_struct552_write_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct552_write_u64();
}

#[test]
fn bench_struct552_read_struct224_struct96() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct552_read_struct224_struct96();
}

#[test]
fn bench_struct552_write_struct224_struct96() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct552_write_struct224_struct96();
}

#[test]
fn bench_struct552_read_struct224_struct32() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct552_read_struct224_struct32();
}

#[test]
fn bench_struct552_write_struct224_struct32() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct552_write_struct224_struct32();
}

#[test]
fn bench_struct552_read_struct224_struct96_struct48() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct552_read_struct224_struct96_struct48();
}

#[test]
fn bench_struct552_write_struct224_struct96_struct48() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct552_write_struct224_struct96_struct48();
}

#[test]
fn bench_struct552_read_struct224_struct96_struct48_struct24() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct552_read_struct224_struct96_struct48_struct24();
}

#[test]
fn bench_struct552_write_struct224_struct96_struct48_struct24() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct552_write_struct224_struct96_struct48_struct24();
}

#[test]
fn bench_struct552_read_struct224_struct96_struct48_struct24_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct552_read_struct224_struct96_struct48_struct24_u64();
}

#[test]
fn bench_struct552_write_struct224_struct96_struct48_struct24_u64() {
    let caller = abi(StorageFieldsPartialAccessAbi, CONTRACT_ID);
    caller.struct552_write_struct224_struct96_struct48_struct24_u64();
}

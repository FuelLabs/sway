use forc_diagnostic::{println_green, println_red};
use paste::paste;
use prettydiff::{basic::DiffOp, diff_lines};
use test_macros::fmt_test_item;

fmt_test_item!(  storage_maps
"storage {
    map1: StorageMap<u64, bool> = StorageMap {},
    map2: StorageMap<u64, u8> = StorageMap {},
    map3: StorageMap<u64, u16> = StorageMap {},
    map4: StorageMap<u64, u32> = StorageMap {},
    map5: StorageMap<u64, u64> = StorageMap {},
    map6: StorageMap<u64, (b256, u8, bool)> = StorageMap {},
    map7: StorageMap<u64, Struct> = StorageMap {},
    map8: StorageMap<u64, Enum> = StorageMap {},
    map9: StorageMap<u64, str[33]> = StorageMap {},
    map10: StorageMap<u64, [b256; 3]> = StorageMap {},
    map11: StorageMap<bool, u64> = StorageMap {},
    map12: StorageMap<u8, u64> = StorageMap {},
    map13: StorageMap<u16, u64> = StorageMap {},
    map14: StorageMap<u32, u64> = StorageMap {},
    map15: StorageMap<(b256, u8, bool), u64> = StorageMap {},
    map16: StorageMap<Struct, u64> = StorageMap {},
    map17: StorageMap<Enum, u64> = StorageMap {},
    map18: StorageMap<str[33], u64> = StorageMap {},
    map19: StorageMap<[b256; 3], u64> = StorageMap {},
}",
            wrong_new_lines
"storage {
    map1: StorageMap<u64,
    bool> = StorageMap {
    },
    map2: StorageMap<u64,
    u8> = StorageMap {
    },
    map3: StorageMap<u64,
    u16> = StorageMap {
    },
    map4: StorageMap<u64,
    u32> = StorageMap {
    },
    map5: StorageMap<u64,
    u64> = StorageMap {
    },
    map6: StorageMap<u64,
    (b256, u8, bool) > = StorageMap {
    },
    map7: StorageMap<u64,
    Struct> = StorageMap {
    },
    map8: StorageMap<u64,
    Enum> = StorageMap {
    },
    map9: StorageMap<u64,
    str[33]> = StorageMap {
    },
    map10: StorageMap<u64,
    [b256;
    3]> = StorageMap {
    },
    map11: StorageMap<bool,
    u64> = StorageMap {
    },
    map12: StorageMap<u8,
    u64> = StorageMap {
    },
    map13: StorageMap<u16,
    u64> = StorageMap {
    },
    map14: StorageMap<u32,
    u64> = StorageMap {
    },
    map15: StorageMap<(b256,
    u8, bool), u64 > = StorageMap {
    },
    map16: StorageMap<Struct,
    u64> = StorageMap {
    },
    map17: StorageMap<Enum,
    u64> = StorageMap {
    },
    map18: StorageMap<str[33],
    u64> = StorageMap {
    },
    map19: StorageMap<[b256;
    3],
    u64> = StorageMap {
    },
}"
);

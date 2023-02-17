#[macro_use]
mod testgen;

mod vec_array;

// abigen!(Contract(
//     name = "TestContractB256",
//     abi = "test_artifacts/storage_vec/svec_b256/out/debug/svec_b256-abi.json",
// ));
// abigen!(Contract(
//     name = "TestContractBool",
//     abi = "test_artifacts/storage_vec/svec_bool/out/debug/svec_bool-abi.json",
// ));
// abigen!(Contract(
//     name = "TestContractEnum",
//     abi = "test_artifacts/storage_vec/svec_enum/out/debug/svec_enum-abi.json",
// ));
// abigen!(Contract(
//     name = "TestContractStr",
//     abi = "test_artifacts/storage_vec/svec_str/out/debug/svec_str-abi.json",
// ));
// abigen!(Contract(
//     name = "TestContractStruct",
//     abi = "test_artifacts/storage_vec/svec_struct/out/debug/svec_struct-abi.json",
// ));
// abigen!(Contract(
//     name = "TestContractTuple",
//     abi = "test_artifacts/storage_vec/svec_tuple/out/debug/svec_tuple-abi.json",
// ));
// abigen!(Contract(
//     name = "TestContractU8",
//     abi = "test_artifacts/storage_vec/svec_u8/out/debug/svec_u8-abi.json",
// ));
// abigen!(Contract(
//     name = "TestContractU16",
//     abi = "test_artifacts/storage_vec/svec_u16/out/debug/svec_u16-abi.json",
// ));
// abigen!(Contract(
//     name = "TestContractU32",
//     abi = "test_artifacts/storage_vec/svec_u32/out/debug/svec_u32-abi.json",
// ));
// abigen!(Contract(
//     name = "TestContractU64",
//     abi = "test_artifacts/storage_vec/svec_u64/out/debug/svec_u64-abi.json",
// ));

// testgen!(
//     TestContractB256,
//     test_b256_vec,
//     "b256",
//     [u8; 32],
//     [1; 32],
//     [2; 32],
//     [3; 32],
//     [4; 32],
//     [5; 32]
// );
// testgen!(TestContractBool, test_bool_vec, "bool", bool, true, false, true, false, true);
// testgen!(
//     TestContractEnum,
//     test_enum_vec,
//     "enum",
//     TestEnum,
//     TestEnum::A(true),
//     TestEnum::A(false),
//     TestEnum::B(1),
//     TestEnum::B(3),
//     TestEnum::B(2),
// );
// testgen!(
//     TestContractStr,
//     test_str_vec,
//     "str",
//     str,
//     "yeet".to_string(),
//     "meow".to_string(),
//     "kekw".to_string(),
//     "gmgn".to_string(),
//     "sway".to_string(),
// );
// testgen!(
//     TestContractStruct
//     test_struct_vec,
//     "struct",
//     TestStruct,
//     TestStruct { a: true, b: 1 },
//     TestStruct { a: false, b: 2 },
//     TestStruct { a: true, b: 3 },
//     TestStruct { a: false, b: 4 },
//     TestStruct { a: true, b: 5 },
// );
// testgen!(
//     TestContractTuple,
//     test_tuple_vec,
//     "tuple",
//     (u8, u8, u8),
//     (1, 1, 1),
//     (2, 2, 2),
//     (4, 4, 4),
//     (5, 5, 5)
// );
// testgen!(TestContractU8, test_u8_vec, "u8", u8, 1u8, 2u8, 3u8, 4u8, 5u8);
// testgen!(TestContractU16, test_u16_vec, "u16", u16, 1u16, 2u16, 3u16, 4u16, 5u16);
// testgen!(TestContractU32, test_u32_vec, "u32", u32, 1u32, 2u32, 3u32, 4u32, 5u32);
// testgen!(TestContractU64, test_u64_vec, "u64", u64, 1u64, 2u64, 3u64, 4u64, 5u64);

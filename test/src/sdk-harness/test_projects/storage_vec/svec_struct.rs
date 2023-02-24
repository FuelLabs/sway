testgen!(
    test_struct_vec,
    "test_artifacts/storage_vec/svec_struct/out/debug/svec_struct-abi.json",
    "struct",
    TestStruct,
    TestStruct { a: true, b: 1 },
    TestStruct { a: false, b: 2 },
    TestStruct { a: true, b: 3 },
    TestStruct { a: false, b: 4 },
    TestStruct { a: true, b: 5 }
);

testgen!(
    test_struct_vec,
    "out_for_sdk_harness_tests/svec_struct-abi.json",
    "struct",
    TestStruct,
    TestStruct { a: true, b: 1 },
    TestStruct { a: false, b: 2 },
    TestStruct { a: true, b: 3 },
    TestStruct { a: false, b: 4 },
    TestStruct { a: true, b: 5 }
);

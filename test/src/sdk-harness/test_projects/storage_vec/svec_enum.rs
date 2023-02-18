

testgen!(
    test_enum_vec,
    "test_artifacts/storage_vec/svec_enum/out/debug/svec_enum-abi.json",
    "enum",
    TestEnum,
    TestEnum::A(true),
    TestEnum::A(false),
    TestEnum::B(1),
    TestEnum::B(3),
    TestEnum::B(2)
);

testgen!(
    test_b256_vec,
    "out_for_sdk_harness_tests/svec_b256-abi.json",
    "b256",
    ::fuels::types::Bits256,
    ::fuels::types::Bits256([1; 32]),
    ::fuels::types::Bits256([2; 32]),
    ::fuels::types::Bits256([3; 32]),
    ::fuels::types::Bits256([4; 32]),
    ::fuels::types::Bits256([5; 32])
);

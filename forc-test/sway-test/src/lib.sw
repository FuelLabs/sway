library;

pub mod random;
pub mod fuzz;

use fuzz::*;

#[test]
fn test_addition_fuzzing() {
    let mut fuzzer = Fuzzer::<u64>::new(100);
    let mut i = 0;

    while i < 100 {
        let value = fuzzer.next();
        // Test your code with fuzzed values
        assert(value + 0 == value);
        i += 1;
    }
}

#[test]
fn test_u32_fuzzing_with_seed() {
    let config = FuzzConfig::new(50).with_seed(42);
    let mut fuzzer = Fuzzer::<u32>::with_config(config);
    let mut i = 0;

    while i < 50 {
        let value = fuzzer.next();
        // Test property: value is equal to itself
        assert(value == value);
        i += 1;
    }
}

#[test]
fn test_deterministic_fuzzing() {
    // First run
    let config1 = FuzzConfig::new(5).with_seed(12345);
    let mut fuzzer1 = Fuzzer::<u64>::with_config(config1);
    let mut values1: [u64; 5] = [0; 5];
    let mut i = 0;

    while i < 5 {
        values1[i] = fuzzer1.next();
        i += 1;
    }

    // Second run with same seed
    let config2 = FuzzConfig::new(5).with_seed(12345);
    let mut fuzzer2 = Fuzzer::<u64>::with_config(config2);
    let mut values2: [u64; 5] = [0; 5];
    i = 0;

    while i < 5 {
        values2[i] = fuzzer2.next();
        i += 1;
    }

    // Should produce the same values
    assert(values1[0] == values2[0]);
    assert(values1[1] == values2[1]);
    assert(values1[2] == values2[2]);
    assert(values1[3] == values2[3]);
    assert(values1[4] == values2[4]);
}

#[test]
fn test_bool_fuzzing() {
    let mut fuzzer = Fuzzer::<bool>::new(20);
    let mut i = 0;

    while i < 20 {
        let _value = fuzzer.next();
        // Just verify we can generate bool values without panicking
        i += 1;
    }
}

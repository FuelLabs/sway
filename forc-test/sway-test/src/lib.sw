library;

pub mod random;
pub mod fuzz;

use fuzz::*;

/// Test struct with mixed field types
pub struct MyStruct {
    pub field1: u8,
    pub field2: bool,
    pub field3: u32,
}

/// Test struct with multiple u64 fields for range validation
pub struct ComplexStruct {
    pub a: u64,
    pub b: u64,
    pub c: u32,
    pub d: u8,
}

/// Simple enum with unit variants
pub enum SimpleEnum {
    A: (),
    B: (),
    C: (),
}

/// Enum with primitive type variants
pub enum PrimitiveEnum {
    Value: u64,
    Flag: bool,
    Count: u32,
}

/// Enum with nested complex variants
pub enum ComplexEnum {
    Simple: SimpleEnum,
    Struct: MyStruct,
    Primitive: u64,
}

#[test]
fn test_u64_fuzzing_generates_varied_values() {
    let mut fuzzer = Fuzzer::<u64>::new(100);
    let mut i = 0;
    let mut has_zero = false;
    let mut has_non_zero = false;

    while i < 100 {
        let value = fuzzer.next();
        if value == 0 {
            has_zero = true;
        } else {
            has_non_zero = true;
        }
        i += 1;
    }

    assert(has_zero || has_non_zero);
}

#[test]
fn test_u32_fuzzing_without_panics() {
    let mut fuzzer = Fuzzer::<u32>::new(100);
    let mut i = 0;

    while i < 100 {
        let _value = fuzzer.next();
        i += 1;
    }
}

#[test]
fn test_deterministic_fuzzing() {
    let config1 = FuzzConfig::new(5).with_seed(12345);
    let mut fuzzer1 = Fuzzer::<u64>::with_config(config1);
    let mut values1: [u64; 5] = [0; 5];
    let mut i = 0;

    while i < 5 {
        values1[i] = fuzzer1.next();
        i += 1;
    }

    let config2 = FuzzConfig::new(5).with_seed(12345);
    let mut fuzzer2 = Fuzzer::<u64>::with_config(config2);
    let mut values2: [u64; 5] = [0; 5];
    i = 0;

    while i < 5 {
        values2[i] = fuzzer2.next();
        i += 1;
    }

    assert(values1[0] == values2[0]);
    assert(values1[1] == values2[1]);
    assert(values1[2] == values2[2]);
    assert(values1[3] == values2[3]);
    assert(values1[4] == values2[4]);
}

#[test]
fn test_different_seeds_produce_different_values() {
    let mut values: [u64; 3] = [0; 3];
    values[0] = fuzz_any(1);
    values[1] = fuzz_any(2);
    values[2] = fuzz_any(3);

    let all_same = values[0] == values[1] && values[1] == values[2];
    assert(!all_same);
}

#[test]
fn test_bool_distribution() {
    let mut fuzzer = Fuzzer::<bool>::with_config(FuzzConfig::new(1000).with_seed(42));
    let mut true_count = 0;
    let mut false_count = 0;
    let mut i = 0;

    while i < 1000 {
        let value = fuzzer.next();
        if value {
            true_count += 1;
        } else {
            false_count += 1;
        }
        i += 1;
    }

    assert(true_count > 0);
    assert(false_count > 0);
    assert(true_count + false_count == 1000);
}

#[test]
fn test_struct_field_independence() {
    let s1: MyStruct = fuzz_any(100);
    let s2: MyStruct = fuzz_any(101);

    let all_same = s1.field1 == s2.field1 && s1.field2 == s2.field2 && s1.field3 == s2.field3;
    assert(!all_same);
}

#[test]
fn test_complex_struct_field_ranges() {
    let mut fuzzer = Fuzzer::<ComplexStruct>::with_config(FuzzConfig::new(20).with_seed(777));
    let mut i = 0;
    let mut has_non_zero_a = false;
    let mut has_non_zero_b = false;

    while i < 20 {
        let s = fuzzer.next();
        assert(s.d <= 255);

        if s.a != 0 {
            has_non_zero_a = true;
        }
        if s.b != 0 {
            has_non_zero_b = true;
        }
        i += 1;
    }

    assert(has_non_zero_a);
    assert(has_non_zero_b);
}

#[test]
fn test_simple_enum_fuzzing() {
    let mut fuzzer = Fuzzer::<SimpleEnum>::new(30);
    let mut i = 0;

    while i < 30 {
        let _e = fuzzer.next();
        i += 1;
    }
}

#[test]
fn test_primitive_enum_fuzzing() {
    let mut fuzzer = Fuzzer::<PrimitiveEnum>::new(40);
    let mut i = 0;

    while i < 40 {
        let _e = fuzzer.next();
        i += 1;
    }
}

#[test]
fn test_complex_enum_fuzzing() {
    let mut fuzzer = Fuzzer::<ComplexEnum>::new(25);
    let mut i = 0;

    while i < 25 {
        let _e = fuzzer.next();
        i += 1;
    }
}

#[test]
fn test_fuzz_any_determinism() {
    let s1: ComplexStruct = fuzz_any(42);
    let s2: ComplexStruct = fuzz_any(42);

    assert(s1.a == s2.a);
    assert(s1.b == s2.b);
    assert(s1.c == s2.c);
    assert(s1.d == s2.d);
}

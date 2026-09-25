# sway_test

A property-based testing and fuzzing library for Sway.

## Overview

`sway_test` provides a simple and powerful fuzzing framework that works with any Sway type automatically. No trait implementations or manual configuration required.

## Features

- **Universal Type Support** - Fuzz any struct, enum, or primitive type
- **Deterministic Testing** - Reproducible test runs with seed-based randomness
- **Zero Configuration** - No trait implementations needed
- **Memory-Safe** - Leverages Sway's type system for safe fuzzing

## Installation

Add to your `Forc.toml`:

```toml
[dependencies]
sway_test = { path = "path/to/sway_test" }
```

## Usage

### Basic Fuzzing

```sway
use sway_test::*;

struct Transaction {
    from: u64,
    to: u64,
    amount: u32,
}

#[test]
fn test_transaction_processing() {
    let mut fuzzer = Fuzzer::<Transaction>::new(100);
    let mut i = 0;

    while i < 100 {
        let tx = fuzzer.next();
        process_transaction(tx);
        i += 1;
    }
}
```

### Deterministic Fuzzing

```sway
#[test]
fn test_with_seed() {
    let config = FuzzConfig::new(50).with_seed(42);
    let mut fuzzer = Fuzzer::<MyStruct>::with_config(config);

    // Same seed always produces same sequence
    let value = fuzzer.next();
}
```

### Direct Value Generation

```sway
// Generate a single fuzzed value
let value: MyStruct = fuzz_any(42);
```

## API

### Core Types

#### `Fuzzer<T>`

Iterator-like fuzzer for generating random values of type `T`.

**Methods:**
- `new(iterations: u64) -> Self` - Create fuzzer with iteration count
- `with_config(config: FuzzConfig) -> Self` - Create with custom configuration
- `next() -> T` - Generate next fuzzed value
- `has_next() -> bool` - Check if more values available

#### `FuzzConfig`

Configuration for fuzzing behavior.

**Methods:**
- `new(iterations: u64) -> Self` - Create configuration
- `with_seed(seed: u64) -> Self` - Set deterministic seed

### Functions

#### `fuzz_any<T>(seed: u64) -> T`

Generate a single fuzzed value of any type.

#### Random Number Generation

```sway
// Non-deterministic
random_u64() -> u64
random_u32() -> u32
random_u8() -> u8

// Deterministic (seeded)
random_u64_seeded(seed: u64) -> u64
random_u32_seeded(seed: u64) -> u32
random_u8_seeded(seed: u64) -> u8
```

## How It Works

The library uses memory-level fuzzing to generate random values:

1. Determines type size using `__size_of::<T>()`
2. Fills memory with random bytes via ecal syscalls
3. Returns the initialized value

This approach works for any type without requiring trait implementations.

## Testing

```bash
forc test
```

## License

Apache-2.0

library;

use ::random::*;

/// Trait for types that can be generated from random bytes
pub trait Arbitrary {
    /// Generate a random value from a seed
    fn arbitrary(seed: u64) -> Self;
}

impl Arbitrary for u64 {
    fn arbitrary(seed: u64) -> Self {
        random_u64_seeded(seed)
    }
}

impl Arbitrary for u32 {
    fn arbitrary(seed: u64) -> Self {
        random_u32_seeded(seed)
    }
}

impl Arbitrary for u8 {
    fn arbitrary(seed: u64) -> Self {
        random_u8_seeded(seed)
    }
}

impl Arbitrary for bool {
    fn arbitrary(seed: u64) -> Self {
        random_u8_seeded(seed) % 2 == 0
    }
}

/// Fuzzing configuration
pub struct FuzzConfig {
    /// Number of iterations to run
    pub iterations: u64,
    /// Base seed for deterministic fuzzing
    pub base_seed: u64,
}

impl FuzzConfig {
    /// Create a new fuzz configuration with default settings
    pub fn new(iterations: u64) -> Self {
        Self {
            iterations,
            base_seed: 0,
        }
    }

    /// Set the base seed for deterministic fuzzing
    pub fn with_seed(self, seed: u64) -> Self {
        Self {
            iterations: self.iterations,
            base_seed: seed,
        }
    }
}

/// Fuzzer for generating random test inputs
pub struct Fuzzer<T> where T: Arbitrary {
    config: FuzzConfig,
    current: u64,
}

impl<T> Fuzzer<T> where T: Arbitrary {
    /// Create a new fuzzer with the given number of iterations
    pub fn new(iterations: u64) -> Self {
        Self {
            config: FuzzConfig::new(iterations),
            current: 0,
        }
    }

    /// Create a new fuzzer with custom configuration
    pub fn with_config(config: FuzzConfig) -> Self {
        Self {
            config,
            current: 0,
        }
    }

    /// Check if there are more values to generate
    pub fn has_next(self) -> bool {
        self.current < self.config.iterations
    }

    /// Get the next fuzzed value
    /// Panics if has_next() is false
    pub fn next(ref mut self) -> T {
        let seed = self.config.base_seed + self.current;
        self.current += 1;
        T::arbitrary(seed)
    }
}


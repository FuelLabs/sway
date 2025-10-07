library;

use ::random::*;

/// Generate a fuzzed value of any type by filling its memory with random bytes.
///
/// # Type Parameters
/// * `T` - The type to fuzz
///
/// # Arguments
/// * `seed` - Seed for deterministic random generation
///
/// # Returns
/// A randomly generated value of type T
///
/// # Example
/// ```sway
/// struct MyStruct { a: u64, b: u32 }
/// let value: MyStruct = fuzz_any(42);
/// ```
pub fn fuzz_any<T>(seed: u64) -> T {
    let size_in_bytes = __size_of::<T>();

    let mut value: T = asm(size: size_in_bytes) {
        size: T
    };

    let ptr = asm(r1: __addr_of(value)) { r1: u64 };
    random_bytes_seeded(ptr, size_in_bytes, seed);

    value
}

/// Fuzzing configuration
pub struct FuzzConfig {
    /// Number of iterations to run
    pub iterations: u64,
    /// Base seed for deterministic fuzzing
    pub base_seed: u64,
}

impl FuzzConfig {
    /// Create a new fuzz configuration
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

/// Fuzzer for generating random test inputs.
/// Works with any type T automatically.
pub struct Fuzzer<T> {
    config: FuzzConfig,
    current: u64,
}

impl<T> Fuzzer<T> {
    /// Create a new fuzzer
    ///
    /// # Arguments
    /// * `iterations` - Number of values to generate
    pub fn new(iterations: u64) -> Self {
        Self {
            config: FuzzConfig::new(iterations),
            current: 0,
        }
    }

    /// Create a fuzzer with custom configuration
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

    /// Generate the next fuzzed value
    pub fn next(ref mut self) -> T {
        let seed = self.config.base_seed + self.current;
        self.current += 1;
        fuzz_any::<T>(seed)
    }
}

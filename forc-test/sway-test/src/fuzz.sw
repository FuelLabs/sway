library;

use ::random::*;

/// Generate a fuzzed value of any type by filling its memory with random bytes.
///
/// This function uses memory-level fuzzing to create random instances of any Sway type
/// without requiring trait implementations. It leverages the VM's random syscall to fill
/// the type's memory representation with deterministic random bytes based on the seed.
///
/// # Type Parameters
/// * `T` - Any Sway type to generate a random value for
///
/// # Arguments
/// * `seed` - Seed value for reproducible random generation
///
/// # Returns
/// A random instance of type `T` with all fields filled with random data
///
/// # Examples
/// ```sway
/// // Generate random primitive
/// let random_number: u64 = fuzz_any(42);
///
/// // Generate random struct
/// struct MyStruct { a: u64, b: u32 }
/// let random_struct: MyStruct = fuzz_any(100);
///
/// // Same seed produces same value
/// let val1: u64 = fuzz_any(42);
/// let val2: u64 = fuzz_any(42);
/// assert(val1 == val2);
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

/// Configuration for fuzzing behavior.
pub struct FuzzConfig {
    /// Number of fuzz iterations to run
    pub iterations: u64,
    /// Base seed value for deterministic generation (defaults to 0)
    pub base_seed: u64,
}

impl FuzzConfig {
    /// Create a new fuzzing configuration with default base seed of 0.
    ///
    /// # Arguments
    /// * `iterations` - Number of random values to generate
    ///
    /// # Returns
    /// A new `FuzzConfig` instance
    pub fn new(iterations: u64) -> Self {
        Self {
            iterations,
            base_seed: 0,
        }
    }

    /// Set a custom base seed for deterministic fuzzing.
    ///
    /// # Arguments
    /// * `seed` - Base seed value (each iteration uses base_seed + iteration_number)
    ///
    /// # Returns
    /// Updated `FuzzConfig` with the specified seed
    pub fn with_seed(self, seed: u64) -> Self {
        Self {
            iterations: self.iterations,
            base_seed: seed,
        }
    }
}

/// Iterator-like fuzzer for generating sequences of random test inputs.
///
/// The fuzzer automatically generates random values of any type without requiring
/// trait implementations. Each call to `next()` produces a new random value using
/// an incrementing seed (base_seed + iteration_count).
///
/// # Type Parameters
/// * `T` - Any Sway type to generate random values for
///
/// # Examples
/// ```sway
/// // Create fuzzer with 100 iterations
/// let mut fuzzer = Fuzzer::<u64>::new(100);
/// while fuzzer.has_next() {
///     let value = fuzzer.next();
///     // Test with random value
/// }
///
/// // Deterministic fuzzing with custom seed
/// let config = FuzzConfig::new(50).with_seed(12345);
/// let mut fuzzer = Fuzzer::<MyStruct>::with_config(config);
/// ```
pub struct Fuzzer<T> {
    config: FuzzConfig,
    current: u64,
}

impl<T> Fuzzer<T> {
    /// Create a new fuzzer with the specified number of iterations.
    ///
    /// # Arguments
    /// * `iterations` - Number of random values to generate
    ///
    /// # Returns
    /// A new `Fuzzer` instance with base seed 0
    pub fn new(iterations: u64) -> Self {
        Self {
            config: FuzzConfig::new(iterations),
            current: 0,
        }
    }

    /// Create a fuzzer with custom configuration.
    ///
    /// # Arguments
    /// * `config` - Fuzzing configuration with iterations and base seed
    ///
    /// # Returns
    /// A new `Fuzzer` instance with the specified configuration
    pub fn with_config(config: FuzzConfig) -> Self {
        Self {
            config,
            current: 0,
        }
    }

    /// Check if there are more values to generate.
    ///
    /// # Returns
    /// `true` if more iterations remain, `false` otherwise
    pub fn has_next(self) -> bool {
        self.current < self.config.iterations
    }

    /// Generate the next random value in the sequence.
    ///
    /// Each call increments the iteration counter and generates a new random value
    /// using seed = base_seed + current_iteration.
    ///
    /// # Returns
    /// A random value of type `T`
    pub fn next(ref mut self) -> T {
        let seed = self.config.base_seed + self.current;
        self.current += 1;
        fuzz_any::<T>(seed)
    }
}

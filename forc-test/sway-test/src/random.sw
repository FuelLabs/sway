library;

/// Syscall ID for non-deterministic random generation
const RANDOM_SYSCALL: u64 = 1002;
/// Syscall ID for deterministic seeded random generation
const RANDOM_SEEDED_SYSCALL: u64 = 1003;

/// Fill a buffer with random bytes using a non-deterministic source.
///
/// # Arguments
/// * `buffer_ptr` - Memory address of the buffer to fill
/// * `count` - Number of bytes to generate
pub fn random_bytes(buffer_ptr: u64, count: u64) {
    asm(r1: RANDOM_SYSCALL, r2: buffer_ptr, r3: count) {
        ecal r1 r2 r3 zero;
    }
}

/// Fill a buffer with random bytes using a deterministic seeded source.
///
/// # Arguments
/// * `buffer_ptr` - Memory address of the buffer to fill
/// * `count` - Number of bytes to generate
/// * `seed` - Seed value for reproducible generation
pub fn random_bytes_seeded(buffer_ptr: u64, count: u64, seed: u64) {
    asm(r1: RANDOM_SEEDED_SYSCALL, r2: buffer_ptr, r3: count, r4: seed) {
        ecal r1 r2 r3 r4;
    }
}

/// Generate a random `u64` value using a non-deterministic source.
///
/// # Returns
/// A random 64-bit unsigned integer
pub fn random_u64() -> u64 {
    let mut buffer: u64 = 0;
    let buffer_ptr = asm(r1: buffer) { r1: u64 };
    random_bytes(buffer_ptr, 8);
    buffer
}

/// Generate a random `u64` value using a deterministic seeded source.
///
/// # Arguments
/// * `seed` - Seed value for reproducible generation
///
/// # Returns
/// A deterministic 64-bit unsigned integer based on the seed
pub fn random_u64_seeded(seed: u64) -> u64 {
    let mut buffer: u64 = 0;
    let buffer_ptr = asm(r1: buffer) { r1: u64 };
    random_bytes_seeded(buffer_ptr, 8, seed);
    buffer
}

/// Generate a random `u32` value using a non-deterministic source.
///
/// # Returns
/// A random 32-bit unsigned integer
pub fn random_u32() -> u32 {
    let mut buffer: u32 = 0;
    let buffer_ptr = asm(r1: buffer) { r1: u64 };
    random_bytes(buffer_ptr, 4);
    buffer
}

/// Generate a random `u32` value using a deterministic seeded source.
///
/// # Arguments
/// * `seed` - Seed value for reproducible generation
///
/// # Returns
/// A deterministic 32-bit unsigned integer based on the seed
pub fn random_u32_seeded(seed: u64) -> u32 {
    let mut buffer: u32 = 0;
    let buffer_ptr = asm(r1: buffer) { r1: u64 };
    random_bytes_seeded(buffer_ptr, 4, seed);
    buffer
}

/// Generate a random `u8` value using a non-deterministic source.
///
/// # Returns
/// A random 8-bit unsigned integer
pub fn random_u8() -> u8 {
    let mut buffer: u8 = 0;
    let buffer_ptr = asm(r1: buffer) { r1: u64 };
    random_bytes(buffer_ptr, 1);
    buffer
}

/// Generate a random `u8` value using a deterministic seeded source.
///
/// # Arguments
/// * `seed` - Seed value for reproducible generation
///
/// # Returns
/// A deterministic 8-bit unsigned integer based on the seed
pub fn random_u8_seeded(seed: u64) -> u8 {
    let mut buffer: u8 = 0;
    let buffer_ptr = asm(r1: buffer) { r1: u64 };
    random_bytes_seeded(buffer_ptr, 1, seed);
    buffer
}

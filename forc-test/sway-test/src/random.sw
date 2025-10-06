library;

/// Syscall numbers for random generation
const RANDOM_SYSCALL: u64 = 1002;
const RANDOM_SEEDED_SYSCALL: u64 = 1003;

/// Generate random bytes into a buffer using a non-deterministic random source
///
/// # Arguments
/// * `buffer_ptr` - Pointer to the buffer where random bytes will be written
/// * `count` - Number of random bytes to generate
pub fn random_bytes(buffer_ptr: u64, count: u64) {
    asm(r1: RANDOM_SYSCALL, r2: buffer_ptr, r3: count) {
        ecal r1 r2 r3 zero;
    }
}

/// Generate random bytes into a buffer using a seeded deterministic random source
///
/// # Arguments
/// * `buffer_ptr` - Pointer to the buffer where random bytes will be written
/// * `count` - Number of random bytes to generate
/// * `seed` - Seed value for deterministic random generation
pub fn random_bytes_seeded(buffer_ptr: u64, count: u64, seed: u64) {
    asm(r1: RANDOM_SEEDED_SYSCALL, r2: buffer_ptr, r3: count, r4: seed) {
        ecal r1 r2 r3 r4;
    }
}

/// Generate a random u64 value using non-deterministic random source
pub fn random_u64() -> u64 {
    let mut buffer: u64 = 0;
    let buffer_ptr = asm(r1: buffer) { r1: u64 };
    random_bytes(buffer_ptr, 8);
    buffer
}

/// Generate a random u64 value using seeded deterministic random source
pub fn random_u64_seeded(seed: u64) -> u64 {
    let mut buffer: u64 = 0;
    let buffer_ptr = asm(r1: buffer) { r1: u64 };
    random_bytes_seeded(buffer_ptr, 8, seed);
    buffer
}

/// Generate a random u32 value using non-deterministic random source
pub fn random_u32() -> u32 {
    let mut buffer: u32 = 0;
    let buffer_ptr = asm(r1: buffer) { r1: u64 };
    random_bytes(buffer_ptr, 4);
    buffer
}

/// Generate a random u32 value using seeded deterministic random source
pub fn random_u32_seeded(seed: u64) -> u32 {
    let mut buffer: u32 = 0;
    let buffer_ptr = asm(r1: buffer) { r1: u64 };
    random_bytes_seeded(buffer_ptr, 4, seed);
    buffer
}

/// Generate a random u8 value using non-deterministic random source
pub fn random_u8() -> u8 {
    let mut buffer: u8 = 0;
    let buffer_ptr = asm(r1: buffer) { r1: u64 };
    random_bytes(buffer_ptr, 1);
    buffer
}

/// Generate a random u8 value using seeded deterministic random source
pub fn random_u8_seeded(seed: u64) -> u8 {
    let mut buffer: u8 = 0;
    let buffer_ptr = asm(r1: buffer) { r1: u64 };
    random_bytes_seeded(buffer_ptr, 1, seed);
    buffer
}

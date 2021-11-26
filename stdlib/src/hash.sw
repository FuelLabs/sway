library hash;

// Should this be a trait eventually? Do we want to allow people to customize what `!` does?
// Scala says yes, Rust says perhaps...
pub fn not(a: bool) -> bool {
    // using direct asm for perf
    asm(r1: a, r2) {
        eq r2 r1 zero;
        r2: bool
    }
}

pub enum HashMethod {
    Sha256: (),
    Keccak256: (),
}

impl HashMethod {
    fn eq(self, other: Self) -> bool {
        // Enum instantiations are on the stack, so we can use MEQ in this case to just compare the
        // tag in the variant (and ignore the unit values).
        asm(r1: self, r2: other, r3, r4) {
            addi r3 zero i8;
            meq r4 r1 r2 r3;
            r4: bool
        }
    }
}

pub fn hash_u64(value: u64, method: HashMethod) -> b256 {
    if method.eq(HashMethod::Sha256) {
        asm(r1: value, hashed_b256_ptr, r3, value_ptr) {
            // put the u64 on the stack
            move value_ptr sp;
            cfei i8;
            sw value_ptr r1 i0;
            move hashed_b256_ptr sp;
            cfei i32;
            addi r3 zero i8; // hash eight bytes since a u64 is eight bytes
            s256 hashed_b256_ptr r1 r3;
            hashed_b256_ptr: b256
        }
    } else {
        asm(r1: value, hashed_b256_ptr, r3, value_ptr) {
            // put the u64 on the stack
            move value_ptr sp;
            cfei i8;
            sw value_ptr r1 i0;
            move hashed_b256_ptr sp;
            cfei i32;
            addi r3 zero i8; // hash eight bytes since a u64 is eight bytes
            k256 hashed_b256_ptr r1 r3;
            hashed_b256_ptr: b256
        }
    }
}

pub fn hash_value(value: b256, method: HashMethod) -> b256 {
    // Use pattern matching for method when we can...
    // NOTE: Deliberately using Sha256 here and Keccak256 below to avoid 'never constructed
    // warning'.
    if method.eq(HashMethod::Sha256) {
        asm(r1: value, r2, r3) {
            move r2 sp;
            cfei i32;
            addi r3 zero i32;
            s256 r2 r1 r3;
            r2: b256
        }
    } else {
        asm(r1: value, r2, r3) {
            move r2 sp;
            cfei i32;
            addi r3 zero i32;
            k256 r2 r1 r3;
            r2: b256
        }
    }
}

pub fn hash_pair(value_a: b256, value_b: b256, method: HashMethod) -> b256 {
    // Use pattern matching for method when we can...
    // NOTE: Deliberately using Keccak256 here and Sha256 above to avoid 'never constructed
    // warning'.
    // TODO: Avoid the code duplication?  Ideally this conditional would be tightly wrapped around
    // the S256 and K256 instructions but we'd need control flow within ASM blocks to allow that.
    if method.eq(HashMethod::Keccak256) {
        asm(r1: value_a, r2: value_b, r3, r4, r5, r6) {
            move r3 sp; // Result buffer.
            cfei i32;
            move r4 sp; // Buffer for copies of value_a and value_b.
            cfei i64;

            addi r5 zero i32;
            mcp r4 r1 r5; // Copy 32 bytes to buffer.
            addi r6 r4 i32;
            mcp r6 r2 r5; // Append 32 bytes to buffer.

            addi r5 r5 i32;
            k256 r3 r4 r5; // Hash 64 bytes to the result buffer.

            cfsi i64; // Free the copies buffer.

            r3: b256
        }
    } else {
        asm(r1: value_a, r2: value_b, r3, r4, r5, r6) {
            move r3 sp; // Result buffer.
            cfei i32;
            move r4 sp; // Buffer for copies of value_a and value_b.
            cfei i64;

            addi r5 zero i32;
            mcp r4 r1 r5; // Copy 32 bytes to buffer.
            addi r6 r4 i32;
            mcp r6 r2 r5; // Append 32 bytes to buffer.

            addi r5 r5 i32;
            s256 r3 r4 r5; // Hash 64 bytes to the result buffer.

            cfsi i64; // Free the copies buffer.

            r3: b256
        }
    }
}

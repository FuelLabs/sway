library;

// Shadows the Add trait from core::ops, imported via core::prelude
pub trait Add {
    // Same name as core::ops::Add, but different return type
    fn add(self, other: Self) -> u64;
}

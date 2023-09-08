#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct B256([u64; 32]);

impl B256 {
    pub fn new(bytes: [u64; 32]) -> Self {
        Self(bytes)
    }
}

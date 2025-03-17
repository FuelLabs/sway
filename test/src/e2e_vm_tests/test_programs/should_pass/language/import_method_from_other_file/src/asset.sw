library;

pub struct Asset {
    pub value: u64,
}

impl PartialEq for Asset {
    fn eq(self, other: Self) -> bool {
        self.value == other.value
    }
}
impl Eq for Asset {}

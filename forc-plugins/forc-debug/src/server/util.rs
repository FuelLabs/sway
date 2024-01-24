#[derive(Debug, Clone)]
pub(crate) struct IdGenerator {
    next_id: i64,
}

impl Default for IdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl IdGenerator {
    pub(crate) fn new() -> Self {
        Self { next_id: 0 }
    }

    pub(crate) fn next(&mut self) -> i64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

library;

pub struct CanInitStruct {
    pub x: u64,
    y: u64,
}

impl CanInitStruct {
    pub fn init(x: u64, y: u64) -> Self {
        Self { x, y }
    }
}

impl PartialEq for CanInitStruct {
    fn eq(self, other: Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
impl Eq for CanInitStruct {}

pub struct CannotInitStruct {
    pub x: u64,
    y: u64,
}

impl CannotInitStruct {
    pub fn init(x: u64, y: u64) -> Self {
        // Cannot evaluate to constant because of `return`.
        return Self { x, y };
    }
}

impl PartialEq for CannotInitStruct {
    fn eq(self, other: Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
impl Eq for CannotInitStruct {}

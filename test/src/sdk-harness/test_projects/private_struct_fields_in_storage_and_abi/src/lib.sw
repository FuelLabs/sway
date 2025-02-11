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

#[cfg(experimental_partial_eq = false)]
impl core::ops::Eq for CanInitStruct {
    fn eq(self, other: Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
#[cfg(experimental_partial_eq = true)]
impl core::ops::PartialEq for CanInitStruct {
    fn eq(self, other: Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
#[cfg(experimental_partial_eq = true)]
impl core::ops::Eq for CanInitStruct {}

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

#[cfg(experimental_partial_eq = false)]
impl core::ops::Eq for CannotInitStruct {
    fn eq(self, other: Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
#[cfg(experimental_partial_eq = true)]
impl core::ops::PartialEq for CannotInitStruct {
    fn eq(self, other: Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
#[cfg(experimental_partial_eq = true)]
impl core::ops::Eq for CannotInitStruct {}

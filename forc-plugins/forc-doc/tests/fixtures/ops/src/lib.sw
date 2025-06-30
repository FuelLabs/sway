library;

pub trait Add {
    fn add(self, other: Self) -> Self;
}

pub trait Subtract {
    fn subtract(self, other: Self) -> Self;
}
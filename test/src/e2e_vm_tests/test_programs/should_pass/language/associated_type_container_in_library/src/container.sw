library;

pub trait Container {
    type E;
    fn empty() -> Self;
    fn insert(ref mut self, elem: Self::E);
    fn pop_last(ref mut self) -> Option<Self::E>;
}
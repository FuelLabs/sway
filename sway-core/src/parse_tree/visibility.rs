#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Visibility {
    Private,
    Public,
}

impl Visibility {
    pub fn is_public(&self) -> bool {
        matches!(self, &Visibility::Public)
    }
    pub fn is_private(&self) -> bool {
        !self.is_public()
    }
}

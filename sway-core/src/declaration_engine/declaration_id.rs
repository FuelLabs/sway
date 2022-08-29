use std::fmt;

/// An ID used to refer to an item in the [DeclarationEngine]
#[derive(Clone, Copy, Debug)]
pub struct DeclarationId(usize);

impl fmt::Display for DeclarationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for DeclarationId {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<usize> for DeclarationId {
    fn from(o: usize) -> Self {
        DeclarationId(o)
    }
}

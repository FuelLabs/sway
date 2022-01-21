use crate::Rule;
use pest::iterators::Pair;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub(crate) fn parse_from_pair(input: Pair<Rule>) -> Self {
        match input.as_str().trim() {
            "pub" => Visibility::Public,
            _ => Visibility::Private,
        }
    }
}

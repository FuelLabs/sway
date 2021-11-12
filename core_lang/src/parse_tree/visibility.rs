use crate::Rule;
use pest::iterators::Pair;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Private,
    Public,
}

impl Visibility {
    pub(crate) fn parse_from_pair(input: Pair<'_, Rule>) -> Self {
        match input.as_str().trim() {
            "pub" => Visibility::Public,
            _ => Visibility::Private,
        }
    }
}


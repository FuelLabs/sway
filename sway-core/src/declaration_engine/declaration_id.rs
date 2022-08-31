use std::fmt;

use super::declaration_engine::de_look_up_decl_id;

/// An ID used to refer to an item in the [DeclarationEngine]
#[derive(Clone, Copy, Debug, Eq)]
pub struct DeclarationId(usize);

impl fmt::Display for DeclarationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&de_look_up_decl_id(*self).to_string())
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

impl PartialEq for DeclarationId {
    fn eq(&self, other: &Self) -> bool {
        de_look_up_decl_id(*self) == de_look_up_decl_id(*other)
    }
}

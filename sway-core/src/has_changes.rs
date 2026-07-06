/// Reports whether an operation that mutates declarations ([`crate::language::ty::TyDecl`]),
/// type ids ([`crate::TypeId`]), or other entities in place actually changed anything.
///
/// It is used to propagate "did anything change" information across various
/// in-place transformations, e.g., [`crate::SubstTypes`], declaration replacement,
/// monomorphization, etc.
#[derive(Default)]
pub enum HasChanges {
    Yes,
    #[default]
    No,
}

impl HasChanges {
    pub fn has_changes(&self) -> bool {
        matches!(self, HasChanges::Yes)
    }
}

impl std::ops::BitOr for HasChanges {
    type Output = HasChanges;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (HasChanges::No, HasChanges::No) => HasChanges::No,
            _ => HasChanges::Yes,
        }
    }
}

impl std::ops::BitOrAssign for HasChanges {
    fn bitor_assign(&mut self, rhs: Self) {
        if rhs.has_changes() {
            *self = HasChanges::Yes;
        }
    }
}

impl From<bool> for HasChanges {
    fn from(value: bool) -> Self {
        if value {
            HasChanges::Yes
        } else {
            HasChanges::No
        }
    }
}

#[macro_export]
macro_rules! has_changes {
    ($($stmt:expr);* ;) => {{
        let mut has_changes = $crate::HasChanges::No;
        $(
            has_changes |= $stmt;
        )*
        has_changes
    }};
}

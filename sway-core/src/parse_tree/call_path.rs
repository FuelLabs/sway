use std::fmt;

use crate::Ident;

use sway_types::{span::Span, Spanned};

/// in the expression `a::b::c()`, `a` and `b` are the prefixes and `c` is the suffix.
/// `c` can be any type `T`, but in practice `c` is either an `Ident` or a `TypeInfo`.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CallPath<T> {
    pub prefixes: Vec<Ident>,
    pub suffix: T,
    // If `is_absolute` is true, then this call path is an absolute path from
    // the project root namespace. If not, then it is relative to the current namespace.
    pub(crate) is_absolute: bool,
}

impl std::convert::From<Ident> for CallPath<Ident> {
    fn from(other: Ident) -> Self {
        CallPath {
            prefixes: vec![],
            suffix: other,
            is_absolute: false,
        }
    }
}

impl<T> fmt::Display for CallPath<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buf = String::new();
        for prefix in self.prefixes.iter() {
            buf.push_str(prefix.as_str());
            buf.push_str("::");
        }
        buf.push_str(&self.suffix.to_string());
        write!(f, "{}", buf)
    }
}

impl<T> Spanned for CallPath<T>
where
    T: Spanned,
{
    fn span(&self) -> Span {
        if self.prefixes.is_empty() {
            self.suffix.span()
        } else {
            let prefixes_span = self
                .prefixes
                .iter()
                .fold(self.prefixes[0].span(), |acc, sp| {
                    Span::join(acc, sp.span())
                });
            Span::join(prefixes_span, self.suffix.span())
        }
    }
}

impl CallPath<Ident> {
    /// shifts the last prefix into the suffix and removes the old suffix
    /// noop if prefixes are empty
    pub fn rshift(&self) -> CallPath<Ident> {
        if self.prefixes.is_empty() {
            self.clone()
        } else {
            CallPath {
                prefixes: self.prefixes[0..self.prefixes.len() - 1].to_vec(),
                suffix: self.prefixes.last().unwrap().clone(),
                is_absolute: self.is_absolute,
            }
        }
    }

    pub(crate) fn full_path(&self) -> impl Iterator<Item = &Ident> {
        self.prefixes.iter().chain(Some(&self.suffix))
    }
}

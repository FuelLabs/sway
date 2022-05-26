use crate::Ident;

use sway_types::span::Span;

/// in the expression `a::b::c()`, `a` and `b` are the prefixes and `c` is the suffix.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CallPath {
    pub prefixes: Vec<Ident>,
    pub suffix: Ident,
    // If `is_absolute` is true, then this call path is an absolute path from
    // the project root namespace. If not, then it is relative to the current namespace.
    pub(crate) is_absolute: bool,
}

impl std::convert::From<Ident> for CallPath {
    fn from(other: Ident) -> Self {
        CallPath {
            prefixes: vec![],
            suffix: other,
            is_absolute: false,
        }
    }
}

use std::fmt;
impl fmt::Display for CallPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buf = self.prefixes.iter().map(|x| x.as_str()).collect::<Vec<_>>();
        let suffix = self.suffix.as_str();
        buf.push(suffix);

        write!(f, "{}", buf.join("::"))
    }
}
impl CallPath {
    /// shifts the last prefix into the suffix and removes the old suffix
    /// noop if prefixes are empty
    pub fn rshift(&self) -> CallPath {
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
}
impl CallPath {
    pub(crate) fn span(&self) -> Span {
        if self.prefixes.is_empty() {
            self.suffix.span().clone()
        } else {
            let prefixes_span = self
                .prefixes
                .iter()
                .fold(self.prefixes[0].span().clone(), |acc, sp| {
                    Span::join(acc, sp.span().clone())
                });
            Span::join(prefixes_span, self.suffix.span().clone())
        }
    }

    pub(crate) fn full_path(&self) -> impl Iterator<Item = &Ident> {
        self.prefixes.iter().chain(Some(&self.suffix))
    }

    pub(crate) fn friendly_name(&self) -> String {
        let mut buf = String::new();
        for prefix in self.prefixes.iter() {
            buf.push_str(prefix.as_str());
            buf.push_str("::");
        }
        buf.push_str(self.suffix.as_str());
        buf
    }
}

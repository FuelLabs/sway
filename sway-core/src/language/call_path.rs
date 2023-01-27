use std::fmt;

use crate::{Ident, Namespace};

use sway_types::{span::Span, Spanned};

/// in the expression `a::b::c()`, `a` and `b` are the prefixes and `c` is the suffix.
/// `c` can be any type `T`, but in practice `c` is either an `Ident` or a `TypeInfo`.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct CallPath<T = Ident> {
    pub prefixes: Vec<Ident>,
    pub suffix: T,
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
        write!(f, "{buf}")
    }
}

impl<T: Spanned> Spanned for CallPath<T> {
    fn span(&self) -> Span {
        if self.prefixes.is_empty() {
            self.suffix.span()
        } else {
            let prefixes_spans = self
                .prefixes
                .iter()
                .map(|x| x.span())
                //LOC below should be removed when #21 goes in
                .filter(|x| x.path() == self.suffix.span().path());
            Span::join(Span::join_all(prefixes_spans), self.suffix.span())
        }
    }
}

impl CallPath {
    /// Shifts the last prefix into the suffix, and removes the old suffix.
    /// Does nothing if prefixes are empty.
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

    pub fn to_fullpath(&self, namespace: &mut Namespace) -> CallPath {
        if self.is_absolute {
            return self.clone();
        }

        let mut full_trait_name = self.clone();
        if self.prefixes.is_empty() {
            let mut synonym_prefixes = vec![];
            let mut submodule_name_in_synonym_prefixes = false;

            if let Some(use_synonym) = namespace.use_synonyms.get(&self.suffix) {
                synonym_prefixes = use_synonym.0.clone();
                let submodule = namespace.submodule(&[use_synonym.0[0].clone()]);
                if let Some(submodule) = submodule {
                    if let Some(submodule_name) = submodule.name.clone() {
                        submodule_name_in_synonym_prefixes =
                            submodule_name.as_str() == synonym_prefixes[0].as_str();
                    }
                }
            }

            let mut prefixes: Vec<Ident> = vec![];

            if synonym_prefixes.is_empty() || !submodule_name_in_synonym_prefixes {
                if let Some(pkg_name) = &namespace.root().module.name {
                    prefixes.push(pkg_name.clone());
                }
                for mod_path in namespace.mod_path() {
                    prefixes.push(mod_path.clone());
                }
            }
            prefixes.extend(synonym_prefixes);

            full_trait_name = CallPath {
                prefixes,
                suffix: self.suffix.clone(),
                is_absolute: true,
            }
        }
        full_trait_name
    }
}

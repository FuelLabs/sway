use std::{fmt, sync::Arc};

use crate::{Ident, Namespace};

use sway_types::{span::Span, Spanned};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CallPathTree {
    pub call_path: CallPath,
    pub children: Vec<CallPathTree>,
}

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
            let mut prefixes_spans = self
                .prefixes
                .iter()
                .map(|x| x.span())
                //LOC below should be removed when #21 goes in
                .filter(|x| {
                    Arc::ptr_eq(x.src(), self.suffix.span().src())
                        && x.path() == self.suffix.span().path()
                })
                .peekable();
            if prefixes_spans.peek().is_some() {
                Span::join(Span::join_all(prefixes_spans), self.suffix.span())
            } else {
                self.suffix.span()
            }
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

    pub fn as_vec_string(&self) -> Vec<String> {
        self.prefixes
            .iter()
            .map(|p| p.to_string())
            .chain(std::iter::once(self.suffix.to_string()))
            .collect::<Vec<_>>()
    }

    /// Convert a given `CallPath` to an symbol to a full `CallPath` from the root of the project
    /// in which the symbol is declared. For example, given a path `pkga::SOME_CONST` where `pkga`
    /// is an _internal_ library of a package named `my_project`, the corresponding call path is
    /// `my_project::pkga::SOME_CONST`.
    ///
    /// Paths to _external_ libraries such `std::lib1::lib2::my_obj` are considered full already
    /// and are left unchanged since `std` is a root of the package `std`.
    pub fn to_fullpath(&self, namespace: &Namespace) -> CallPath {
        if self.is_absolute {
            return self.clone();
        }

        if self.prefixes.is_empty() {
            // Given a path to a symbol that has no prefixes, discover the path to the symbol as a
            // combination of the package name in which the symbol is defined and the path to the
            // current submodule.
            let mut synonym_prefixes = vec![];
            let mut is_external = false;

            if let Some(use_synonym) = namespace.use_synonyms.get(&self.suffix) {
                synonym_prefixes = use_synonym.0.clone();
                let submodule = namespace.submodule(&[use_synonym.0[0].clone()]);
                if let Some(submodule) = submodule {
                    is_external = submodule.is_external;
                }
            }

            let mut prefixes: Vec<Ident> = vec![];

            if !is_external {
                if let Some(pkg_name) = &namespace.root().module.name {
                    prefixes.push(pkg_name.clone());
                }

                for mod_path in namespace.mod_path() {
                    prefixes.push(mod_path.clone());
                }
            }

            prefixes.extend(synonym_prefixes);

            CallPath {
                prefixes,
                suffix: self.suffix.clone(),
                is_absolute: true,
            }
        } else if let Some(m) = namespace.submodule(&[self.prefixes[0].clone()]) {
            // If some prefixes are already present, attempt to complete the path by adding the
            // package name and the path to the current submodule.
            //
            // If the path starts with an external module (i.e. a module that is imported in
            // `Forc.toml`, then do not change it since it's a complete path already.
            if m.is_external {
                self.clone()
            } else {
                let mut prefixes: Vec<Ident> = vec![];
                if let Some(pkg_name) = &namespace.root().module.name {
                    prefixes.push(pkg_name.clone());
                }
                for mod_path in namespace.mod_path() {
                    prefixes.push(mod_path.clone());
                }

                prefixes.extend(self.prefixes.clone());

                CallPath {
                    prefixes,
                    suffix: self.suffix.clone(),
                    is_absolute: true,
                }
            }
        } else {
            self.clone()
        }
    }
}

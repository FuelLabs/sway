use std::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

use crate::{
    engine_threading::{
        DebugWithEngines, DisplayWithEngines, EqWithEngines, HashWithEngines, OrdWithEngines,
        OrdWithEnginesContext, PartialEqWithEngines, PartialEqWithEnginesContext,
    },
    Engines, Ident, Namespace,
};

use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{span::Span, Spanned};

use super::parsed::QualifiedPathType;

#[derive(Clone, Debug)]
pub struct CallPathTree {
    pub qualified_call_path: QualifiedCallPath,
    pub children: Vec<CallPathTree>,
}

impl HashWithEngines for CallPathTree {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let CallPathTree {
            qualified_call_path,
            children,
        } = self;
        qualified_call_path.hash(state, engines);
        children.hash(state, engines);
    }
}

impl EqWithEngines for CallPathTree {}
impl PartialEqWithEngines for CallPathTree {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        let CallPathTree {
            qualified_call_path,
            children,
        } = self;
        qualified_call_path.eq(&other.qualified_call_path, ctx) && children.eq(&other.children, ctx)
    }
}

impl<T: PartialEqWithEngines> EqWithEngines for Vec<T> {}
impl<T: PartialEqWithEngines> PartialEqWithEngines for Vec<T> {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        if self.len() != other.len() {
            return false;
        }
        self.iter().zip(other.iter()).all(|(a, b)| a.eq(b, ctx))
    }
}

impl OrdWithEngines for CallPathTree {
    fn cmp(&self, other: &Self, ctx: &OrdWithEnginesContext) -> Ordering {
        let CallPathTree {
            qualified_call_path: l_call_path,
            children: l_children,
        } = self;
        let CallPathTree {
            qualified_call_path: r_call_path,
            children: r_children,
        } = other;
        l_call_path
            .cmp(r_call_path, ctx)
            .then_with(|| l_children.cmp(r_children, ctx))
    }
}

#[derive(Clone, Debug)]

pub struct QualifiedCallPath {
    pub call_path: CallPath,
    pub qualified_path_root: Option<Box<QualifiedPathType>>,
}

impl std::convert::From<Ident> for QualifiedCallPath {
    fn from(other: Ident) -> Self {
        QualifiedCallPath {
            call_path: CallPath {
                prefixes: vec![],
                suffix: other,
                is_absolute: false,
            },
            qualified_path_root: None,
        }
    }
}

impl std::convert::From<CallPath> for QualifiedCallPath {
    fn from(other: CallPath) -> Self {
        QualifiedCallPath {
            call_path: other,
            qualified_path_root: None,
        }
    }
}

impl QualifiedCallPath {
    pub fn to_call_path(self, handler: &Handler) -> Result<CallPath, ErrorEmitted> {
        if let Some(qualified_path_root) = self.qualified_path_root {
            Err(handler.emit_err(CompileError::Internal(
                "Unexpected qualified path.",
                qualified_path_root.as_trait_span,
            )))
        } else {
            Ok(self.call_path)
        }
    }
}

impl HashWithEngines for QualifiedCallPath {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let QualifiedCallPath {
            call_path,
            qualified_path_root,
        } = self;
        call_path.hash(state);
        qualified_path_root.hash(state, engines);
    }
}

impl EqWithEngines for QualifiedCallPath {}
impl PartialEqWithEngines for QualifiedCallPath {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        let QualifiedCallPath {
            call_path,
            qualified_path_root,
        } = self;
        PartialEqWithEngines::eq(call_path, &other.call_path, ctx)
            && qualified_path_root.eq(&other.qualified_path_root, ctx)
    }
}

impl OrdWithEngines for QualifiedCallPath {
    fn cmp(&self, other: &Self, ctx: &OrdWithEnginesContext) -> Ordering {
        let QualifiedCallPath {
            call_path: l_call_path,
            qualified_path_root: l_qualified_path_root,
        } = self;
        let QualifiedCallPath {
            call_path: r_call_path,
            qualified_path_root: r_qualified_path_root,
        } = other;
        l_call_path
            .cmp(r_call_path)
            .then_with(|| l_qualified_path_root.cmp(r_qualified_path_root, ctx))
    }
}

impl DisplayWithEngines for QualifiedCallPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        if let Some(qualified_path_root) = &self.qualified_path_root {
            write!(
                f,
                "{}::{}",
                engines.help_out(qualified_path_root),
                &self.call_path
            )
        } else {
            write!(f, "{}", &self.call_path)
        }
    }
}

impl DebugWithEngines for QualifiedCallPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(f, "{}", engines.help_out(self))
    }
}

/// In the expression `a::b::c()`, `a` and `b` are the prefixes and `c` is the suffix.
/// `c` can be any type `T`, but in practice `c` is either an `Ident` or a `TypeInfo`.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct CallPath<T = Ident> {
    pub prefixes: Vec<Ident>,
    pub suffix: T,
    // If `is_absolute` is true, then this call path is an absolute path from
    // the project root namespace. If not, then it is relative to the current namespace.
    pub is_absolute: bool,
}

impl EqWithEngines for CallPath {}
impl PartialEqWithEngines for CallPath {
    fn eq(&self, other: &Self, _ctx: &PartialEqWithEnginesContext) -> bool {
        self.prefixes == other.prefixes
            && self.suffix == other.suffix
            && self.is_absolute == other.is_absolute
    }
}

impl<T: EqWithEngines> EqWithEngines for CallPath<T> {}
impl<T: PartialEqWithEngines> PartialEqWithEngines for CallPath<T> {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.prefixes == other.prefixes
            && self.suffix.eq(&other.suffix, ctx)
            && self.is_absolute == other.is_absolute
    }
}

impl<T: OrdWithEngines> OrdWithEngines for CallPath<T> {
    fn cmp(&self, other: &Self, ctx: &OrdWithEnginesContext) -> Ordering {
        self.prefixes
            .cmp(&other.prefixes)
            .then_with(|| self.suffix.cmp(&other.suffix, ctx))
            .then_with(|| self.is_absolute.cmp(&other.is_absolute))
    }
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
        for prefix in self.prefixes.iter() {
            write!(f, "{}::", prefix.as_str())?;
        }
        write!(f, "{}", &self.suffix)
    }
}

impl<T: DisplayWithEngines> DisplayWithEngines for CallPath<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        for prefix in self.prefixes.iter() {
            write!(f, "{}::", prefix.as_str())?;
        }
        write!(f, "{}", engines.help_out(&self.suffix))
    }
}

impl<T: DisplayWithEngines> DebugWithEngines for CallPath<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        for prefix in self.prefixes.iter() {
            write!(f, "{}::", prefix.as_str())?;
        }
        write!(f, "{}", engines.help_out(&self.suffix))
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
                        && x.source_id() == self.suffix.span().source_id()
                })
                .peekable();
            if prefixes_spans.peek().is_some() {
                Span::join(Span::join_all(prefixes_spans), &self.suffix.span())
            } else {
                self.suffix.span()
            }
        }
    }
}

impl CallPath {
    pub fn absolute(path: &[&str]) -> Self {
        assert!(!path.is_empty());

        CallPath {
            prefixes: path
                .iter()
                .take(path.len() - 1)
                .map(|&x| Ident::new_no_span(x.into()))
                .collect(),
            suffix: path.last().map(|&x| Ident::new_no_span(x.into())).unwrap(),
            is_absolute: true,
        }
    }

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

    /// Removes the first prefix. Does nothing if prefixes are empty.
    pub fn lshift(&self) -> CallPath {
        if self.prefixes.is_empty() {
            self.clone()
        } else {
            CallPath {
                prefixes: self.prefixes[1..self.prefixes.len()].to_vec(),
                suffix: self.suffix.clone(),
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

    /// Convert a given [CallPath] to a symbol to a full [CallPath] from the root of the project
    /// in which the symbol is declared. For example, given a path `pkga::SOME_CONST` where `pkga`
    /// is an _internal_ library of a package named `my_project`, the corresponding call path is
    /// `my_project::pkga::SOME_CONST`.
    ///
    /// Paths to _external_ libraries such `std::lib1::lib2::my_obj` are considered full already
    /// and are left unchanged since `std` is a root of the package `std`.
    pub fn to_fullpath(&self, engines: &Engines, namespace: &Namespace) -> CallPath {
        if self.is_absolute {
            return self.clone();
        }

        if self.prefixes.is_empty() {
            // Given a path to a symbol that has no prefixes, discover the path to the symbol as a
            // combination of the package name in which the symbol is defined and the path to the
            // current submodule.
            let mut synonym_prefixes = vec![];
            let mut is_external = false;
            let mut is_absolute = false;

            if let Some(mod_path) = namespace.program_id(engines).read(engines, |m| {
                if let Some((_, path, _)) = m
                    .current_items()
                    .use_item_synonyms
                    .get(&self.suffix)
                    .cloned()
                {
                    Some(path)
                } else if let Some(paths_and_decls) = m
                    .current_items()
                    .use_glob_synonyms
                    .get(&self.suffix)
                    .cloned()
                {
                    if paths_and_decls.len() == 1 {
                        Some(paths_and_decls[0].0.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }) {
                synonym_prefixes.clone_from(&mod_path);
                is_absolute = true;
                let submodule = namespace
                    .module(engines)
                    .submodule(engines, &[mod_path[0].clone()]);
                if let Some(submodule) = submodule {
                    is_external = submodule.read(engines, |m| m.is_external);
                }
            }

            let mut prefixes: Vec<Ident> = vec![];

            if !is_external {
                if let Some(pkg_name) = &namespace.root_module().name {
                    prefixes.push(pkg_name.clone());
                }

                if !is_absolute {
                    for mod_path in namespace.mod_path() {
                        prefixes.push(mod_path.clone());
                    }
                }
            }

            prefixes.extend(synonym_prefixes);

            CallPath {
                prefixes,
                suffix: self.suffix.clone(),
                is_absolute: true,
            }
        } else if let Some(m) = namespace
            .module(engines)
            .submodule(engines, &[self.prefixes[0].clone()])
        {
            // If some prefixes are already present, attempt to complete the path by adding the
            // package name and the path to the current submodule.
            //
            // If the path starts with an external module (i.e. a module that is imported in
            // `Forc.toml`), then do not change it since it's a complete path already.
            if m.read(engines, |m| m.is_external) {
                CallPath {
                    prefixes: self.prefixes.clone(),
                    suffix: self.suffix.clone(),
                    is_absolute: true,
                }
            } else {
                let mut prefixes: Vec<Ident> = vec![];
                if let Some(pkg_name) = &namespace.root_module().read(engines, |m| m.name.clone()) {
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
            CallPath {
                prefixes: self.prefixes.clone(),
                suffix: self.suffix.clone(),
                is_absolute: true,
            }
        }
    }

    /// Convert a given [CallPath] into a call path suitable for a `use` statement.
    ///
    /// For example, given a path `pkga::SOME_CONST` where `pkga` is an _internal_ library of a package named
    /// `my_project`, the corresponding call path is `pkga::SOME_CONST`.
    ///
    /// Paths to _external_ libraries such `std::lib1::lib2::my_obj` are left unchanged.
    pub fn to_import_path(&self, engines: &Engines, namespace: &Namespace) -> CallPath {
        let converted = self.to_fullpath(engines, namespace);

        if let Some(first) = converted.prefixes.first() {
            if namespace.root_module().read(engines, |m| m.name.clone()) == Some(first.clone()) {
                return converted.lshift();
            }
        }
        converted
    }
}

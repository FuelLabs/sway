use crate::priv_prelude::*;

pub struct Dependency {
    pub dep_token: DepToken,
    pub path: DependencyPath,
    pub semicolon_token: SemicolonToken,
}

pub struct DependencyPath {
    pub prefix: Ident,
    pub suffix: Option<(ForwardSlashToken, Box<DependencyPath>)>,
}

impl DependencyPath {
    pub fn iter(&self) -> DependencyPathIter<'_> {
        DependencyPathIter {
            dependency_path_opt: Some(self),
        }
    }
}

pub struct DependencyPathIter<'a> {
    dependency_path_opt: Option<&'a DependencyPath>,
}

impl<'a> Iterator for DependencyPathIter<'a> {
    type Item = &'a Ident;

    fn next(&mut self) -> Option<&'a Ident> {
        let DependencyPath { prefix, suffix } = self.dependency_path_opt?;
        match suffix {
            Some((_forward_slash_token, tail)) => self.dependency_path_opt = Some(tail),
            None => self.dependency_path_opt = None,
        }
        Some(prefix)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut dependency_path = match self.dependency_path_opt {
            Some(dependency_path) => dependency_path,
            None => return (0, Some(0)),
        };
        let mut acc = 0;
        let len = loop {
            acc += 1;
            match &dependency_path.suffix {
                None => break acc,
                Some((_forward_slash_token, tail)) => dependency_path = &*tail,
            }
        };
        (len, Some(len))
    }
}

impl<'a> ExactSizeIterator for DependencyPathIter<'a> {}
impl<'a> iter::FusedIterator for DependencyPathIter<'a> {}

impl Spanned for Dependency {
    fn span(&self) -> Span {
        Span::join(self.dep_token.span(), self.semicolon_token.span())
    }
}

pub fn dependency() -> impl Parser<char, Dependency, Error = Cheap<char, Span>> + Clone {
    dep_token()
    .then_whitespace()
    .then(dependency_path())
    .then_optional_whitespace()
    .then(semicolon_token())
    .map(|((dep_token, path), semicolon_token)| {
        Dependency { dep_token, path, semicolon_token }
    })
}

pub fn dependency_path() -> impl Parser<char, DependencyPath, Error = Cheap<char, Span>> + Clone {
    recursive(|recurse| {
        ident()
        .then(forward_slash_token().then(recurse.map(Box::new)).or_not())
        .map(|(prefix, suffix)| DependencyPath { prefix, suffix })
    })
}



use crate::priv_prelude::*;

#[derive(Clone, Debug, Serialize)]
pub struct PathExpr {
    pub root_opt: Option<(Option<AngleBrackets<QualifiedPathRoot>>, DoubleColonToken)>,
    pub prefix: PathExprSegment,
    pub suffix: Vec<(DoubleColonToken, PathExprSegment)>,
    // path expression with incomplete suffix are needed to do
    // parser recovery on inputs like foo::
    #[serde(skip_serializing)]
    pub incomplete_suffix: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct PathExprSegment {
    pub name: Ident,
    pub generics_opt: Option<(DoubleColonToken, GenericArgs)>,
}

impl PathExpr {
    pub fn last_segment(&self) -> &PathExprSegment {
        self.suffix
            .iter()
            .map(|s| &s.1)
            .next_back()
            .unwrap_or(&self.prefix)
    }

    pub fn last_segment_mut(&mut self) -> &mut PathExprSegment {
        self.suffix
            .iter_mut()
            .map(|s| &mut s.1)
            .next_back()
            .unwrap_or(&mut self.prefix)
    }
}

impl Spanned for PathExpr {
    fn span(&self) -> Span {
        let start = match &self.root_opt {
            Some((qualified_path_root_opt, double_colon_token)) => match qualified_path_root_opt {
                Some(qualified_path_root) => qualified_path_root.span(),
                None => double_colon_token.span(),
            },
            None => self.prefix.span(),
        };
        let end = match self.suffix.last() {
            Some((_, path_expr_segment)) => path_expr_segment.span(),
            None => self.prefix.span(),
        };
        Span::join(start, &end)
    }
}

impl PathExpr {
    #[allow(clippy::result_large_err)]
    pub fn try_into_ident(self) -> Result<Ident, PathExpr> {
        if self.root_opt.is_none()
            && self.suffix.is_empty()
            && self.prefix.generics_opt.is_none()
            && !self.incomplete_suffix
        {
            return Ok(self.prefix.name);
        }
        Err(self)
    }
}

impl Spanned for PathExprSegment {
    fn span(&self) -> Span {
        let start = self.name.span();
        match &self.generics_opt {
            Some((_, generic_args)) => Span::join(start, &generic_args.span()),
            None => start,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct PathType {
    pub root_opt: Option<(Option<AngleBrackets<QualifiedPathRoot>>, DoubleColonToken)>,
    pub prefix: PathTypeSegment,
    pub suffix: Vec<(DoubleColonToken, PathTypeSegment)>,
}

impl PathType {
    pub fn last_segment(&self) -> &PathTypeSegment {
        self.suffix
            .iter()
            .map(|s| &s.1)
            .next_back()
            .unwrap_or(&self.prefix)
    }

    pub fn last_segment_mut(&mut self) -> &mut PathTypeSegment {
        self.suffix
            .iter_mut()
            .map(|s| &mut s.1)
            .next_back()
            .unwrap_or(&mut self.prefix)
    }
}

impl Spanned for PathType {
    fn span(&self) -> Span {
        let start = match &self.root_opt {
            Some((qualified_path_root_opt, double_colon_token)) => match qualified_path_root_opt {
                Some(qualified_path_root) => qualified_path_root.span(),
                None => double_colon_token.span(),
            },
            None => self.prefix.span(),
        };
        let end = match self.suffix.last() {
            Some((_, path_type_segment)) => path_type_segment.span(),
            None => self.prefix.span(),
        };
        Span::join(start, &end)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct PathTypeSegment {
    pub name: Ident,
    pub generics_opt: Option<(Option<DoubleColonToken>, GenericArgs)>,
}

impl Spanned for PathTypeSegment {
    fn span(&self) -> Span {
        let start = self.name.span();
        match &self.generics_opt {
            Some((_, generic_args)) => Span::join(start, &generic_args.span()),
            None => start,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct QualifiedPathRoot {
    pub ty: Box<Ty>,
    pub as_trait: (AsToken, Box<PathType>),
}

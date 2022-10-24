use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct PathExpr {
    pub root_opt: Option<(Option<AngleBrackets<QualifiedPathRoot>>, DoubleColonToken)>,
    pub prefix:   PathExprSegment,
    pub suffix:   Vec<(DoubleColonToken, PathExprSegment)>,
}

#[derive(Clone, Debug)]
pub struct PathExprSegment {
    pub fully_qualified: Option<TildeToken>,
    pub name:            Ident,
    pub generics_opt:    Option<(DoubleColonToken, GenericArgs)>,
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
        Span::join(start, end)
    }
}

impl PathExpr {
    pub fn try_into_ident(self) -> Result<Ident, PathExpr> {
        if self.root_opt.is_none()
            && self.suffix.is_empty()
            && self.prefix.fully_qualified.is_none()
            && self.prefix.generics_opt.is_none()
        {
            return Ok(self.prefix.name);
        }
        Err(self)
    }
}

impl Spanned for PathExprSegment {
    fn span(&self) -> Span {
        let start = match &self.fully_qualified {
            Some(tilde_token) => tilde_token.span(),
            None => self.name.span(),
        };
        let end = match &self.generics_opt {
            Some((_, generic_args)) => generic_args.span(),
            None => self.name.span(),
        };
        Span::join(start, end)
    }
}

#[derive(Clone, Debug)]
pub struct PathType {
    pub root_opt: Option<(Option<AngleBrackets<QualifiedPathRoot>>, DoubleColonToken)>,
    pub prefix:   PathTypeSegment,
    pub suffix:   Vec<(DoubleColonToken, PathTypeSegment)>,
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
        Span::join(start, end)
    }
}

#[derive(Clone, Debug)]
pub struct PathTypeSegment {
    pub fully_qualified: Option<TildeToken>,
    pub name:            Ident,
    pub generics_opt:    Option<(Option<DoubleColonToken>, GenericArgs)>,
}

impl Spanned for PathTypeSegment {
    fn span(&self) -> Span {
        let start = match &self.fully_qualified {
            Some(tilde_token) => tilde_token.span(),
            None => self.name.span(),
        };
        let end = match &self.generics_opt {
            Some((_, generic_args)) => generic_args.span(),
            None => self.name.span(),
        };
        Span::join(start, end)
    }
}

#[derive(Clone, Debug)]
pub struct QualifiedPathRoot {
    pub ty:       Box<Ty>,
    pub as_trait: Option<(AsToken, Box<PathType>)>,
}

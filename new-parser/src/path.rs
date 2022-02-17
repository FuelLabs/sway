use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct PathExpr {
    pub root_opt: Option<(Option<AngleBrackets<QualifiedPathRoot>>, DoubleColonToken)>,
    pub prefix: PathExprSegment,
    pub suffix: Vec<(DoubleColonToken, PathExprSegment)>,
}

#[derive(Clone, Debug)]
pub struct PathExprSegment {
    pub fully_qualified: Option<TildeToken>,
    pub name: Ident,
    pub generics_opt: Option<(DoubleColonToken, GenericArgs)>,
}

#[derive(Clone, Debug)]
pub struct PathType {
    pub root_opt: Option<(Option<AngleBrackets<QualifiedPathRoot>>, DoubleColonToken)>,
    pub prefix: PathTypeSegment,
    pub suffix: Vec<(DoubleColonToken, PathTypeSegment)>,
}

#[derive(Clone, Debug)]
pub struct PathTypeSegment {
    pub fully_qualified: Option<TildeToken>,
    pub name: Ident,
    pub generics_opt: Option<(Option<DoubleColonToken>, GenericArgs)>,
}

#[derive(Clone, Debug)]
pub struct QualifiedPathRoot {
    pub ty: Box<Ty>,
    pub as_trait: Option<(AsToken, Box<PathType>)>,
}

impl Spanned for PathExpr {
    fn span(&self) -> Span {
        let start = match &self.root_opt {
            Some((qualified_path_root_opt, double_colon_token)) => {
                match qualified_path_root_opt {
                    Some(qualified_path_root) => qualified_path_root.span(),
                    None => double_colon_token.span(),
                }
            },
            None => self.prefix.span(),
        };
        let end = match self.suffix.last() {
            Some((_, segment)) => segment.span(),
            None => self.prefix.span(),
        };
        Span::join(start, end)
    }
}

impl Spanned for PathExprSegment {
    fn span(&self) -> Span {
        let start = match &self.fully_qualified {
            Some(tilde_token) => tilde_token.span(),
            None => self.name.span(),
        };
        let end = match &self.generics_opt {
            Some((_, generics)) => generics.span(),
            None => self.name.span(),
        };
        Span::join(start, end)
    }
}

impl Spanned for PathType {
    fn span(&self) -> Span {
        let start = match &self.root_opt {
            Some((qualified_path_root_opt, double_colon_token)) => {
                match qualified_path_root_opt {
                    Some(qualified_path_root) => qualified_path_root.span(),
                    None => double_colon_token.span(),
                }
            },
            None => self.prefix.span(),
        };
        let end = match self.suffix.last() {
            Some((_, segment)) => segment.span(),
            None => self.prefix.span(),
        };
        Span::join(start, end)
    }
}

impl Spanned for PathTypeSegment {
    fn span(&self) -> Span {
        let start = match &self.fully_qualified {
            Some(tilde_token) => tilde_token.span(),
            None => self.name.span(),
        };
        let end = match &self.generics_opt {
            Some((_, generics)) => generics.span(),
            None => self.name.span(),
        };
        Span::join(start, end)
    }
}

pub fn path_expr() -> impl Parser<Output = PathExpr> + Clone {
    angle_brackets(qualified_path_root())
    .then_optional_whitespace()
    .optional()
    .then(double_colon_token())
    .then_optional_whitespace()
    .optional()
    .then(path_expr_segment())
    .then(
        optional_leading_whitespace(double_colon_token())
        .then(optional_leading_whitespace(path_expr_segment()))
        .repeated()
    )
    .map(|((root_opt, prefix), suffix)| {
        PathExpr { root_opt, prefix, suffix }
    })
}

pub fn path_expr_segment() -> impl Parser<Output = PathExprSegment> + Clone {
    tilde_token()
    .then_optional_whitespace()
    .optional()
    .then(ident())
    .then(
        optional_leading_whitespace(
            double_colon_token()
            .then_optional_whitespace()
            .then(generic_args())
        )
        .optional()
    )
    .map(|((fully_qualified, name), generics_opt)| {
        PathExprSegment { fully_qualified, name, generics_opt }
    })
}

pub fn path_type() -> impl Parser<Output = PathType> + Clone {
    angle_brackets(qualified_path_root())
    .then_optional_whitespace()
    .optional()
    .then(double_colon_token())
    .then_optional_whitespace()
    .optional()
    .then(path_type_segment())
    .then(
        optional_leading_whitespace(double_colon_token())
        .then(optional_leading_whitespace(path_type_segment()))
        .repeated()
    )
    .map(|((root_opt, prefix), suffix)| {
        PathType { root_opt, prefix, suffix }
    })
}

pub fn path_type_segment() -> impl Parser<Output = PathTypeSegment> + Clone {
    tilde_token()
    .then_optional_whitespace()
    .optional()
    .then(ident())
    .then(
        optional_leading_whitespace(
            double_colon_token()
            .then_optional_whitespace()
            .optional()
            .then(lazy(|| generic_args()))
        )
        .optional()
    )
    .map(|((fully_qualified, name), generics_opt)| {
        PathTypeSegment { fully_qualified, name, generics_opt }
    })
}

pub fn qualified_path_root() -> impl Parser<Output = QualifiedPathRoot> + Clone {
    lazy(|| ty())
    .map(Box::new)
    .then(
        leading_whitespace(
            as_token()
            .then_whitespace()
            .then(lazy(|| path_type()).map(Box::new))
        )
        .optional()
    )
    .map(|(ty, as_trait)| {
        QualifiedPathRoot { ty, as_trait }
    })
}


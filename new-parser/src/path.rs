use crate::priv_prelude::*;

/*
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
*/

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

/*
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
*/

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

/*
pub struct ExpectedPathExprError {
    position: usize,
}

pub enum PathExprFatalError {
}

pub fn path_expr()
    -> impl Parser<Output = PathExpr, Error = ExpectedPathExprError, FatalError = PathExprFatalError> + Clone
{
    angle_brackets(qualified_path_root())
    .map_err(|AngleBracketsError { .. }| ())
    .then_optional_whitespace()
    .optional()
    .then(double_colon_token().map_err(|ExpectedDoubleColonTokenError { .. }| ()))
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

pub struct ExpectedPathExprError {
    position: usize,
}

pub enum PathExprFatalError {
}

pub fn path_expr_segment()
    -> impl Parser<Output = PathExprSegment, Error = ExpectPathExprSegmentError, FatalError = PathExprSegmentFatalError> + Clone
{
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
*/

#[derive(Clone)]
pub struct ExpectedPathTypeError {
    pub position: usize,
}
#[derive(Clone)]
pub enum PathTypeFatalError {
    ExpectedQualifiedPathRoot(ExpectedQualifiedPathRootError),
    QualifiedPathRoot(QualifiedPathRootFatalError),
    ExpectedCloseAngleBracket { position: usize },
    UnclosedMultilineComment(UnclosedMultilineCommentError),
}

pub fn path_type()
    -> impl Parser<
        Output = PathType,
        Error = ExpectedPathTypeError,
        FatalError = PathTypeFatalError,
    > + Clone {
    angle_brackets(qualified_path_root())
    .map_err(|AngleBracketsError { .. }| ())
    .map_fatal_err(|error| match error {
        AngleBracketsFatalError::Inner(error) => PathTypeFatalError::ExpectedQualifiedPathRoot(error),
        AngleBracketsFatalError::InnerFatal(error) => PathTypeFatalError::QualifiedPathRoot(error),
        AngleBracketsFatalError::ExpectedCloseAngleBracket { position } => {
            PathTypeFatalError::ExpectedCloseAngleBracket { position }
        },
    })
    .then_optional_whitespace()
    .map_fatal_err(|error| match error {
        PaddedFatalError::UnclosedMultilineComment(error) => {
            PathTypeFatalError::UnclosedMultilineComment(error)
        },
        PaddedFatalError::Inner(error) => error,
    })
    .optional()
    .then(
        double_colon_token()
        .map_err(|ExpectedDoubleColonTokenError { .. }| ())
    )
    .then_optional_whitespace()
    .map_fatal_err(|error| match error {
        PaddedFatalError::UnclosedMultilineComment(error) => {
            PathTypeFatalError::UnclosedMultilineComment(error)
        },
        PaddedFatalError::Inner(error) => error,
    })
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

#[derive(Clone)]
pub struct ExpectedPathTypeSegmentError {
    pub position: usize,
}

#[derive(Clone)]
pub enum PathTypeSegmentFatalError {
    UnclosedMultilineComment(UnclosedMultilineCommentError),
    ExpectedCloseAngleBracket {
        position: usize,
    },
}

pub fn path_type_segment()
    -> impl Parser<
        Output = PathTypeSegment,
        Error = ExpectedPathTypeSegmentError,
        FatalError = PathTypeSegmentFatalError,
    > + Clone {
    tilde_token()
    .then_optional_whitespace()
    .map_err(|ExpectedTildeTokenError { .. }| ())
    .map_fatal_err(|error| {
        PathTypeSegmentFatalError::UnclosedMultilineComment(error.to_unclosed_multiline_comment_error())
    })
    .optional()
    .then(
        ident()
        .map_err(|ExpectedIdentError { position }| ExpectedPathTypeSegmentError { position })
    )
    .then(
        padded(double_colon_token())
        .map_err(|ExpectedDoubleColonTokenError { .. }| ())
        .map_fatal_err(|error| {
            PathTypeSegmentFatalError::UnclosedMultilineComment(error.to_unclosed_multiline_comment_error())
        })
        .optional()
        .then(
            lazy(|| generic_args())
            .map_err(|ExpectedGenericArgsError { .. }| ())
            .map_fatal_err(|error| match error {
                GenericArgsFatalError::UnclosedMultilineComment(error) => {
                    PathTypeSegmentFatalError::UnclosedMultilineComment(error)
                },
                GenericArgsFatalError::ExpectedCloseAngleBracket { position } => {
                    PathTypeSegmentFatalError::ExpectedCloseAngleBracket { position }
                },
            })
        )
        .optional()
    )
    .map(|((fully_qualified, name), generics_opt)| {
        PathTypeSegment { fully_qualified, name, generics_opt }
    })
}

#[derive(Clone)]
pub struct ExpectedQualifiedPathRootError {
    pub position: usize,
}

#[derive(Clone)]
pub enum QualifiedPathRootFatalError {
    MissingTraitFollowingAs {
        position: usize,
    },
    ParseTraitFatalError(PathTypeFatalError),
    UnclosedMultilineComment(UnclosedMultilineCommentError),
}

pub fn qualified_path_root()
    -> impl Parser<
        Output = QualifiedPathRoot,
        Error = ExpectedQualifiedPathRootError,
        FatalError = QualifiedPathRootFatalError,
    > + Clone
{
    lazy(|| ty())
    .map_err(|ExpectedTypeError { position }| ExpectedQualifiedPathRootError { position })
    .map(Box::new)
    .then(
        padded(as_token())
        .map_err(|ExpectedAsTokenError { .. }| ())
        .map_fatal_err(|error| {
            QualifiedPathRootFatalError::UnclosedMultilineComment(error.to_unclosed_multiline_comment_error())
        })
        .then(
            lazy(|| path_type())
            .map_err(|ExpectedPathTypeError { position }| {
                QualifiedPathRootFatalError::MissingTraitFollowingAs { position }
            })
            .map_fatal_err(QualifiedPathRootFatalError::ParseTraitFatalError)
            .map(Box::new)
            .fatal()
        )
        .optional()
    )
    .map(|(ty, as_trait)| {
        QualifiedPathRoot { ty, as_trait }
    })
}


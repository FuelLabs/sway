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

impl PathExpr {
    pub fn span(&self) -> Span {
        let start = match &self.root_opt {
            Some((qualified_path_root_opt, double_colon_token)) => match qualified_path_root_opt {
                Some(qualified_path_root) => qualified_path_root.span(),
                None => double_colon_token.span(),
            },
            None => self.prefix.span(),
        };
        let end = match self.suffix.last() {
            Some((_double_colon_token, path_expr_segment)) => path_expr_segment.span(),
            None => self.prefix.span(),
        };
        Span::join(start, end)
    }

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

impl PathExprSegment {
    pub fn span(&self) -> Span {
        let start = match &self.fully_qualified {
            Some(tilde_token) => tilde_token.span(),
            None => self.name.span().clone(),
        };
        let end = match &self.generics_opt {
            Some((_double_colon_token, generic_args)) => generic_args.span(),
            None => self.name.span().clone(),
        };
        Span::join(start, end)
    }
}

impl Parse for PathExpr {
    fn parse(parser: &mut Parser) -> ParseResult<PathExpr> {
        let root_opt = match parser.take() {
            Some(open_angle_bracket_token) => {
                let qualified_path_root = parser.parse()?;
                let close_angle_bracket_token = parser.parse()?;
                let angle_brackets = AngleBrackets {
                    open_angle_bracket_token,
                    inner: qualified_path_root,
                    close_angle_bracket_token,
                };
                let double_colon_token = parser.parse()?;
                Some((Some(angle_brackets), double_colon_token))
            }
            None => parser
                .take()
                .map(|double_colon_token| (None, double_colon_token)),
        };
        let prefix = parser.parse()?;
        let mut suffix = Vec::new();
        while let Some(double_colon_token) = parser.take() {
            let segment = parser.parse()?;
            suffix.push((double_colon_token, segment));
        }
        Ok(PathExpr {
            root_opt,
            prefix,
            suffix,
        })
    }
}

impl Parse for PathExprSegment {
    fn parse(parser: &mut Parser) -> ParseResult<PathExprSegment> {
        let fully_qualified = parser.take();
        let name = parser.parse()?;
        let generics_opt = if parser
            .peek2::<DoubleColonToken, OpenAngleBracketToken>()
            .is_some()
        {
            let double_colon_token = parser.parse()?;
            let generics = parser.parse()?;
            Some((double_colon_token, generics))
        } else {
            None
        };
        Ok(PathExprSegment {
            fully_qualified,
            name,
            generics_opt,
        })
    }
}

#[derive(Clone, Debug)]
pub struct PathType {
    pub root_opt: Option<(Option<AngleBrackets<QualifiedPathRoot>>, DoubleColonToken)>,
    pub prefix: PathTypeSegment,
    pub suffix: Vec<(DoubleColonToken, PathTypeSegment)>,
}

impl PathType {
    pub fn span(&self) -> Span {
        let start = match &self.root_opt {
            Some((qualified_path_root_opt, double_colon_token)) => match qualified_path_root_opt {
                Some(qualified_path_root) => qualified_path_root.span(),
                None => double_colon_token.span(),
            },
            None => self.prefix.span(),
        };
        let end = match self.suffix.last() {
            Some((_double_colon_token, path_type_segment)) => path_type_segment.span(),
            None => self.prefix.span(),
        };
        Span::join(start, end)
    }
}

#[derive(Clone, Debug)]
pub struct PathTypeSegment {
    pub fully_qualified: Option<TildeToken>,
    pub name: Ident,
    pub generics_opt: Option<(Option<DoubleColonToken>, GenericArgs)>,
}

impl PathTypeSegment {
    pub fn span(&self) -> Span {
        let start = match &self.fully_qualified {
            Some(tilde_token) => tilde_token.span(),
            None => self.name.span().clone(),
        };
        let end = match &self.generics_opt {
            Some((_double_colon_token, generic_args)) => generic_args.span(),
            None => self.name.span().clone(),
        };
        Span::join(start, end)
    }
}

#[derive(Clone, Debug)]
pub struct QualifiedPathRoot {
    pub ty: Box<Ty>,
    pub as_trait: Option<(AsToken, Box<PathType>)>,
}

impl Parse for PathType {
    fn parse(parser: &mut Parser) -> ParseResult<PathType> {
        let root_opt = match parser.take() {
            Some(open_angle_bracket_token) => {
                let qualified_path_root = parser.parse()?;
                let close_angle_bracket_token = parser.parse()?;
                let angle_brackets = AngleBrackets {
                    open_angle_bracket_token,
                    inner: qualified_path_root,
                    close_angle_bracket_token,
                };
                let double_colon_token = parser.parse()?;
                Some((Some(angle_brackets), double_colon_token))
            }
            None => parser
                .take()
                .map(|double_colon_token| (None, double_colon_token)),
        };
        let prefix = parser.parse()?;
        let mut suffix = Vec::new();
        while let Some(double_colon_token) = parser.take() {
            let segment = parser.parse()?;
            suffix.push((double_colon_token, segment));
        }
        Ok(PathType {
            root_opt,
            prefix,
            suffix,
        })
    }
}

impl Parse for PathTypeSegment {
    fn parse(parser: &mut Parser) -> ParseResult<PathTypeSegment> {
        let fully_qualified = parser.take();
        let name = parser.parse()?;
        let generics_opt = if parser.peek::<OpenAngleBracketToken>().is_some() {
            let generics = parser.parse()?;
            Some((None, generics))
        } else if parser
            .peek2::<DoubleColonToken, OpenAngleBracketToken>()
            .is_some()
        {
            let double_colon_token = parser.parse()?;
            let generics = parser.parse()?;
            Some((Some(double_colon_token), generics))
        } else {
            None
        };
        Ok(PathTypeSegment {
            fully_qualified,
            name,
            generics_opt,
        })
    }
}

impl Parse for QualifiedPathRoot {
    fn parse(parser: &mut Parser) -> ParseResult<QualifiedPathRoot> {
        let ty = parser.parse()?;
        let as_trait = match parser.take() {
            Some(as_token) => {
                let path_type = parser.parse()?;
                Some((as_token, path_type))
            }
            None => None,
        };
        Ok(QualifiedPathRoot { ty, as_trait })
    }
}

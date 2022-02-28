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
    pub fn try_into_ident(self) -> Result<Ident, PathExpr> {
        if {
            self.root_opt.is_none() &&
            self.suffix.is_empty() &&
            self.prefix.fully_qualified.is_none() &&
            self.prefix.generics_opt.is_none()
        } {
            return Ok(self.prefix.name);
        }
        Err(self)
    }
}

impl Parse for PathExpr {
    fn parse(parser: &mut Parser) -> ParseResult<PathExpr> {
        let root_opt = match parser.take() {
            Some(less_than_token) => {
                let qualified_path_root = parser.parse()?;
                let greater_than_token = parser.parse()?;
                let angle_brackets = AngleBrackets {
                    less_than_token,
                    inner: qualified_path_root,
                    greater_than_token,
                };
                let double_colon_token = parser.parse()?;
                Some((Some(angle_brackets), double_colon_token))
            },
            None => {
                match parser.take() {
                    Some(double_colon_token) => {
                        Some((None, double_colon_token))
                    },
                    None => None,
                }
            },
        };
        let prefix = parser.parse()?;
        let mut suffix = Vec::new();
        loop {
            let double_colon_token = match parser.take() {
                Some(double_colon_token) => double_colon_token,
                None => break,
            };
            let segment = parser.parse()?;
            suffix.push((double_colon_token, segment));
        }
        Ok(PathExpr { root_opt, prefix, suffix })
    }
}

impl Parse for PathExprSegment {
    fn parse(parser: &mut Parser) -> ParseResult<PathExprSegment> {
        let fully_qualified = parser.take();
        let name = parser.parse()?;
        let generics_opt = if parser.peek2::<DoubleColonToken, LessThanToken>().is_some() {
            let double_colon_token = parser.parse()?;
            let generics = parser.parse()?;
            Some((double_colon_token, generics))
        } else {
            None
        };
        Ok(PathExprSegment { fully_qualified, name, generics_opt })
    }
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

impl Parse for PathType {
    fn parse(parser: &mut Parser) -> ParseResult<PathType> {
        let root_opt = match parser.take() {
            Some(less_than_token) => {
                let qualified_path_root = parser.parse()?;
                let greater_than_token = parser.parse()?;
                let angle_brackets = AngleBrackets {
                    less_than_token,
                    inner: qualified_path_root,
                    greater_than_token,
                };
                let double_colon_token = parser.parse()?;
                Some((Some(angle_brackets), double_colon_token))
            },
            None => {
                match parser.take() {
                    Some(double_colon_token) => {
                        Some((None, double_colon_token))
                    },
                    None => None,
                }
            },
        };
        let prefix = parser.parse()?;
        let mut suffix = Vec::new();
        loop {
            let double_colon_token = match parser.take() {
                Some(double_colon_token) => double_colon_token,
                None => break,
            };
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
        let generics_opt = if parser.peek::<LessThanToken>().is_some() {
            let generics = parser.parse()?;
            Some((None, generics))
        } else if parser.peek2::<DoubleColonToken, LessThanToken>().is_some() {
            let double_colon_token = parser.parse()?;
            let generics = parser.parse()?;
            Some((Some(double_colon_token), generics))
        } else {
            None
        };
        Ok(PathTypeSegment { fully_qualified, name, generics_opt })
    }
}

impl Parse for QualifiedPathRoot {
    fn parse(parser: &mut Parser) -> ParseResult<QualifiedPathRoot> {
        let ty = parser.parse()?;
        let as_trait = match parser.take() {
            Some(as_token) => {
                let path_type = parser.parse()?;
                Some((as_token, path_type))
            },
            None => None,
        };
        Ok(QualifiedPathRoot { ty, as_trait })
    }
}

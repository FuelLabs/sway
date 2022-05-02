use crate::priv_prelude::*;

macro_rules! define_keyword (
    ($ty_name:ident, $keyword:literal) => {
        #[derive(Clone, Debug)]
        pub struct $ty_name {
            span: Span,
        }

        impl $ty_name {
            pub fn span(&self) -> Span {
                self.span.clone()
            }
        }

        impl Peek for $ty_name {
            fn peek(peeker: Peeker<'_>) -> Option<$ty_name> {
                let ident = peeker.peek_ident().ok()?;
                if ident.as_str() == $keyword {
                    Some($ty_name { span: ident.span().clone() })
                } else {
                    None
                }
            }
        }

        impl Parse for $ty_name {
            fn parse(parser: &mut Parser) -> ParseResult<$ty_name> {
                match parser.take() {
                    Some(value) => Ok(value),
                    None => {
                        Err(parser.emit_error(ParseErrorKind::ExpectedKeyword { word: $keyword }))
                    },
                }
            }
        }
    };
);

define_keyword!(ScriptToken, "script");
define_keyword!(ContractToken, "contract");
define_keyword!(PredicateToken, "predicate");
define_keyword!(LibraryToken, "library");
define_keyword!(DepToken, "dep");
define_keyword!(PubToken, "pub");
define_keyword!(UseToken, "use");
define_keyword!(AsToken, "as");
define_keyword!(StructToken, "struct");
define_keyword!(EnumToken, "enum");
define_keyword!(SelfToken, "self");
define_keyword!(FnToken, "fn");
define_keyword!(ImpureToken, "impure");
define_keyword!(TraitToken, "trait");
define_keyword!(ImplToken, "impl");
define_keyword!(ForToken, "for");
define_keyword!(AbiToken, "abi");
define_keyword!(ConstToken, "const");
define_keyword!(StorageToken, "storage");
define_keyword!(StrToken, "str");
define_keyword!(AsmToken, "asm");
define_keyword!(ReturnToken, "return");
define_keyword!(IfToken, "if");
define_keyword!(ElseToken, "else");
define_keyword!(MatchToken, "match");
define_keyword!(MutToken, "mut");
define_keyword!(LetToken, "let");
define_keyword!(WhileToken, "while");
define_keyword!(WhereToken, "where");
define_keyword!(RefToken, "ref");
define_keyword!(DerefToken, "deref");

macro_rules! define_token (
    ($ty_name:ident, $description:literal, [$($punct_kinds:ident),*], [$($not_followed_by:ident),*]) => {
        #[derive(Clone, Debug)]
        pub struct $ty_name {
            span: Span,
        }

        impl $ty_name {
            pub fn span(&self) -> Span {
                self.span.clone()
            }

            pub fn ident(&self) -> Ident {
                Ident::new(self.span())
            }
        }

        impl From<$ty_name> for Ident {
            fn from(o: $ty_name) -> Ident {
                o.ident()
            }
        }

        impl Peek for $ty_name {
            fn peek(peeker: Peeker<'_>) -> Option<$ty_name> {
                let span = peeker.peek_punct_kinds(
                    &[$(PunctKind::$punct_kinds,)*],
                    &[$(PunctKind::$not_followed_by,)*],
                ).ok()?;
                Some($ty_name { span })
            }
        }

        impl Parse for $ty_name {
            fn parse(parser: &mut Parser) -> ParseResult<$ty_name> {
                match parser.take() {
                    Some(value) => Ok(value),
                    None => {
                        let kinds = vec![$(PunctKind::$punct_kinds,)*];
                        Err(parser.emit_error(ParseErrorKind::ExpectedPunct { kinds }))
                    },
                }
            }
        }
    };
);

define_token!(SemicolonToken, "a semicolon", [Semicolon], []);
define_token!(ForwardSlashToken, "a forward slash", [ForwardSlash], []);
define_token!(
    DoubleColonToken,
    "a double colon (::)",
    [Colon, Colon],
    [Colon]
);
define_token!(StarToken, "an asterisk (*)", [Star], []);
define_token!(CommaToken, "a comma", [Comma], []);
define_token!(ColonToken, "a colon", [Colon], [Colon]);
define_token!(
    RightArrowToken,
    "`->`",
    [Sub, GreaterThan],
    [GreaterThan, Equals]
);
define_token!(LessThanToken, "`<`", [LessThan], [LessThan, Equals]);
define_token!(
    GreaterThanToken,
    "`>`",
    [GreaterThan],
    [GreaterThan, Equals]
);
define_token!(OpenAngleBracketToken, "`<`", [LessThan], []);
define_token!(CloseAngleBracketToken, "`>`", [GreaterThan], []);
define_token!(TildeToken, "`~`", [Tilde], []);
define_token!(EqToken, "`=`", [Equals], [GreaterThan, Equals]);
define_token!(
    FatRightArrowToken,
    "`=>`",
    [Equals, GreaterThan],
    [GreaterThan, Equals]
);
define_token!(DotToken, "`.`", [Dot], []);
define_token!(BangToken, "`!`", [Bang], [Equals]);
define_token!(PercentToken, "`%`", [Percent], []);
define_token!(AddToken, "`+`", [Add], []);
define_token!(SubToken, "`-`", [Sub], []);
define_token!(
    ShrToken,
    "`>>`",
    [GreaterThan, GreaterThan],
    [GreaterThan, Equals]
);
define_token!(ShlToken, "`<<`", [LessThan, LessThan], [LessThan, Equals]);
define_token!(AmpersandToken, "`&`", [Ampersand], [Ampersand]);
define_token!(CaretToken, "`^`", [Caret], []);
define_token!(PipeToken, "`|`", [Pipe], [Pipe]);
define_token!(
    DoubleEqToken,
    "`==`",
    [Equals, Equals],
    [Equals, GreaterThan]
);
define_token!(BangEqToken, "`!=`", [Bang, Equals], [Equals, GreaterThan]);
define_token!(
    GreaterThanEqToken,
    "`>=`",
    [GreaterThan, Equals],
    [Equals, GreaterThan]
);
define_token!(
    LessThanEqToken,
    "`<=`",
    [LessThan, Equals],
    [Equals, GreaterThan]
);
define_token!(
    DoubleAmpersandToken,
    "`&&`",
    [Ampersand, Ampersand],
    [Ampersand]
);
define_token!(DoublePipeToken, "`||`", [Pipe, Pipe], [Pipe]);
define_token!(UnderscoreToken, "`_`", [Underscore], [Underscore]);

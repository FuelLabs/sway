use crate::priv_prelude::*;

/// The type is a keyword.
pub trait Keyword: Spanned + Sized {
    /// Creates the keyword from the given `span`.
    fn new(span: Span) -> Self;

    /// Returns an identifier for this keyword.
    fn ident(&self) -> Ident;

    /// What the string representation of the keyword is when lexing.
    const AS_STR: &'static str;
}

macro_rules! define_keyword (
    ($ty_name:ident, $keyword:literal) => {
        #[derive(Clone, Debug, Serialize)]
        pub struct $ty_name {
            span: Span,
        }

        impl Spanned for $ty_name {
            fn span(&self) -> Span {
                self.span.clone()
            }
        }

        impl Keyword for $ty_name {
            fn new(span: Span) -> Self {
                $ty_name { span }
            }

            fn ident(&self) -> Ident {
                Ident::new(self.span())
            }

            const AS_STR: &'static str = $keyword;
        }

        impl From<$ty_name> for Ident {
            fn from(o: $ty_name) -> Ident {
                o.ident()
            }
        }
    };
);

define_keyword!(ScriptToken, "script");
define_keyword!(ContractToken, "contract");
define_keyword!(PredicateToken, "predicate");
define_keyword!(LibraryToken, "library");
define_keyword!(ModToken, "mod");
define_keyword!(PubToken, "pub");
define_keyword!(UseToken, "use");
define_keyword!(AsToken, "as");
define_keyword!(StructToken, "struct");
define_keyword!(ClassToken, "class"); // Not in the language! Exists for recovery.
define_keyword!(EnumToken, "enum");
define_keyword!(SelfToken, "self");
define_keyword!(FnToken, "fn");
define_keyword!(TraitToken, "trait");
define_keyword!(ImplToken, "impl");
define_keyword!(ForToken, "for");
define_keyword!(InToken, "in");
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
define_keyword!(TrueToken, "true");
define_keyword!(FalseToken, "false");
define_keyword!(BreakToken, "break");
define_keyword!(ContinueToken, "continue");
define_keyword!(ConfigurableToken, "configurable");
define_keyword!(TypeToken, "type");
define_keyword!(PtrToken, "__ptr");
define_keyword!(SliceToken, "__slice");
define_keyword!(PanicToken, "panic");

/// The type is a token.
pub trait Token: Spanned + Sized {
    /// Creates the token from the given `span`.
    fn new(span: Span) -> Self;

    /// Returns an identifier for this token.
    fn ident(&self) -> Ident;

    /// The sequence of punctuations that make up the token.
    const PUNCT_KINDS: &'static [PunctKind];

    /// Punctuations that will not follow the token.
    const NOT_FOLLOWED_BY: &'static [PunctKind];

    /// What the string representation of the token is when lexing.
    const AS_STR: &'static str;
}

macro_rules! define_token (
    ($ty_name:ident, $description:literal, $as_str:literal, [$($punct_kinds:ident),*], [$($not_followed_by:ident),*]) => {
        #[derive(Clone, Debug, Serialize)]
        pub struct $ty_name {
            span: Span,
        }

        impl Default for $ty_name {
            fn default() -> Self {
                Self {
                    span: Span::dummy()
                }
            }
        }

        impl Spanned for $ty_name {
            fn span(&self) -> Span {
                self.span.clone()
            }
        }

        impl Token for $ty_name {
            fn new(span: Span) -> Self {
                $ty_name { span }
            }

            fn ident(&self) -> Ident {
                Ident::new(self.span())
            }

            const PUNCT_KINDS: &'static [PunctKind] = &[$(PunctKind::$punct_kinds,)*];
            const NOT_FOLLOWED_BY: &'static [PunctKind] = &[$(PunctKind::$not_followed_by,)*];
            const AS_STR: &'static str = $as_str;
        }

        impl From<$ty_name> for Ident {
            fn from(o: $ty_name) -> Ident {
                o.ident()
            }
        }
    };
);

define_token!(SemicolonToken, "a semicolon", ";", [Semicolon], []);
define_token!(
    ForwardSlashToken,
    "a forward slash",
    "/",
    [ForwardSlash],
    [Equals]
);
define_token!(
    DoubleColonToken,
    "a double colon (::)",
    "::",
    [Colon, Colon],
    [Colon]
);
define_token!(StarToken, "an asterisk (*)", "*", [Star], [Equals]);
define_token!(DoubleStarToken, "`**`", "**", [Star, Star], []);
define_token!(CommaToken, "a comma", ",", [Comma], []);
define_token!(ColonToken, "a colon", ":", [Colon], [Colon]);
define_token!(
    RightArrowToken,
    "`->`",
    "->",
    [Sub, GreaterThan],
    [GreaterThan, Equals]
);
define_token!(LessThanToken, "`<`", "<", [LessThan], [LessThan, Equals]);
define_token!(
    GreaterThanToken,
    "`>`",
    ">",
    [GreaterThan],
    [GreaterThan, Equals]
);
define_token!(OpenAngleBracketToken, "`<`", "<", [LessThan], []);
define_token!(CloseAngleBracketToken, "`>`", ">", [GreaterThan], []);
define_token!(EqToken, "`=`", "=", [Equals], [GreaterThan, Equals]);
define_token!(AddEqToken, "`+=`", "+=", [Add, Equals], []);
define_token!(SubEqToken, "`-=`", "-=", [Sub, Equals], []);
define_token!(StarEqToken, "`*=`", "*=", [Star, Equals], []);
define_token!(DivEqToken, "`/=`", "/=", [ForwardSlash, Equals], []);
define_token!(ShlEqToken, "`<<=`", "<<=", [LessThan, LessThan, Equals], []);
define_token!(
    ShrEqToken,
    "`>>=`",
    ">>=",
    [GreaterThan, GreaterThan, Equals],
    []
);
define_token!(
    FatRightArrowToken,
    "`=>`",
    "=>",
    [Equals, GreaterThan],
    [GreaterThan, Equals]
);
define_token!(DotToken, "`.`", ".", [Dot], []);
define_token!(DoubleDotToken, "`..`", "..", [Dot, Dot], [Dot]);
define_token!(BangToken, "`!`", "!", [Bang], [Equals]);
define_token!(PercentToken, "`%`", "%", [Percent], []);
define_token!(AddToken, "`+`", "+", [Add], [Equals]);
define_token!(SubToken, "`-`", "-", [Sub], [Equals]);
define_token!(
    ShrToken,
    "`>>`",
    ">>",
    [GreaterThan, GreaterThan],
    [GreaterThan, Equals]
);
define_token!(
    ShlToken,
    "`<<`",
    "<<",
    [LessThan, LessThan],
    [LessThan, Equals]
);
define_token!(AmpersandToken, "`&`", "&", [Ampersand], [Ampersand]);
define_token!(CaretToken, "`^`", "^", [Caret], []);
define_token!(PipeToken, "`|`", "|", [Pipe], [Pipe]);
define_token!(
    DoubleEqToken,
    "`==`",
    "==",
    [Equals, Equals],
    [Equals, GreaterThan]
);
define_token!(
    BangEqToken,
    "`!=`",
    "!=",
    [Bang, Equals],
    [Equals, GreaterThan]
);
define_token!(
    GreaterThanEqToken,
    "`>=`",
    ">=",
    [GreaterThan, Equals],
    [Equals, GreaterThan]
);
define_token!(
    LessThanEqToken,
    "`<=`",
    "<=",
    [LessThan, Equals],
    [Equals, GreaterThan]
);
define_token!(
    DoubleAmpersandToken,
    "`&&`",
    "&&",
    [Ampersand, Ampersand],
    [Ampersand]
);
define_token!(DoublePipeToken, "`||`", "||", [Pipe, Pipe], [Pipe]);
define_token!(UnderscoreToken, "`_`", "_", [Underscore], [Underscore]);
define_token!(HashToken, "`#`", "#", [Sharp], []);
define_token!(HashBangToken, "`#!`", "#!", [Sharp, Bang], []);

use crate::priv_prelude::*;

macro_rules! define_keyword (
    ($ty_name:ident, $err_name:ident, $fn_name:ident, $s:literal) => (
        #[derive(Clone, Debug)]
        pub struct $ty_name {
            span: Span,
        }

        #[derive(Clone)]
        pub struct $err_name {
            pub position: usize,
        }

        impl Spanned for $ty_name {
            fn span(&self) -> Span {
                self.span.clone()
            }
        }

        pub fn $fn_name<R>() -> impl Parser<Output = $ty_name, Error = $err_name, FatalError = R> + Clone {
            ident()
            .map_err(|ExpectedIdentError { position }| $err_name { position })
            .try_map(|ident: Ident| {
                if ident.as_str() == $s {
                    Ok($ty_name { span: ident.span() })
                } else {
                    Err(Ok($err_name {
                        position: ident.span().start(),
                    }))
                }
            })
        }
    )
);

macro_rules! define_token (
    ($ty_name:ident, $err_name:ident, $fn_name:ident, $s:literal) => (
        #[derive(Clone, Debug)]
        pub struct $ty_name {
            span: Span,
        }

        #[derive(Clone)]
        pub struct $err_name {
            pub position: usize,
        }

        impl Spanned for $ty_name {
            fn span(&self) -> Span {
                self.span.clone()
            }
        }

        pub fn $fn_name<R>() -> impl Parser<Output = $ty_name, Error = $err_name, FatalError = R> + Clone {
            keyword($s)
            .map_err(|ExpectedKeywordError { position, .. }| $err_name { position })
            .map_with_span(|(), span| $ty_name { span })
        }
    );
);

define_keyword!(ScriptToken, ExpectedScriptTokenError, script_token, "script");
define_keyword!(ContractToken, ExpectedContractTokenError, contract_token, "contract");
define_keyword!(PredicateToken, ExpectedPredicateTokenError, predicate_token, "predicate");
define_keyword!(LibraryToken, ExpectedLibraryTokenError, library_token, "library");
define_keyword!(DepToken, ExpectedDepTokenError, dep_token, "dep");
define_keyword!(UseToken, ExpectedUseTokenError, use_token, "use");
define_keyword!(AsToken, ExpectedAsTokenError, as_token, "as");
define_keyword!(PubToken, ExpectedPubTokenError, pub_token, "pub");
define_keyword!(StructToken, ExpectedStructTokenError, struct_token, "struct");
define_keyword!(EnumToken, ExpectedEnumTokenError, enum_token, "enum");
define_keyword!(FnToken, ExpectedFnTokenError, fn_token, "fn");
define_keyword!(TraitToken, ExpectedTraitTokenError, trait_token, "trait");
define_keyword!(AbiToken, ExpectedAbiTokenError, abi_token, "abi");
define_keyword!(LetToken, ExpectedLetTokenError, let_token, "let");
define_keyword!(AsmToken, ExpectedAsmTokenError, asm_token, "asm");
define_keyword!(ReturnToken, ExpectedReturnTokenError, return_token, "return");
define_keyword!(ForToken, ExpectedForTokenError, for_token, "for");
define_keyword!(ImplToken, ExpectedImplTokenError, impl_token, "impl");
define_keyword!(IfToken, ExpectedIfTokenError, if_token, "if");
define_keyword!(ElseToken, ExpectedElseTokenError, else_token, "else");
define_keyword!(MutToken, ExpectedMutTokenError, mut_token, "mut");
define_keyword!(StrToken, ExpectedStrTokenError, str_token, "str");
define_keyword!(ConstToken, ExpectedConstTokenError, const_token, "const");
define_keyword!(ImpureToken, ExpectedImpureTokenError, impure_token, "impure");
define_keyword!(SelfToken, ExpectedSelfTokenError, self_token, "self");
define_keyword!(MatchToken, ExpectedMatchTokenError, match_token, "match");
define_keyword!(StorageToken, ExpectedStorageTokenError, storage_token, "storage");

define_keyword!(I8Token, ExpectedI8TokenError, i8_token, "i8");
define_keyword!(I16Token, ExpectedI16TokenError, i16_token, "i16");
define_keyword!(I32Token, ExpectedI32TokenError, i32_token, "i32");
define_keyword!(I64Token, ExpectedI64TokenError, i64_token, "i64");
define_keyword!(U8Token, ExpectedU8TokenError, u8_token, "u8");
define_keyword!(U16Token, ExpectedU16TokenError, u16_token, "u16");
define_keyword!(U32Token, ExpectedU32TokenError, u32_token, "u32");
define_keyword!(U64Token, ExpectedU64TokenError, u64_token, "u64");

define_token!(SemicolonToken, ExpectedSemicolonTokenError, semicolon_token, ";");
define_token!(ColonToken, ExpectedColonTokenError, colon_token, ":");
define_token!(DoubleColonToken, ExpectedDoubleColonTokenError, double_colon_token, "::");
define_token!(ForwardSlashToken, ExpectedForwardSlashTokenError, forward_slash_token, "/");
define_token!(CommaToken, ExpectedCommaTokenError, comma_token, ",");
define_token!(StarToken, ExpectedStarTokenError, star_token, "*");
define_token!(AddToken, ExpectedAddTokenError, add_token, "+");
define_token!(SubToken, ExpectedSubTokenError, sub_token, "-");
define_token!(RightArrowToken, ExpectedRightArrowTokenError, right_arrow_token, "->");
define_token!(FatRightArrowToken, ExpectedFatRightArrowTokenError, fat_right_arrow_token, "=>");
define_token!(LessThanToken, ExpectedLessThanTokenError, less_than_token, "<");
define_token!(GreaterThanToken, ExpectedGreaterThanTokenError, greater_than_token, ">");
define_token!(EqToken, ExpectedEqTokenError, eq_token, "=");
define_token!(QuoteToken, ExpectedQuoteTokenError, quote_token, "\"");
define_token!(DotToken, ExpectedDotTokenError, dot_token, ".");
define_token!(BangToken, ExpectedBangTokenError, bang_token, "!");
define_token!(PercentToken, ExpectedPercentTokenError, percent_token, "%");
define_token!(ShlToken, ExpectedShlTokenError, shl_token, "<<");
define_token!(ShrToken, ExpectedShrTokenError, shr_token, ">>");
define_token!(AmpersandToken, ExpectedAmpersandTokenError, ampersand_token, "&");
define_token!(CaretToken, ExpectedCaretTokenError, caret_token, "^");
define_token!(PipeToken, ExpectedPipeTokenError, pipe_token, "|");
define_token!(DoubleEqToken, ExpectedDoubleEqTokenError, double_eq_token, "==");
define_token!(BangEqToken, ExpectedBangEqTokenError, bang_eq_token, "!=");
define_token!(LessThanEqToken, ExpectedLessThanEqTokenError, less_than_eq_token, "<=");
define_token!(GreaterThanEqToken, ExpectedGreaterThanEqTokenError, greater_than_eq_token, ">=");
define_token!(DoubleAmpersandToken, ExpectedDoubleAmpersandTokenError, double_ampersand_token, "&&");
define_token!(DoublePipeToken, ExpectedDoublePipeTokenError, double_pipe_token, "||");
define_token!(TildeToken, ExpectedTildeTokenError, tilde_token, "~");

define_token!(HexPrefixToken, ExpectedHexPrefixTokenError, hex_prefix_token, "0x");
define_token!(OctalPrefixToken, ExpectedOctalPrefixTokenError, octal_prefix_token, "0o");
define_token!(BinaryPrefixToken, ExpectedBinaryPrefixTokenError, binary_prefix_token, "0b");

define_token!(OpenParenToken, ExpectedOpenParenTokenError, open_paren_token, "(");
define_token!(CloseParenToken, ExpectedCloseParenTokenError, close_paren_token, ")");
define_token!(OpenSquareBracketToken, ExpectedOpenSquareBracketTokenError, open_square_bracket_token, "[");
define_token!(CloseSquareBracketToken, ExpectedCloseSquareBracketTokenError, close_square_bracket_token, "]");
define_token!(OpenBraceToken, ExpectedOpenBraceTokenError, open_brace_token, "{");
define_token!(CloseBraceToken, ExpectedCloseBraceTokenError, close_brace_token, "}");


/*
#[test]
fn parse_script_token() {
    use std::sync::Arc;

    let src = "blah blah script foo foo";
    let text: Arc<str> = Arc::from(src);
    let span = Span::new(text.clone(), "blah blah ".len(), src.len());
    let parsed = script_token().parse(&span).unwrap();
    let expected_span = Span::new(text.clone(), "blah blah ".len(), "blah blah script".len());
    assert_eq!(parsed.span, expected_span);
}
*/


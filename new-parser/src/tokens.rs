use crate::priv_prelude::*;

macro_rules! define_token (
    ($ty_name:ident, $fn_name:ident, $s:literal) => (
        #[derive(Clone, Debug)]
        pub struct $ty_name {
            span: Span,
        }

        impl Spanned for $ty_name {
            fn span(&self) -> Span {
                self.span.clone()
            }
        }

        pub fn $fn_name() -> impl Parser<Output = $ty_name> + Clone {
            keyword($s).map_with_span(|(), span| $ty_name { span })
        }
    );
);

define_token!(ScriptToken, script_token, "script");
define_token!(ContractToken, contract_token, "contract");
define_token!(PredicateToken, predicate_token, "predicate");
define_token!(LibraryToken, library_token, "library");
define_token!(DepToken, dep_token, "dep");
define_token!(UseToken, use_token, "use");
define_token!(AsToken, as_token, "as");
define_token!(PubToken, pub_token, "pub");
define_token!(StructToken, struct_token, "struct");
define_token!(EnumToken, enum_token, "enum");
define_token!(FnToken, fn_token, "fn");
define_token!(TraitToken, trait_token, "trait");
define_token!(AbiToken, abi_token, "abi");
define_token!(LetToken, let_token, "let");

define_token!(SemicolonToken, semicolon_token, ";");
define_token!(ColonToken, colon_token, ":");
define_token!(DoubleColonToken, double_colon_token, "::");
define_token!(ForwardSlashToken, forward_slash_token, "/");
define_token!(CommaToken, comma_token, ",");
define_token!(StarToken, star_token, "*");
define_token!(AddToken, add_token, "+");
define_token!(SubToken, sub_token, "-");
define_token!(RightArrowToken, right_arrow_token, "->");
define_token!(LessThanToken, less_than_token, "<");
define_token!(GreaterThanToken, greater_than_token, ">");
define_token!(EqToken, eq_token, "=");
define_token!(QuoteToken, quote_token, "\"");
define_token!(DotToken, dot_token, ".");
define_token!(BangToken, bang_token, "!");
define_token!(PercentToken, percent_token, "%");
define_token!(ShlToken, shl_token, "<<");
define_token!(ShrToken, shr_token, ">>");
define_token!(AmpersandToken, ampersand_token, "&");
define_token!(CaretToken, caret_token, "^");
define_token!(PipeToken, pipe_token, "|");
define_token!(DoubleEqToken, double_eq_token, "==");
define_token!(BangEqToken, bang_eq_token, "!=");
define_token!(LessThanEqToken, less_than_eq_token, "<=");
define_token!(GreaterThanEqToken, greater_than_eq_token, ">=");
define_token!(DoubleAmpersandToken, double_ampersand_token, "&&");
define_token!(DoublePipeToken, double_pipe_token, "||");
define_token!(TildeToken, tilde_token, "~");

define_token!(HexPrefixToken, hex_prefix_token, "0x");
define_token!(OctalPrefixToken, octal_prefix_token, "0o");
define_token!(BinaryPrefixToken, binary_prefix_token, "0b");

define_token!(I8Token, i8_token, "i8");
define_token!(I16Token, i16_token, "i16");
define_token!(I32Token, i32_token, "i32");
define_token!(I64Token, i64_token, "i64");
define_token!(U8Token, u8_token, "u8");
define_token!(U16Token, u16_token, "u16");
define_token!(U32Token, u32_token, "u32");
define_token!(U64Token, u64_token, "u64");

define_token!(OpenParenToken, open_paren_token, "(");
define_token!(CloseParenToken, close_paren_token, ")");
define_token!(OpenSquareBracketToken, open_square_bracket_token, "[");
define_token!(CloseSquareBracketToken, close_square_bracket_token, "]");
define_token!(OpenBraceToken, open_brace_token, "{");
define_token!(CloseBraceToken, close_brace_token, "}");


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


use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemBreak {
    pub break_token: BreakToken,
    pub semicolon_token: SemicolonToken,
}

impl Spanned for ItemBreak {
    fn span(&self) -> Span {
        let start = self.break_token.span();
        let end = self.semicolon_token.span();
        Span::join(start, end)
    }
}

impl Parse for ItemBreak {
    fn parse(parser: &mut Parser) -> ParseResult<ItemBreak> {
        let break_token = parser.parse()?;
        let semicolon_token = parser.parse()?;
        Ok(ItemBreak {
            break_token,
            semicolon_token,
        })
    }
}

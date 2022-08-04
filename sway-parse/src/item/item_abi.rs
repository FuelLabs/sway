use crate::{Parse, ParseBracket, ParseErrorKind, ParseResult, Parser};

use sway_ast::attribute::Annotated;
use sway_ast::keywords::Keyword;
use sway_ast::{Braces, FnSignature, ItemAbi, ItemFn};
use sway_types::Spanned;

impl Parse for ItemAbi {
    fn parse(parser: &mut Parser) -> ParseResult<ItemAbi> {
        let abi_token = parser.parse()?;
        let name = parser.parse()?;
        let abi_items: Braces<Vec<(Annotated<FnSignature>, _)>> = parser.parse()?;
        for (fn_signature, _) in abi_items.get().iter() {
            if let Some(token) = &fn_signature.value.visibility {
                return Err(parser.emit_error_with_span(
                    ParseErrorKind::UnnecessaryVisibilityQualifier {
                        visibility: token.ident(),
                    },
                    token.span(),
                ));
            }
        }
        let abi_defs_opt: Option<Braces<Vec<Annotated<ItemFn>>>> = Braces::try_parse(parser)?;
        if let Some(abi_defs) = &abi_defs_opt {
            for item_fn in abi_defs.get().iter() {
                if let Some(token) = &item_fn.value.fn_signature.visibility {
                    return Err(parser.emit_error_with_span(
                        ParseErrorKind::UnnecessaryVisibilityQualifier {
                            visibility: token.ident(),
                        },
                        token.span(),
                    ));
                }
            }
        }
        Ok(ItemAbi {
            abi_token,
            name,
            abi_items,
            abi_defs_opt,
        })
    }
}

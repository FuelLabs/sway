use crate::{Parse, ParseResult, Parser};

use sway_ast::attribute::Annotated;
use sway_ast::keywords::{
    ConstToken, FnToken, OpenAngleBracketToken, SemicolonToken, TypeToken, WhereToken,
};
use sway_ast::{Braces, ItemImpl, ItemImplItem, PubToken, Ty};
use sway_error::parser_error::ParseErrorKind;

impl Parse for ItemImplItem {
    fn parse(parser: &mut Parser) -> ParseResult<ItemImplItem> {
        if parser.peek::<PubToken>().is_some() || parser.peek::<FnToken>().is_some() {
            let fn_decl = parser.parse()?;
            Ok(ItemImplItem::Fn(fn_decl))
        } else if let Some(_const_keyword) = parser.peek::<ConstToken>() {
            let const_decl = parser.parse()?;
            parser.parse::<SemicolonToken>()?;
            Ok(ItemImplItem::Const(const_decl))
        } else if let Some(_type_keyword) = parser.peek::<TypeToken>() {
            let type_decl = parser.parse()?;
            parser.parse::<SemicolonToken>()?;
            Ok(ItemImplItem::Type(type_decl))
        } else {
            Err(parser.emit_error(ParseErrorKind::ExpectedAnItem))
        }
    }
}

impl Parse for ItemImpl {
    fn parse(parser: &mut Parser) -> ParseResult<ItemImpl> {
        let impl_token = parser.parse()?;
        let generic_params_opt = parser.guarded_parse::<OpenAngleBracketToken, _>()?;
        let ty = parser.parse()?;
        let (trait_opt, ty) = match parser.take() {
            Some(for_token) => match ty {
                Ty::Path(path_type) => (Some((path_type, for_token)), parser.parse()?),
                _ => {
                    return Err(parser.emit_error(ParseErrorKind::ExpectedPathType));
                }
            },
            None => (None, ty),
        };
        let where_clause_opt = parser.guarded_parse::<WhereToken, _>()?;
        let contents: Braces<Vec<Annotated<ItemImplItem>>> = parser.parse()?;
        if trait_opt.is_some() {
            for annotated in contents.get().iter() {
                if let ItemImplItem::Fn(item_fn) = &annotated.value {
                    parser.ban_visibility_qualifier(&item_fn.fn_signature.visibility)?;
                }
            }
        }
        Ok(ItemImpl {
            impl_token,
            generic_params_opt,
            trait_opt,
            ty,
            where_clause_opt,
            contents,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::parse;
    use assert_matches::*;

    #[test]
    fn parse_impl_ptr() {
        let item = parse::<ItemImpl>(
            r#"
            impl __ptr[T] {}
            "#,
        );
        assert_matches!(item.ty, Ty::Ptr { .. });
    }

    #[test]
    fn parse_impl_for_ptr() {
        let item = parse::<ItemImpl>(
            r#"
            impl Foo for __ptr[T] {}
            "#,
        );
        assert_matches!(item.ty, Ty::Ptr { .. });
    }

    #[test]
    fn parse_impl_slice() {
        let item = parse::<ItemImpl>(
            r#"
            impl __slice[T] {}
            "#,
        );
        assert_matches!(item.ty, Ty::Slice { .. });
    }

    #[test]
    fn parse_impl_for_slice() {
        let item = parse::<ItemImpl>(
            r#"
            impl Foo for __slice[T] {}
            "#,
        );
        assert_matches!(item.ty, Ty::Slice { .. });
    }
}

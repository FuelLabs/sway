use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemImpl {
    pub impl_token: ImplToken,
    pub trait_opt: Option<(PathType, ForToken)>,
    pub ty: Ty,
    pub contents: Braces<Vec<ItemFn>>,
}

impl Parse for ItemImpl {
    fn parse(parser: &mut Parser) -> ParseResult<ItemImpl> {
        let impl_token = parser.parse()?;
        let path_type = parser.parse()?;
        let (trait_opt, ty) = match parser.take() {
            Some(for_token) => {
                let ty = parser.parse()?;
                (Some((path_type, for_token)), ty)
            },
            None => (None, Ty::Path(path_type)),
        };
        let contents = parser.parse()?;
        Ok(ItemImpl { impl_token, trait_opt, ty, contents })
    }
}

use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemImpl {
    pub impl_token: ImplToken,
    pub generic_params_opt: Option<GenericParams>,
    pub trait_opt: Option<(PathType, ForToken)>,
    pub ty: Ty,
    pub contents: Braces<Vec<ItemFn>>,
}

impl ItemImpl {
    pub fn span(&self) -> Span {
        Span::join(self.impl_token.span(), self.contents.span())
    }
}

impl Parse for ItemImpl {
    fn parse(parser: &mut Parser) -> ParseResult<ItemImpl> {
        let impl_token = parser.parse()?;
        let generic_params_opt = match parser.peek::<OpenAngleBracketToken>() {
            Some(_open_angle_bracket_token) => {
                let generic_params = parser.parse()?;
                Some(generic_params)
            }
            None => None,
        };
        let path_type = parser.parse()?;
        let (trait_opt, ty) = match parser.take() {
            Some(for_token) => {
                let ty = parser.parse()?;
                (Some((path_type, for_token)), ty)
            }
            None => (None, Ty::Path(path_type)),
        };
        let contents = parser.parse()?;
        Ok(ItemImpl {
            impl_token,
            generic_params_opt,
            trait_opt,
            ty,
            contents,
        })
    }
}

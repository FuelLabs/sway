use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemImpl {
    pub impl_token: ImplToken,
    pub generic_params_opt: Option<GenericParams>,
    pub trait_opt: Option<(PathType, ForToken)>,
    pub ty: Ty,
    pub where_clause_opt: Option<WhereClause>,
    pub contents: Braces<Vec<Annotated<ItemFn>>>,
}

impl Spanned for ItemImpl {
    fn span(&self) -> Span {
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
        let where_clause_opt = match parser.peek::<WhereToken>() {
            Some(..) => {
                let where_clause = parser.parse()?;
                Some(where_clause)
            }
            None => None,
        };
        let contents: Braces<Vec<Annotated<ItemFn>>> = parser.parse()?;
        if trait_opt.is_some() {
            for item_fn in contents.get().iter() {
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

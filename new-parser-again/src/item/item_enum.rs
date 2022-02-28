use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemEnum {
    pub visibility: Option<PubToken>,
    pub enum_token: EnumToken,
    pub name: Ident,
    pub generics: Option<GenericParams>,
    pub fields: Braces<Punctuated<TypeField, CommaToken>>,
}

impl Parse for ItemEnum {
    fn parse(parser: &mut Parser) -> ParseResult<ItemEnum> {
        let visibility = parser.take();
        let enum_token = parser.parse()?;
        let name = parser.parse()?;
        let generics = if parser.peek::<LessThanToken>().is_some() {
            Some(parser.parse()?)
        } else {
            None
        };
        let fields = parser.parse()?;
        Ok(ItemEnum { visibility, enum_token, name, generics, fields })
    }
}


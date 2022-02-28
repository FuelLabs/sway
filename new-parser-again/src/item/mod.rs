use crate::priv_prelude::*;

pub mod item_use;
pub mod item_struct;
pub mod item_enum;
pub mod item_fn;
pub mod item_trait;
pub mod item_impl;
pub mod item_abi;
pub mod item_const;
pub mod item_storage;

#[derive(Clone, Debug)]
pub enum Item {
    Use(ItemUse),
    Struct(ItemStruct),
    Enum(ItemEnum),
    Fn(ItemFn),
    Trait(ItemTrait),
    Impl(ItemImpl),
    Abi(ItemAbi),
    Const(ItemConst),
    Storage(ItemStorage),
}

impl Parse for Item {
    fn parse(parser: &mut Parser) -> ParseResult<Item> {
        if parser.peek::<UseToken>().is_some() || parser.peek2::<PubToken, UseToken>().is_some() {
            let item_use = parser.parse()?;
            return Ok(Item::Use(item_use));
        }
        if parser.peek::<StructToken>().is_some() || parser.peek2::<PubToken, StructToken>().is_some() {
            let item_struct = parser.parse()?;
            return Ok(Item::Struct(item_struct));
        }
        if parser.peek::<EnumToken>().is_some() || parser.peek2::<PubToken, EnumToken>().is_some() {
            let item_enum = parser.parse()?;
            return Ok(Item::Enum(item_enum));
        }
        if {
            parser.peek::<FnToken>().is_some() ||
            parser.peek2::<PubToken, FnToken>().is_some() ||
            parser.peek2::<ImpureToken, FnToken>().is_some() ||
            parser.peek3::<PubToken, ImpureToken, FnToken>().is_some()
        } {
            let item_fn = parser.parse()?;
            return Ok(Item::Fn(item_fn));
        }
        if parser.peek::<TraitToken>().is_some() || parser.peek2::<PubToken, TraitToken>().is_some() {
            let item_trait = parser.parse()?;
            return Ok(Item::Trait(item_trait));
        }
        if parser.peek::<ImplToken>().is_some() {
            let item_impl = parser.parse()?;
            return Ok(Item::Impl(item_impl));
        }
        if parser.peek::<AbiToken>().is_some() {
            let item_abi = parser.parse()?;
            return Ok(Item::Abi(item_abi));
        }
        if parser.peek::<ConstToken>().is_some() || parser.peek2::<PubToken, ConstToken>().is_some() {
            let item_const = parser.parse()?;
            return Ok(Item::Const(item_const));
        }
        if parser.peek::<StorageToken>().is_some() {
            let item_storage = parser.parse()?;
            return Ok(Item::Storage(item_storage));
        }
        Err(parser.emit_error("expected an item"))
    }
}

#[derive(Clone, Debug)]
pub struct TypeField {
    pub name: Ident,
    pub colon_token: ColonToken,
    pub ty: Ty,
}

impl Parse for TypeField {
    fn parse(parser: &mut Parser) -> ParseResult<TypeField> {
        let name = parser.parse()?;
        let colon_token = parser.parse()?;
        let ty = parser.parse()?;
        Ok(TypeField { name, colon_token, ty })
    }
}

#[derive(Clone, Debug)]
pub enum FnArgs {
    Static(Punctuated<TypeField, CommaToken>),
    NonStatic {
        self_token: SelfToken,
        args_opt: Option<(CommaToken, Punctuated<TypeField, CommaToken>)>,
    },
}

impl ParseToEnd for FnArgs {
    fn parse_to_end<'a>(mut parser: Parser<'a>) -> ParseResult<(FnArgs, ParserConsumed<'a>)> {
        match parser.take() {
            Some(self_token) => {
                match parser.take() {
                    Some(comma_token) => {
                        let (args, consumed) = parser.parse_to_end()?;
                        let fn_args = FnArgs::NonStatic {
                            self_token,
                            args_opt: Some((comma_token, args)),
                        };
                        Ok((fn_args, consumed))
                    },
                    None => {
                        let fn_args = FnArgs::NonStatic {
                            self_token,
                            args_opt: None,
                        };
                        match parser.check_empty() {
                            Some(consumed) => {
                                Ok((fn_args, consumed))
                            },
                            None => {
                                Err(parser.emit_error("expected a comma or closing parens"))
                            },
                        }
                    },
                }
            },
            None => {
                let (args, consumed) = parser.parse_to_end()?;
                let fn_args = FnArgs::Static(args);
                Ok((fn_args, consumed))
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct FnSignature {
    pub visibility: Option<PubToken>,
    pub impure: Option<ImpureToken>,
    pub fn_token: FnToken,
    pub name: Ident,
    pub generics: Option<GenericParams>,
    pub arguments: Parens<FnArgs>,
    pub return_type_opt: Option<(RightArrowToken, Ty)>,
}

impl Parse for FnSignature {
    fn parse(parser: &mut Parser) -> ParseResult<FnSignature> {
        let visibility = parser.take();
        let impure = parser.take();
        let fn_token = parser.parse()?;
        let name = parser.parse()?;
        let generics = if parser.peek::<LessThanToken>().is_some() {
            Some(parser.parse()?)
        } else {
            None
        };
        let arguments = parser.parse()?;
        let return_type_opt = match parser.take() {
            Some(right_arrow_token) => {
                let ty = parser.parse()?;
                Some((right_arrow_token, ty))
            },
            None => None,
        };
        Ok(FnSignature { visibility, impure, fn_token, name, generics, arguments, return_type_opt })
    }
}


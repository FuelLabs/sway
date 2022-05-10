use crate::priv_prelude::*;

pub mod item_abi;
pub mod item_const;
pub mod item_enum;
pub mod item_fn;
pub mod item_impl;
pub mod item_storage;
pub mod item_struct;
pub mod item_trait;
pub mod item_use;

pub struct Item {
    pub attribute_list: Vec<AttributeDecl>,
    pub kind: ItemKind,
}

impl Item {
    pub fn span(&self) -> Span {
        match self.attribute_list.first() {
            Some(attr0) => Span::join(attr0.span(), self.kind.span()),
            None => self.kind.span(),
        }
    }
}

impl Parse for Item {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        let mut attribute_list = Vec::new();
        loop {
            if parser.peek::<HashToken>().is_some() {
                attribute_list.push(parser.parse()?);
            } else {
                break;
            }
        }
        let kind = parser.parse()?;
        Ok(Item {
            attribute_list,
            kind,
        })
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum ItemKind {
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

impl ItemKind {
    pub fn span(&self) -> Span {
        match self {
            ItemKind::Use(item_use) => item_use.span(),
            ItemKind::Struct(item_struct) => item_struct.span(),
            ItemKind::Enum(item_enum) => item_enum.span(),
            ItemKind::Fn(item_fn) => item_fn.span(),
            ItemKind::Trait(item_trait) => item_trait.span(),
            ItemKind::Impl(item_impl) => item_impl.span(),
            ItemKind::Abi(item_abi) => item_abi.span(),
            ItemKind::Const(item_const) => item_const.span(),
            ItemKind::Storage(item_storage) => item_storage.span(),
        }
    }
}

impl Parse for ItemKind {
    fn parse(parser: &mut Parser) -> ParseResult<ItemKind> {
        if parser.peek::<UseToken>().is_some() || parser.peek2::<PubToken, UseToken>().is_some() {
            let item_use = parser.parse()?;
            return Ok(ItemKind::Use(item_use));
        }
        if parser.peek::<StructToken>().is_some()
            || parser.peek2::<PubToken, StructToken>().is_some()
        {
            let item_struct = parser.parse()?;
            return Ok(ItemKind::Struct(item_struct));
        }
        if parser.peek::<EnumToken>().is_some() || parser.peek2::<PubToken, EnumToken>().is_some() {
            let item_enum = parser.parse()?;
            return Ok(ItemKind::Enum(item_enum));
        }
        if parser.peek::<FnToken>().is_some()
            || parser.peek2::<PubToken, FnToken>().is_some()
            || parser.peek2::<ImpureToken, FnToken>().is_some()
            || parser.peek3::<PubToken, ImpureToken, FnToken>().is_some()
        {
            let item_fn = parser.parse()?;
            return Ok(ItemKind::Fn(item_fn));
        }
        if parser.peek::<TraitToken>().is_some() || parser.peek2::<PubToken, TraitToken>().is_some()
        {
            let item_trait = parser.parse()?;
            return Ok(ItemKind::Trait(item_trait));
        }
        if parser.peek::<ImplToken>().is_some() {
            let item_impl = parser.parse()?;
            return Ok(ItemKind::Impl(item_impl));
        }
        if parser.peek::<AbiToken>().is_some() {
            let item_abi = parser.parse()?;
            return Ok(ItemKind::Abi(item_abi));
        }
        if parser.peek::<ConstToken>().is_some() || parser.peek2::<PubToken, ConstToken>().is_some()
        {
            let item_const = parser.parse()?;
            return Ok(ItemKind::Const(item_const));
        }
        if parser.peek::<StorageToken>().is_some() {
            let item_storage = parser.parse()?;
            return Ok(ItemKind::Storage(item_storage));
        }
        Err(parser.emit_error(ParseErrorKind::ExpectedAnItem))
    }
}

#[derive(Clone, Debug)]
pub struct TypeField {
    pub name: Ident,
    pub colon_token: ColonToken,
    pub ty: Ty,
}

impl TypeField {
    pub fn span(&self) -> Span {
        Span::join(self.name.span().clone(), self.ty.span())
    }
}

impl Parse for TypeField {
    fn parse(parser: &mut Parser) -> ParseResult<TypeField> {
        let name = parser.parse()?;
        let colon_token = parser.parse()?;
        let ty = parser.parse()?;
        Ok(TypeField {
            name,
            colon_token,
            ty,
        })
    }
}

#[derive(Clone, Debug)]
pub enum FnArgs {
    Static(Punctuated<FnArg, CommaToken>),
    NonStatic {
        self_token: SelfToken,
        args_opt: Option<(CommaToken, Punctuated<FnArg, CommaToken>)>,
    },
}

#[derive(Clone, Debug)]
pub struct FnArg {
    pub pattern: Pattern,
    pub colon_token: ColonToken,
    pub ty: Ty,
}

impl FnArg {
    pub fn span(&self) -> Span {
        Span::join(self.pattern.span(), self.ty.span())
    }
}

impl ParseToEnd for FnArgs {
    fn parse_to_end<'a, 'e>(
        mut parser: Parser<'a, 'e>,
    ) -> ParseResult<(FnArgs, ParserConsumed<'a>)> {
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
                    }
                    None => {
                        let fn_args = FnArgs::NonStatic {
                            self_token,
                            args_opt: None,
                        };
                        match parser.check_empty() {
                            Some(consumed) => Ok((fn_args, consumed)),
                            None => Err(parser
                                .emit_error(ParseErrorKind::ExpectedCommaOrCloseParenInFnArgs)),
                        }
                    }
                }
            }
            None => {
                let (args, consumed) = parser.parse_to_end()?;
                let fn_args = FnArgs::Static(args);
                Ok((fn_args, consumed))
            }
        }
    }
}

impl Parse for FnArg {
    fn parse(parser: &mut Parser) -> ParseResult<FnArg> {
        let pattern = parser.parse()?;
        let colon_token = parser.parse()?;
        let ty = parser.parse()?;
        Ok(FnArg {
            pattern,
            colon_token,
            ty,
        })
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
    pub where_clause_opt: Option<WhereClause>,
}

impl FnSignature {
    pub fn span(&self) -> Span {
        let start = match &self.visibility {
            Some(pub_token) => pub_token.span(),
            None => match &self.impure {
                Some(impure_token) => impure_token.span(),
                None => self.fn_token.span(),
            },
        };
        let end = match &self.where_clause_opt {
            Some(where_clause) => where_clause.span(),
            None => match &self.return_type_opt {
                Some((_right_arrow, ty)) => ty.span(),
                None => self.arguments.span(),
            },
        };
        Span::join(start, end)
    }
}

impl Parse for FnSignature {
    fn parse(parser: &mut Parser) -> ParseResult<FnSignature> {
        let visibility = parser.take();
        let impure = parser.take();
        let fn_token = parser.parse()?;
        let name = parser.parse()?;
        let generics = if parser.peek::<OpenAngleBracketToken>().is_some() {
            Some(parser.parse()?)
        } else {
            None
        };
        let arguments = parser.parse()?;
        let return_type_opt = match parser.take() {
            Some(right_arrow_token) => {
                let ty = parser.parse()?;
                Some((right_arrow_token, ty))
            }
            None => None,
        };
        let where_clause_opt = match parser.peek::<WhereToken>() {
            Some(_where_token) => {
                let where_clause = parser.parse()?;
                Some(where_clause)
            }
            None => None,
        };
        Ok(FnSignature {
            visibility,
            impure,
            fn_token,
            name,
            generics,
            arguments,
            return_type_opt,
            where_clause_opt,
        })
    }
}

// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_item(input: &str) -> Item {
        let token_stream = crate::token::lex(&Arc::from(input), 0, input.len(), None).unwrap();
        let mut errors = Vec::new();
        let mut parser = Parser::new(&token_stream, &mut errors);
        match Item::parse(&mut parser) {
            Ok(item) => item,
            Err(_) => {
                //println!("Tokens: {:?}", token_stream);
                panic!("Parse error: {:?}", errors);
            }
        }
    }

    #[test]
    fn parse_attributes_none() {
        let item = parse_item(
            r#"
            fn f() -> bool {
                false
            }
            "#,
        );

        assert!(matches!(item.kind, ItemKind::Fn(_)));
        assert!(item.attribute_list.is_empty());
    }

    #[test]
    fn parse_attributes_fn_basic() {
        let item = parse_item(
            r#"
            #[foo]
            fn f() -> bool {
                false
            }
            "#,
        );

        assert!(matches!(item.kind, ItemKind::Fn(_)));

        assert_eq!(item.attribute_list.len(), 1);

        let attrib = item.attribute_list.get(0).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "foo");
        assert!(attrib.attribute.get().args.is_none());
    }

    #[test]
    fn parse_attributes_fn_two_basic() {
        let item = parse_item(
            r#"
            #[foo]
            #[bar]
            fn f() -> bool {
                false
            }
            "#,
        );

        assert!(matches!(item.kind, ItemKind::Fn(_)));

        assert_eq!(item.attribute_list.len(), 2);

        let attrib = item.attribute_list.get(0).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "foo");
        assert!(attrib.attribute.get().args.is_none());

        let attrib = item.attribute_list.get(1).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "bar");
        assert!(attrib.attribute.get().args.is_none());
    }

    #[test]
    fn parse_attributes_fn_one_arg() {
        let item = parse_item(
            r#"
            #[foo(one)]
            fn f() -> bool {
                false
            }
            "#,
        );

        assert!(matches!(item.kind, ItemKind::Fn(_)));

        assert_eq!(item.attribute_list.len(), 1);

        let attrib = item.attribute_list.get(0).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "foo");
        assert!(attrib.attribute.get().args.is_some());

        let mut args = attrib
            .attribute
            .get()
            .args
            .as_ref()
            .unwrap()
            .get()
            .into_iter();
        assert_eq!(args.next().map(|arg| arg.as_str()), Some("one"));
        assert_eq!(args.next().map(|arg| arg.as_str()), None);
    }

    #[test]
    fn parse_attributes_fn_empty_parens() {
        let item = parse_item(
            r#"
            #[foo()]
            fn f() -> bool {
                false
            }
            "#,
        );

        assert!(matches!(item.kind, ItemKind::Fn(_)));

        assert_eq!(item.attribute_list.len(), 1);

        let attrib = item.attribute_list.get(0).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "foo");

        // Args are still parsed as 'some' but with an empty collection.
        assert!(attrib.attribute.get().args.is_some());

        let mut args = attrib
            .attribute
            .get()
            .args
            .as_ref()
            .unwrap()
            .get()
            .into_iter();
        assert_eq!(args.next().map(|arg| arg.as_str()), None);
    }

    #[test]
    fn parse_attributes_fn_zero_and_one_arg() {
        let item = parse_item(
            r#"
            #[bar]
            #[foo(one)]
            fn f() -> bool {
                false
            }
            "#,
        );

        assert!(matches!(item.kind, ItemKind::Fn(_)));

        assert_eq!(item.attribute_list.len(), 2);

        let attrib = item.attribute_list.get(0).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "bar");
        assert!(attrib.attribute.get().args.is_none());

        let attrib = item.attribute_list.get(1).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "foo");
        assert!(attrib.attribute.get().args.is_some());

        let mut args = attrib
            .attribute
            .get()
            .args
            .as_ref()
            .unwrap()
            .get()
            .into_iter();
        assert_eq!(args.next().map(|arg| arg.as_str()), Some("one"));
        assert_eq!(args.next().map(|arg| arg.as_str()), None);
    }

    #[test]
    fn parse_attributes_fn_one_and_zero_arg() {
        let item = parse_item(
            r#"
            #[foo(one)]
            #[bar]
            fn f() -> bool {
                false
            }
            "#,
        );

        assert!(matches!(item.kind, ItemKind::Fn(_)));

        assert_eq!(item.attribute_list.len(), 2);

        let attrib = item.attribute_list.get(0).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "foo");
        assert!(attrib.attribute.get().args.is_some());

        let mut args = attrib
            .attribute
            .get()
            .args
            .as_ref()
            .unwrap()
            .get()
            .into_iter();
        assert_eq!(args.next().map(|arg| arg.as_str()), Some("one"));
        assert_eq!(args.next().map(|arg| arg.as_str()), None);

        let attrib = item.attribute_list.get(1).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "bar");
        assert!(attrib.attribute.get().args.is_none());
    }

    #[test]
    fn parse_attributes_fn_two_args() {
        let item = parse_item(
            r#"
            #[foo(one, two)]
            fn f() -> bool {
                false
            }
            "#,
        );

        assert!(matches!(item.kind, ItemKind::Fn(_)));

        assert_eq!(item.attribute_list.len(), 1);

        let attrib = item.attribute_list.get(0).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "foo");
        assert!(attrib.attribute.get().args.is_some());

        let mut args = attrib
            .attribute
            .get()
            .args
            .as_ref()
            .unwrap()
            .get()
            .into_iter();
        assert_eq!(args.next().map(|arg| arg.as_str()), Some("one"));
        assert_eq!(args.next().map(|arg| arg.as_str()), Some("two"));
        assert_eq!(args.next().map(|arg| arg.as_str()), None);
    }

    #[test]
    fn parse_attributes_fn_zero_one_and_three_args() {
        let item = parse_item(
            r#"
            #[bar]
            #[foo(one)]
            #[baz(two,three,four)]
            fn f() -> bool {
                false
            }
            "#,
        );

        assert!(matches!(item.kind, ItemKind::Fn(_)));

        assert_eq!(item.attribute_list.len(), 3);

        let attrib = item.attribute_list.get(0).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "bar");
        assert!(attrib.attribute.get().args.is_none());

        let attrib = item.attribute_list.get(1).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "foo");
        assert!(attrib.attribute.get().args.is_some());

        let mut args = attrib
            .attribute
            .get()
            .args
            .as_ref()
            .unwrap()
            .get()
            .into_iter();
        assert_eq!(args.next().map(|arg| arg.as_str()), Some("one"));
        assert_eq!(args.next().map(|arg| arg.as_str()), None);

        let attrib = item.attribute_list.get(2).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "baz");
        assert!(attrib.attribute.get().args.is_some());

        let mut args = attrib
            .attribute
            .get()
            .args
            .as_ref()
            .unwrap()
            .get()
            .into_iter();
        assert_eq!(args.next().map(|arg| arg.as_str()), Some("two"));
        assert_eq!(args.next().map(|arg| arg.as_str()), Some("three"));
        assert_eq!(args.next().map(|arg| arg.as_str()), Some("four"));
        assert_eq!(args.next().map(|arg| arg.as_str()), None);
    }
}

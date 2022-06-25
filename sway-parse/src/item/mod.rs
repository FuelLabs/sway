use crate::priv_prelude::*;

pub mod item_abi;
pub mod item_const;
pub mod item_control_flow;
pub mod item_enum;
pub mod item_fn;
pub mod item_impl;
pub mod item_storage;
pub mod item_struct;
pub mod item_trait;
pub mod item_use;

pub type Item = Annotated<ItemKind>;

impl Spanned for Item {
    fn span(&self) -> Span {
        match self.attribute_list.first() {
            Some(attr0) => Span::join(attr0.span(), self.value.span()),
            None => self.value.span(),
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum ItemKind {
    Use(ItemUse),
    Break(ItemBreak),
    Struct(ItemStruct),
    Enum(ItemEnum),
    Fn(ItemFn),
    Trait(ItemTrait),
    Impl(ItemImpl),
    Abi(ItemAbi),
    Const(ItemConst),
    Storage(ItemStorage),
}

impl Spanned for ItemKind {
    fn span(&self) -> Span {
        match self {
            ItemKind::Use(item_use) => item_use.span(),
            ItemKind::Break(item_break) => item_break.span(),
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
        if parser.peek::<BreakToken>().is_some() {
            let item_break = parser.parse()?;
            return Ok(ItemKind::Break(item_break));
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
        if parser.peek::<FnToken>().is_some() || parser.peek2::<PubToken, FnToken>().is_some() {
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

impl Spanned for TypeField {
    fn span(&self) -> Span {
        Span::join(self.name.span(), self.ty.span())
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
        mutable_self: Option<MutToken>,
        args_opt: Option<(CommaToken, Punctuated<FnArg, CommaToken>)>,
    },
}

#[derive(Clone, Debug)]
pub struct FnArg {
    pub pattern: Pattern,
    pub colon_token: ColonToken,
    pub ty: Ty,
}

impl Spanned for FnArg {
    fn span(&self) -> Span {
        Span::join(self.pattern.span(), self.ty.span())
    }
}

impl ParseToEnd for FnArgs {
    fn parse_to_end<'a, 'e>(
        mut parser: Parser<'a, 'e>,
    ) -> ParseResult<(FnArgs, ParserConsumed<'a>)> {
        let mutable_self = match parser.peek::<MutToken>() {
            Some(_mut_token) => {
                let mut_token = parser.parse()?;
                Some(mut_token)
            }
            None => None,
        };
        match parser.take() {
            Some(self_token) => {
                match parser.take() {
                    Some(comma_token) => {
                        let (args, consumed) = parser.parse_to_end()?;
                        let fn_args = FnArgs::NonStatic {
                            self_token,
                            mutable_self,
                            args_opt: Some((comma_token, args)),
                        };
                        Ok((fn_args, consumed))
                    }
                    None => {
                        let fn_args = FnArgs::NonStatic {
                            self_token,
                            mutable_self,
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
    pub fn_token: FnToken,
    pub name: Ident,
    pub generics: Option<GenericParams>,
    pub arguments: Parens<FnArgs>,
    pub return_type_opt: Option<(RightArrowToken, Ty)>,
    pub where_clause_opt: Option<WhereClause>,
}

impl Spanned for FnSignature {
    fn span(&self) -> Span {
        let start = match &self.visibility {
            Some(pub_token) => pub_token.span(),
            None => self.fn_token.span(),
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
        let fn_token = parser.parse()?;
        let name: Ident = parser.parse()?;
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

        assert!(matches!(item.value, ItemKind::Fn(_)));
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

        assert!(matches!(item.value, ItemKind::Fn(_)));

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

        assert!(matches!(item.value, ItemKind::Fn(_)));

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

        assert!(matches!(item.value, ItemKind::Fn(_)));

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

        assert!(matches!(item.value, ItemKind::Fn(_)));

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

        assert!(matches!(item.value, ItemKind::Fn(_)));

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

        assert!(matches!(item.value, ItemKind::Fn(_)));

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

        assert!(matches!(item.value, ItemKind::Fn(_)));

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

        assert!(matches!(item.value, ItemKind::Fn(_)));

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

    #[test]
    fn parse_attributes_trait() {
        let item = parse_item(
            r#"
            trait T {
                #[foo(one)]
                #[bar]
                fn f() -> bool;
            } {
                #[bar(one, two, three)]
                fn g() -> bool {
                    f()
                }
            }
            "#,
        );

        // The trait itself has no attributes.
        assert!(matches!(item.value, ItemKind::Trait(_)));
        assert_eq!(item.attribute_list.len(), 0);

        if let ItemKind::Trait(item_trait) = item.value {
            let mut decls = item_trait.trait_items.get().iter();

            let f_sig = decls.next();
            assert!(f_sig.is_some());

            let attrib = f_sig.unwrap().0.attribute_list.get(0).unwrap();
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

            let attrib = f_sig.unwrap().0.attribute_list.get(1).unwrap();
            assert_eq!(attrib.attribute.get().name.as_str(), "bar");
            assert!(attrib.attribute.get().args.is_none());

            assert!(decls.next().is_none());

            assert!(item_trait.trait_defs_opt.is_some());
            let mut defs = item_trait.trait_defs_opt.as_ref().unwrap().get().iter();

            let g_sig = defs.next();
            assert!(g_sig.is_some());

            let attrib = g_sig.unwrap().attribute_list.get(0).unwrap();
            assert_eq!(attrib.attribute.get().name.as_str(), "bar");
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
            assert_eq!(args.next().map(|arg| arg.as_str()), Some("three"));
            assert_eq!(args.next().map(|arg| arg.as_str()), None);

            assert!(defs.next().is_none());
        } else {
            panic!("Parsed trait is not a trait.");
        }
    }

    #[test]
    fn parse_attributes_abi() {
        let item = parse_item(
            r#"
            abi A {
                #[bar(one, two, three)]
                fn f() -> bool;

                #[foo]
                fn g() -> u64;
            } {
                #[baz(one)]
                fn h() -> bool {
                    f()
                }
            }
            "#,
        );

        // The ABI itself has no attributes.
        assert!(matches!(item.value, ItemKind::Abi(_)));
        assert_eq!(item.attribute_list.len(), 0);

        if let ItemKind::Abi(item_abi) = item.value {
            let mut decls = item_abi.abi_items.get().iter();

            let f_sig = decls.next();
            assert!(f_sig.is_some());

            let attrib = f_sig.unwrap().0.attribute_list.get(0).unwrap();
            assert_eq!(attrib.attribute.get().name.as_str(), "bar");
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
            assert_eq!(args.next().map(|arg| arg.as_str()), Some("three"));
            assert_eq!(args.next().map(|arg| arg.as_str()), None);

            assert!(f_sig.unwrap().0.attribute_list.get(1).is_none());

            let g_sig = decls.next();
            assert!(g_sig.is_some());

            let attrib = g_sig.unwrap().0.attribute_list.get(0).unwrap();
            assert_eq!(attrib.attribute.get().name.as_str(), "foo");
            assert!(attrib.attribute.get().args.is_none());

            assert!(g_sig.unwrap().0.attribute_list.get(1).is_none());

            assert!(decls.next().is_none());

            assert!(item_abi.abi_defs_opt.is_some());
            let mut defs = item_abi.abi_defs_opt.as_ref().unwrap().get().iter();

            let h_sig = defs.next();
            assert!(h_sig.is_some());

            let attrib = h_sig.unwrap().attribute_list.get(0).unwrap();
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
            assert_eq!(args.next().map(|arg| arg.as_str()), Some("one"));
            assert_eq!(args.next().map(|arg| arg.as_str()), None);

            assert!(defs.next().is_none());
        } else {
            panic!("Parsed ABI is not an ABI.");
        }
    }
}

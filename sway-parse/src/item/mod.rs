use crate::{Parse, ParseErrorKind, ParseResult, ParseToEnd, Parser, ParserConsumed};

use sway_ast::keywords::{
    AbiToken, BreakToken, ConstToken, ContinueToken, EnumToken, FnToken, ImplToken, MutToken,
    OpenAngleBracketToken, PubToken, StorageToken, StructToken, TraitToken, UseToken, WhereToken,
};
use sway_ast::token::{DocComment, DocStyle};
use sway_ast::{FnArg, FnArgs, FnSignature, ItemKind, TypeField};

mod item_abi;
mod item_const;
mod item_control_flow;
mod item_enum;
mod item_fn;
mod item_impl;
mod item_storage;
mod item_struct;
mod item_trait;
mod item_use;

impl Parse for ItemKind {
    fn parse(parser: &mut Parser) -> ParseResult<ItemKind> {
        // FIXME(Centril): Visibility should be moved out of `ItemKind` variants,
        // introducing a struct `Item` that holds the visibility and the kind,
        // and then validate in an "AST validation" step which kinds that should have `pub`s.

        if parser.peek::<UseToken>().is_some() || parser.peek::<(PubToken, UseToken)>().is_some() {
            let item_use = parser.parse()?;
            return Ok(ItemKind::Use(item_use));
        }
        if parser.peek::<StructToken>().is_some()
            || parser.peek::<(PubToken, StructToken)>().is_some()
        {
            let item_struct = parser.parse()?;
            return Ok(ItemKind::Struct(item_struct));
        }
        if parser.peek::<EnumToken>().is_some() || parser.peek::<(PubToken, EnumToken)>().is_some()
        {
            let item_enum = parser.parse()?;
            return Ok(ItemKind::Enum(item_enum));
        }
        if parser.peek::<FnToken>().is_some() || parser.peek::<(PubToken, FnToken)>().is_some() {
            let item_fn = parser.parse()?;
            return Ok(ItemKind::Fn(item_fn));
        }
        if parser.peek::<TraitToken>().is_some()
            || parser.peek::<(PubToken, TraitToken)>().is_some()
        {
            let item_trait = parser.parse()?;
            return Ok(ItemKind::Trait(item_trait));
        }
        if let Some(item) = parser.guarded_parse::<ImplToken, _>()? {
            return Ok(ItemKind::Impl(item));
        }
        if let Some(item) = parser.guarded_parse::<AbiToken, _>()? {
            return Ok(ItemKind::Abi(item));
        }
        if parser.peek::<ConstToken>().is_some()
            || parser.peek::<(PubToken, ConstToken)>().is_some()
        {
            let item_const = parser.parse()?;
            return Ok(ItemKind::Const(item_const));
        }
        if let Some(item) = parser.guarded_parse::<StorageToken, _>()? {
            return Ok(ItemKind::Storage(item));
        }
        if let Some(item) = parser.guarded_parse::<BreakToken, _>()? {
            return Ok(ItemKind::Break(item));
        }
        if let Some(item) = parser.guarded_parse::<ContinueToken, _>()? {
            return Ok(ItemKind::Continue(item));
        }
        Err(parser.emit_error(ParseErrorKind::ExpectedAnItem))
    }
}

impl Parse for TypeField {
    fn parse(parser: &mut Parser) -> ParseResult<TypeField> {
        // TODO: Remove this when `TypeField`s are annotated.
        // Eat DocComments to prevent errors in existing code.
        while let Some(DocComment {
            doc_style: DocStyle::Outer,
            ..
        }) = parser.peek::<DocComment>()
        {
            let _ = parser.parse::<DocComment>()?;
        }
        Ok(TypeField {
            name: parser.parse()?,
            colon_token: parser.parse()?,
            ty: parser.parse()?,
        })
    }
}

impl ParseToEnd for FnArgs {
    fn parse_to_end<'a, 'e>(
        mut parser: Parser<'a, 'e>,
    ) -> ParseResult<(FnArgs, ParserConsumed<'a>)> {
        let mutable_self = parser.take::<MutToken>();
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
        Ok(FnArg {
            pattern: parser.parse()?,
            colon_token: parser.parse()?,
            ty: parser.parse()?,
        })
    }
}

impl Parse for FnSignature {
    fn parse(parser: &mut Parser) -> ParseResult<FnSignature> {
        Ok(FnSignature {
            visibility: parser.take(),
            fn_token: parser.parse()?,
            name: parser.parse()?,
            generics: parser.guarded_parse::<OpenAngleBracketToken, _>()?,
            arguments: parser.parse()?,
            return_type_opt: match parser.take() {
                Some(right_arrow_token) => {
                    let ty = parser.parse()?;
                    Some((right_arrow_token, ty))
                }
                None => None,
            },
            where_clause_opt: parser.guarded_parse::<WhereToken, _>()?,
        })
    }
}

// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use sway_ast::Item;

    fn parse_item(input: &str) -> Item {
        let token_stream = crate::token::lex(&Arc::from(input), 0, input.len(), None).unwrap();
        let mut errors = Vec::new();
        let mut parser = Parser::new(&token_stream, &mut errors);
        match Item::parse(&mut parser) {
            Ok(item) => item,
            Err(_) => {
                panic!("Parse error: {:?}", errors);
            }
        }
    }

    #[test]
    fn parse_doc_comment() {
        let item = parse_item(
            r#"
            // I will be ignored.
            //! I will be ignored.
            /// This is a doc comment.
            //! I will be ignored.
            // I will be ignored.
            fn f() -> bool {
                false
            }
            "#,
        );

        assert!(matches!(item.value, ItemKind::Fn(_)));

        assert_eq!(item.attribute_list.len(), 1);

        let attrib = item.attribute_list.get(0).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "doc");
        assert!(attrib.attribute.get().args.is_some());

        let mut args = attrib
            .attribute
            .get()
            .args
            .as_ref()
            .unwrap()
            .get()
            .into_iter();
        assert_eq!(
            args.next().map(|arg| arg.as_str()),
            Some(" This is a doc comment.")
        );
        assert_eq!(args.next().map(|arg| arg.as_str()), None);
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

    #[test]
    fn parse_attributes_doc_comment() {
        let item = parse_item(
            r#"
            /// This is a doc comment.
            /// This is another doc comment.
            fn f() -> bool {
                false
            }
            "#,
        );

        assert!(matches!(item.value, ItemKind::Fn(_)));

        assert_eq!(item.attribute_list.len(), 2);

        for i in 0..2 {
            let attrib = item.attribute_list.get(i).unwrap();
            assert_eq!(attrib.attribute.get().name.as_str(), "doc");
            assert!(attrib.attribute.get().args.is_some());

            let mut args = attrib
                .attribute
                .get()
                .args
                .as_ref()
                .unwrap()
                .get()
                .into_iter();
            assert_eq!(
                args.next().map(|arg| arg.as_str()),
                match i {
                    0 => Some(" This is a doc comment."),
                    1 => Some(" This is another doc comment."),
                    _ => unreachable!(),
                }
            );
            assert_eq!(args.next().map(|arg| arg.as_str()), None);
        }
    }
}

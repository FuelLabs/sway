use super::{MaybeResolvedType, ResolvedType};
use crate::build_config::BuildConfig;
use crate::error::*;
use crate::span::Span;
use crate::types::PartiallyResolvedType;
use crate::{Ident, Rule};
use pest::iterators::Pair;
use std::iter::FromIterator;

/// Type information without an associated value, used for type inferencing and definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeInfo<'sc> {
    Str(u64),
    UnsignedInteger(IntegerBits),
    Boolean,
    /// A custom type could be a struct or similar if the name is in scope,
    /// or just a generic parameter if it is not.
    /// At parse time, there is no sense of scope, so this determination is not made
    /// until the semantic analysis stage.
    Custom {
        name: Ident<'sc>,
    },
    Unit,
    SelfType,
    Byte,
    B256,
    /// This means that specific type of a number is not yet known. It will be
    /// determined via inference at a later time.
    Numeric,
    Contract,
    // used for recovering from errors in the ast
    ErrorRecovery,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum IntegerBits {
    Eight,
    Sixteen,
    ThirtyTwo,
    SixtyFour,
}

impl<'sc> TypeInfo<'sc> {
    /// This is a shortcut function. It should only be called as a convenience method in match
    /// statements resolving types when it has already been verified that this type is _not_
    /// a custom (enum, struct, user-defined) or generic type.
    /// This function just passes all the trivial types through to a [ResolvedType].
    pub(crate) fn to_resolved(&self) -> MaybeResolvedType<'sc> {
        self.attempt_naive_resolution().expect(
            "Invalid use of `to_resolved`. See documentation of [TypeInfo::to_resolved] for \
                 more details.",
        )
    }
    /// Like `to_resolved()`, but instead of panicking on failure, it returns an option.
    pub(crate) fn attempt_naive_resolution(&self) -> Option<MaybeResolvedType<'sc>> {
        Some(match self {
            TypeInfo::Custom { .. } | TypeInfo::SelfType => return None,
            TypeInfo::Boolean => MaybeResolvedType::Resolved(ResolvedType::Boolean),
            TypeInfo::Str(len) => MaybeResolvedType::Resolved(ResolvedType::Str(*len)),
            TypeInfo::Contract => MaybeResolvedType::Resolved(ResolvedType::Contract),
            TypeInfo::UnsignedInteger(bits) => {
                MaybeResolvedType::Resolved(ResolvedType::UnsignedInteger(*bits))
            }
            TypeInfo::Numeric => MaybeResolvedType::Partial(PartiallyResolvedType::Numeric),
            TypeInfo::Unit => MaybeResolvedType::Resolved(ResolvedType::Unit),
            TypeInfo::Byte => MaybeResolvedType::Resolved(ResolvedType::Byte),
            TypeInfo::B256 => MaybeResolvedType::Resolved(ResolvedType::B256),
            TypeInfo::ErrorRecovery => MaybeResolvedType::Resolved(ResolvedType::ErrorRecovery),
        })
    }

    pub(crate) fn parse_from_pair(
        input: Pair<'sc, Rule>,
        config: Option<BuildConfig>,
    ) -> CompileResult<'sc, Self> {
        let mut r#type = input.into_inner();
        Self::parse_from_pair_inner(r#type.next().unwrap(), config)
    }

    pub(crate) fn parse_from_pair_inner(
        input: Pair<'sc, Rule>,
        config: Option<BuildConfig>,
    ) -> CompileResult<'sc, Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let input = if let Some(input) = input.clone().into_inner().next() {
            input
        } else {
            input
        };
        ok(
            match input.as_str().trim() {
                "u8" => TypeInfo::UnsignedInteger(IntegerBits::Eight),
                "u16" => TypeInfo::UnsignedInteger(IntegerBits::Sixteen),
                "u32" => TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo),
                "u64" => TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
                "bool" => TypeInfo::Boolean,
                "unit" => TypeInfo::Unit,
                "byte" => TypeInfo::Byte,
                "b256" => TypeInfo::B256,
                "Self" | "self" => TypeInfo::SelfType,
                "Contract" => TypeInfo::Contract,
                "()" => TypeInfo::Unit,
                a if a.contains("str[") => check!(
                    parse_str_type(
                        a,
                        Span {
                            span: input.as_span(),
                            path: if let Some(config) = config {
                                Some(config.dir_of_code)
                            } else {
                                None
                            }
                        }
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                ),
                _other => TypeInfo::Custom {
                    name: check!(
                        Ident::parse_from_pair(input, config),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ),
                },
            },
            warnings,
            errors,
        )
    }
}

fn parse_str_type<'sc>(raw: &'sc str, span: Span<'sc>) -> CompileResult<'sc, TypeInfo<'sc>> {
    if raw.starts_with("str[") {
        let mut rest = raw.split_at("str[".len()).1.chars().collect::<Vec<_>>();
        if let Some(']') = rest.pop() {
            if let Ok(num) = String::from_iter(rest).parse() {
                return ok(TypeInfo::Str(num), vec![], vec![]);
            }
        }
        return err(
            vec![],
            vec![CompileError::InvalidStrType {
                raw: raw.to_string(),
                span,
            }],
        );
    }
    return err(vec![], vec![CompileError::UnknownType { span }]);
}

#[test]
fn test_str_parse() {
    match parse_str_type(
        "str[20]",
        Span {
            span: pest::Span::new("", 0, 0).unwrap(),
            path: None,
        },
    )
    .value
    {
        Some(value) if value == TypeInfo::Str(20) => (),
        _ => panic!("failed test"),
    }
    match parse_str_type(
        "str[]",
        Span {
            span: pest::Span::new("", 0, 0).unwrap(),
            path: None,
        },
    )
    .value
    {
        None => (),
        _ => panic!("failed test"),
    }
    match parse_str_type(
        "str[ab]",
        Span {
            span: pest::Span::new("", 0, 0).unwrap(),
            path: None,
        },
    )
    .value
    {
        None => (),
        _ => panic!("failed test"),
    }
    match parse_str_type(
        "str [ab]",
        Span {
            span: pest::Span::new("", 0, 0).unwrap(),
            path: None,
        },
    )
    .value
    {
        None => (),
        _ => panic!("failed test"),
    }

    match parse_str_type(
        "not even a str[ type",
        Span {
            span: pest::Span::new("", 0, 0).unwrap(),
            path: None,
        },
    )
    .value
    {
        None => (),
        _ => panic!("failed test"),
    }
    match parse_str_type(
        "",
        Span {
            span: pest::Span::new("", 0, 0).unwrap(),
            path: None,
        },
    )
    .value
    {
        None => (),
        _ => panic!("failed test"),
    }
    match parse_str_type(
        "20",
        Span {
            span: pest::Span::new("", 0, 0).unwrap(),
            path: None,
        },
    )
    .value
    {
        None => (),
        _ => panic!("failed test"),
    }
    match parse_str_type(
        "[20]",
        Span {
            span: pest::Span::new("", 0, 0).unwrap(),
            path: None,
        },
    )
    .value
    {
        None => (),
        _ => panic!("failed test"),
    }
}

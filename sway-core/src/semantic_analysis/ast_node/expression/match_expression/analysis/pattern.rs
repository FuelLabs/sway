use std::{cmp::Ordering, fmt};

use std::fmt::Write;
use sway_types::Span;

use crate::{
    error::{err, ok},
    language::Literal,
    semantic_analysis::{TyScrutinee, TyScrutineeVariant},
    CompileError, CompileResult, Namespace, TypeInfo,
};

use super::{patstack::PatStack, range::Range};

/// A `Pattern` represents something that could be on the LHS of a match
/// expression arm.
///
/// For instance this match expression:
///
/// ```ignore
/// let x = (0, 5);
/// match x {
///     (0, 1) => true,
///     (2, 3) => true,
///     _ => false
/// }
/// ```
///
/// would result in these patterns:
///
/// ```ignore
/// Pattern::Tuple([
///     Pattern::U64(Range { first: 0, last: 0 }),
///     Pattern::U64(Range { first: 1, last: 1 })
/// ])
/// Pattern::Tuple([
///     Pattern::U64(Range { first: 2, last: 2 }),
///     Pattern::U64(Range { first: 3, last: 3 })
/// ])
/// Pattern::Wildcard
/// ```
///
/// ---
///
/// A `Pattern` is semantically constructed from a "constructor" and its
/// "arguments." Given the `Pattern`:
///
/// ```ignore
/// Pattern::Tuple([
///     Pattern::U64(Range { first: 0, last: 0 }),
///     Pattern::U64(Range { first: 1, last: 1 })
/// ])
/// ```
///
/// the constructor is:
///
/// ```ignore
/// Pattern::Tuple([.., ..])
/// ```
///
/// and the arguments are:
///
/// ```ignore
/// [
///     Pattern::U64(Range { first: 0, last: 0 }),
///     Pattern::U64(Range { first: 1, last: 1 })
/// ]
/// ```
///
/// Given the `Pattern`:
///
/// ```ignore
/// Pattern::U64(Range { first: 0, last: 0 })
/// ```
///
/// the constructor is:
///
/// ```ignore
/// Pattern::U64(Range { first: 0, last: 0 })
/// ```
/// and the arguments are empty. More specifically, in the case of u64 (and
/// other numbers), we can think of u64 as a giant enum, where every u64 value
/// is one variant of the enum, and each of these variants maps to a `Pattern`.
/// So "2u64" can be mapped to a `Pattern` with the constructor "2u64"
/// (represented as a `Range<u64>`) and with empty arguments.
///
/// This idea of a constructor and arguments is used in the match exhaustivity
/// algorithm.
///
/// ---
///
/// The variants of `Pattern` can be semantically categorized into 3 categories:
///
/// 1. the wildcard pattern (Pattern::Wildcard)
/// 2. the or pattern (Pattern::Or(..))
/// 3. constructed patterns (everything else)
///
/// This idea of semantic categorization is used in the match exhaustivity
/// algorithm.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Pattern {
    Wildcard,
    U8(Range<u8>),
    U16(Range<u16>),
    U32(Range<u32>),
    U64(Range<u64>),
    B256([u8; 32]),
    Boolean(bool),
    Numeric(Range<u64>),
    String(String),
    Struct(StructPattern),
    Enum(EnumPattern),
    Tuple(PatStack),
    Or(PatStack),
}

impl Pattern {
    /// Converts a `Scrutinee` to a `Pattern`.
    pub(crate) fn from_scrutinee(
        namespace: &Namespace,
        scrutinee: TyScrutinee,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let pat = match scrutinee.variant {
            TyScrutineeVariant::CatchAll => Pattern::Wildcard,
            TyScrutineeVariant::Variable(_) => Pattern::Wildcard,
            TyScrutineeVariant::Literal(value) => Pattern::from_literal(value),
            TyScrutineeVariant::Constant(_, value, _) => Pattern::from_literal(value),
            TyScrutineeVariant::StructScrutinee(struct_name, fields) => {
                let mut new_fields = vec![];
                for field in fields.into_iter() {
                    let f = match field.scrutinee {
                        Some(scrutinee) => check!(
                            Pattern::from_scrutinee(namespace, scrutinee),
                            return err(warnings, errors),
                            warnings,
                            errors
                        ),
                        None => Pattern::Wildcard,
                    };
                    new_fields.push((field.field.as_str().to_string(), f));
                }
                Pattern::Struct(StructPattern {
                    struct_name: struct_name.to_string(),
                    fields: new_fields,
                })
            }
            TyScrutineeVariant::Tuple(elems) => {
                let mut new_elems = PatStack::empty();
                for elem in elems.into_iter() {
                    new_elems.push(check!(
                        Pattern::from_scrutinee(namespace, elem),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ));
                }
                Pattern::Tuple(new_elems)
            }
            TyScrutineeVariant::EnumScrutinee {
                call_path, value, ..
            } => {
                let enum_name = call_path.prefixes.last().unwrap().to_string();
                let variant_name = call_path.suffix.to_string();
                Pattern::Enum(EnumPattern {
                    enum_name,
                    variant_name,
                    value: Box::new(check!(
                        Pattern::from_scrutinee(namespace, *value),
                        return err(warnings, errors),
                        warnings,
                        errors
                    )),
                })
            }
        };
        ok(pat, warnings, errors)
    }

    /// Convert the given literal `value` into a pattern.
    fn from_literal(value: Literal) -> Pattern {
        match value {
            Literal::U8(x) => Pattern::U8(Range::from_single(x)),
            Literal::U16(x) => Pattern::U16(Range::from_single(x)),
            Literal::U32(x) => Pattern::U32(Range::from_single(x)),
            Literal::U64(x) => Pattern::U64(Range::from_single(x)),
            Literal::B256(x) => Pattern::B256(x),
            Literal::Boolean(b) => Pattern::Boolean(b),
            Literal::Numeric(x) => Pattern::Numeric(Range::from_single(x)),
            Literal::String(s) => Pattern::String(s.as_str().to_string()),
        }
    }

    /// Converts a `PatStack` to a `Pattern`. If the `PatStack` is of lenth 1,
    /// this function returns the single element, if it is of length > 1, this
    /// function wraps the provided `PatStack` in a `Pattern::Or(..)`.
    pub(crate) fn from_pat_stack(pat_stack: PatStack, span: &Span) -> CompileResult<Pattern> {
        if pat_stack.len() == 1 {
            pat_stack.first(span)
        } else {
            ok(Pattern::Or(pat_stack), vec![], vec![])
        }
    }

    /// Given a `Pattern` *c* and a `PatStack` *args*, extracts the constructor
    /// from *c* and applies it to *args*. For example, given:
    ///
    /// ```ignore
    /// c:    Pattern::Tuple([
    ///         Pattern::U64(Range { first: 5, last: 7, }),
    ///         Pattern::U64(Range { first: 10, last: 12 })
    ///       ])
    /// args: [
    ///         Pattern::U64(Range { first: 0, last: 0 }),
    ///         Pattern::U64(Range { first: 1, last: 1 })
    ///       ]
    /// ```
    ///
    /// the extracted constructor *ctor* from *c* would be:
    ///
    /// ```ignore
    /// Pattern::Tuple([.., ..])
    /// ```
    ///
    /// Applying *args* to *ctor* would give:
    ///
    /// ```ignore
    /// Pattern::Tuple([
    ///     Pattern::U64(Range { first: 0, last: 0 }),
    ///     Pattern::U64(Range { first: 1, last: 1 })
    /// ])
    /// ```
    ///
    /// ---
    ///
    /// If if is the case that at lease one element of *args* is a
    /// or-pattern, then *args* is first "serialized". Meaning, that all
    /// or-patterns are extracted to create a vec of `PatStack`s *args*' where
    /// each `PatStack` is a copy of *args* where the index of the or-pattern is
    /// instead replaced with one element from the or-patterns contents. More
    /// specifically, given an *args* with one or-pattern that contains n
    /// elements, this "serialization" would result in *args*' of length n.
    /// Given an *args* with two or-patterns that contain n elements and m
    /// elements, this would result in *args*' of length n*m.
    ///
    /// Once *args*' is constructed, *ctor* is applied to every element of
    /// *args*' and the resulting `Pattern`s are wrapped inside of an
    /// or-pattern.
    ///
    /// For example, given:
    ///
    /// ```ignore
    /// ctor: Pattern::Tuple([.., ..])
    /// args: [
    ///         Pattern::Or([
    ///             Pattern::U64(Range { first: 0, last: 0 }),
    ///             Pattern::U64(Range { first: 1, last: 1 })
    ///         ]),
    ///         Pattern::Wildcard
    ///       ]
    /// ```
    ///
    /// *args* would serialize to:
    ///
    /// ```ignore
    /// [
    ///     [
    ///         Pattern::U64(Range { first: 0, last: 0 }),
    ///         Pattern::Wildcard
    ///     ],
    ///     [
    ///         Pattern::U64(Range { first: 1, last: 1 }),
    ///         Pattern::Wildcard
    ///     ]
    /// ]
    /// ```
    ///
    /// applying *ctor* would create:
    ///
    /// ```ignore
    /// [
    ///     Pattern::Tuple([
    ///         Pattern::U64(Range { first: 0, last: 0 }),
    ///         Pattern::Wildcard
    ///     ]),
    ///     Pattern::Tuple([
    ///         Pattern::U64(Range { first: 1, last: 1 }),
    ///         Pattern::Wildcard
    ///     ]),
    /// ]
    /// ```
    ///
    /// and wrapping this in an or-pattern would create:
    ///
    /// ```ignore
    /// Pattern::Or([
    ///     Pattern::Tuple([
    ///         Pattern::U64(Range { first: 0, last: 0 }),
    ///         Pattern::Wildcard
    ///     ]),
    ///     Pattern::Tuple([
    ///         Pattern::U64(Range { first: 1, last: 1 }),
    ///         Pattern::Wildcard
    ///     ]),
    /// ])
    /// ```
    pub(crate) fn from_constructor_and_arguments(
        c: &Pattern,
        args: PatStack,
        span: &Span,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let pat = match c {
            Pattern::Wildcard => unreachable!(),
            Pattern::U8(range) => {
                if !args.is_empty() {
                    errors.push(CompileError::Internal(
                        "malformed constructor request",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                }
                Pattern::U8(range.clone())
            }
            Pattern::U16(range) => {
                if !args.is_empty() {
                    errors.push(CompileError::Internal(
                        "malformed constructor request",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                }
                Pattern::U16(range.clone())
            }
            Pattern::U32(range) => {
                if !args.is_empty() {
                    errors.push(CompileError::Internal(
                        "malformed constructor request",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                }
                Pattern::U32(range.clone())
            }
            Pattern::U64(range) => {
                if !args.is_empty() {
                    errors.push(CompileError::Internal(
                        "malformed constructor request",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                }
                Pattern::U64(range.clone())
            }
            Pattern::B256(b) => {
                if !args.is_empty() {
                    errors.push(CompileError::Internal(
                        "malformed constructor request",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                }
                Pattern::B256(*b)
            }
            Pattern::Boolean(b) => {
                if !args.is_empty() {
                    errors.push(CompileError::Internal(
                        "malformed constructor request",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                }
                Pattern::Boolean(*b)
            }
            Pattern::Numeric(range) => {
                if !args.is_empty() {
                    errors.push(CompileError::Internal(
                        "malformed constructor request",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                }
                Pattern::Numeric(range.clone())
            }
            Pattern::String(s) => {
                if !args.is_empty() {
                    errors.push(CompileError::Internal(
                        "malformed constructor request",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                }
                Pattern::String(s.clone())
            }
            Pattern::Struct(struct_pattern) => {
                if args.len() != struct_pattern.fields.len() {
                    errors.push(CompileError::Internal(
                        "malformed constructor request",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                }
                let pats: PatStack = check!(
                    args.serialize_multi_patterns(span),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
                .into_iter()
                .map(|args| {
                    Pattern::Struct(StructPattern {
                        struct_name: struct_pattern.struct_name.clone(),
                        fields: struct_pattern
                            .fields
                            .iter()
                            .zip(args.into_iter())
                            .map(|((name, _), arg)| (name.clone(), arg))
                            .collect::<Vec<_>>(),
                    })
                })
                .collect::<Vec<_>>()
                .into();
                check!(
                    Pattern::from_pat_stack(pats, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            Pattern::Enum(enum_pattern) => {
                if args.len() != 1 {
                    errors.push(CompileError::Internal(
                        "malformed constructor request",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                }
                let serialized_args = check!(
                    args.serialize_multi_patterns(span),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let mut pats: PatStack = PatStack::empty();
                for args in serialized_args.into_iter() {
                    let arg = check!(
                        args.first(span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    pats.push(Pattern::Enum(EnumPattern {
                        enum_name: enum_pattern.enum_name.clone(),
                        variant_name: enum_pattern.variant_name.clone(),
                        value: Box::new(arg),
                    }));
                }
                check!(
                    Pattern::from_pat_stack(pats, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            Pattern::Tuple(elems) => {
                if elems.len() != args.len() {
                    errors.push(CompileError::Internal(
                        "malformed constructor request",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                }
                let pats: PatStack = check!(
                    args.serialize_multi_patterns(span),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
                .into_iter()
                .map(Pattern::Tuple)
                .collect::<Vec<_>>()
                .into();
                check!(
                    Pattern::from_pat_stack(pats, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            Pattern::Or(_) => unreachable!(),
        };
        ok(pat, warnings, errors)
    }

    /// Create a `Pattern::Wildcard`
    pub(crate) fn wild_pattern() -> Self {
        Pattern::Wildcard
    }

    /// Finds the "a value" of the `Pattern`, AKA the number of sub-patterns
    /// used in the pattern's constructor. For example, the pattern
    /// `Pattern::Tuple([.., ..])` would have an "a value" of 2.
    pub(crate) fn a(&self) -> usize {
        match self {
            Pattern::U8(_) => 0,
            Pattern::U16(_) => 0,
            Pattern::U32(_) => 0,
            Pattern::U64(_) => 0,
            Pattern::B256(_) => 0,
            Pattern::Boolean(_) => 0,
            Pattern::Numeric(_) => 0,
            Pattern::String(_) => 0,
            Pattern::Struct(StructPattern { fields, .. }) => fields.len(),
            Pattern::Enum(_) => 1,
            Pattern::Tuple(elems) => elems.len(),
            Pattern::Wildcard => unreachable!(),
            Pattern::Or(_) => unreachable!(),
        }
    }

    /// Checks to see if two `Pattern` have the same constructor. For example,
    /// given the patterns:
    ///
    /// ```ignore
    /// A: Pattern::U64(Range { first: 0, last: 0 })
    /// B: Pattern::U64(Range { first: 0, last: 0 })
    /// C: Pattern::U64(Range { first: 1, last: 1 })
    /// ```
    ///
    /// A and B have the same constructor but A and C do not.
    ///
    /// Given the patterns:
    ///
    /// ```ignore
    /// A: Pattern::Tuple([
    ///     Pattern::U64(Range { first: 0, last: 0 }),
    ///     Pattern::U64(Range { first: 1, last: 1 }),
    ///    ])
    /// B: Pattern::Tuple([
    ///     Pattern::U64(Range { first: 2, last: 2 }),
    ///     Pattern::U64(Range { first: 3, last: 3 }),
    ///    ])
    /// C: Pattern::Tuple([
    ///     Pattern::U64(Range { first: 4, last: 4 }),
    ///    ])
    /// ```
    ///
    /// A and B have the same constructor but A and C do not.
    pub(crate) fn has_the_same_constructor(&self, other: &Pattern) -> bool {
        match (self, other) {
            (Pattern::Wildcard, Pattern::Wildcard) => true,
            (Pattern::U8(a), Pattern::U8(b)) => a == b,
            (Pattern::U16(a), Pattern::U16(b)) => a == b,
            (Pattern::U32(a), Pattern::U32(b)) => a == b,
            (Pattern::U64(a), Pattern::U64(b)) => a == b,
            (Pattern::B256(a), Pattern::B256(b)) => a == b,
            (Pattern::Boolean(a), Pattern::Boolean(b)) => a == b,
            (Pattern::Numeric(a), Pattern::Numeric(b)) => a == b,
            (Pattern::String(a), Pattern::String(b)) => a == b,
            (
                Pattern::Struct(StructPattern {
                    struct_name: struct_name1,
                    fields: fields1,
                }),
                Pattern::Struct(StructPattern {
                    struct_name: struct_name2,
                    fields: fields2,
                }),
            ) => struct_name1 == struct_name2 && fields1.len() == fields2.len(),
            (
                Pattern::Enum(EnumPattern {
                    enum_name: enum_name1,
                    variant_name: variant_name1,
                    ..
                }),
                Pattern::Enum(EnumPattern {
                    enum_name: enum_name2,
                    variant_name: variant_name2,
                    ..
                }),
            ) => enum_name1 == enum_name2 && variant_name1 == variant_name2,
            (Pattern::Tuple(elems1), Pattern::Tuple(elems2)) => elems1.len() == elems2.len(),
            (Pattern::Or(_), Pattern::Or(_)) => unreachable!(),
            _ => false,
        }
    }

    /// Extracts the "sub-patterns" of a `Pattern`, aka the "arguments" to the
    /// patterns "constructor". Some patterns have 0 sub-patterns and some
    /// patterns have >0 sub-patterns. For example, this pattern:
    ///
    /// ```ignore
    /// Pattern::U64(Range { first: 0, last: 0 }),
    /// ```
    ///
    /// has 0 sub-patterns. While this pattern:
    ///
    /// ```ignore
    /// Pattern::Tuple([
    ///     Pattern::U64(Range { first: 0, last: 0 }),
    ///     Pattern::U64(Range { first: 1, last: 1 })
    /// ])
    /// ```
    ///
    /// has 2 sub-patterns:
    ///
    /// ```ignore
    /// [
    ///     Pattern::U64(Range { first: 0, last: 0 }),
    ///     Pattern::U64(Range { first: 1, last: 1 })
    /// ]
    /// ```
    pub(crate) fn sub_patterns(&self, span: &Span) -> CompileResult<PatStack> {
        let warnings = vec![];
        let mut errors = vec![];
        let pats = match self {
            Pattern::Struct(StructPattern { fields, .. }) => fields
                .iter()
                .map(|(_, field)| field.to_owned())
                .collect::<Vec<_>>()
                .into(),
            Pattern::Enum(EnumPattern { value, .. }) => PatStack::from_pattern((**value).clone()),
            Pattern::Tuple(elems) => elems.to_owned(),
            _ => PatStack::empty(),
        };
        if self.a() != pats.len() {
            errors.push(CompileError::Internal(
                "invariant self.a() == pats.len() broken",
                span.clone(),
            ));
            return err(warnings, errors);
        }
        ok(pats, warnings, errors)
    }

    /// Flattens a `Pattern` into a `PatStack`. If the pattern is an
    /// "or-pattern", return its contents, otherwise return the pattern as a
    /// `PatStack`
    pub(crate) fn flatten(&self) -> PatStack {
        match self {
            Pattern::Or(pats) => pats.to_owned(),
            pat => PatStack::from_pattern(pat.to_owned()),
        }
    }

    pub(crate) fn matches_type_info(&self, type_info: &TypeInfo) -> bool {
        match (self, type_info) {
            (
                Pattern::Enum(EnumPattern {
                    enum_name: l_enum_name,
                    variant_name,
                    ..
                }),
                TypeInfo::Enum {
                    name: r_enum_name,
                    variant_types,
                    ..
                },
            ) => {
                l_enum_name.as_str() == r_enum_name.as_str()
                    && variant_types
                        .iter()
                        .map(|variant_type| variant_type.name.clone())
                        .any(|name| name.as_str() == variant_name.as_str())
            }
            _ => false, // NOTE: We may need to expand this in the future
        }
    }

    fn discriminant_value(&self) -> usize {
        match self {
            Pattern::Wildcard => 0,
            Pattern::U8(_) => 1,
            Pattern::U16(_) => 2,
            Pattern::U32(_) => 3,
            Pattern::U64(_) => 4,
            Pattern::B256(_) => 5,
            Pattern::Boolean(_) => 6,
            Pattern::Numeric(_) => 7,
            Pattern::String(_) => 8,
            Pattern::Struct(_) => 9,
            Pattern::Enum(_) => 10,
            Pattern::Tuple(_) => 11,
            Pattern::Or(_) => 12,
        }
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Pattern::Wildcard => "_".to_string(),
            Pattern::U8(range) => format!("{}", range),
            Pattern::U16(range) => format!("{}", range),
            Pattern::U32(range) => format!("{}", range),
            Pattern::U64(range) => format!("{}", range),
            Pattern::Numeric(range) => format!("{}", range),
            Pattern::B256(n) => format!("{:#?}", n),
            Pattern::Boolean(b) => format!("{}", b),
            Pattern::String(s) => s.clone(),
            Pattern::Struct(struct_pattern) => format!("{}", struct_pattern),
            Pattern::Enum(enum_pattern) => format!("{}", enum_pattern),
            Pattern::Tuple(elems) => {
                let mut builder = String::new();
                builder.push('(');
                write!(builder, "{}", elems)?;
                builder.push(')');
                builder
            }
            Pattern::Or(_) => unreachable!(),
        };
        write!(f, "{}", s)
    }
}

impl std::cmp::Ord for Pattern {
    fn cmp(&self, other: &Self) -> Ordering {
        use Ordering::*;

        match (self, other) {
            (Pattern::Wildcard, Pattern::Wildcard) => Equal,
            (Pattern::U8(x), Pattern::U8(y)) => x.cmp(y),
            (Pattern::U16(x), Pattern::U16(y)) => x.cmp(y),
            (Pattern::U32(x), Pattern::U32(y)) => x.cmp(y),
            (Pattern::U64(x), Pattern::U64(y)) => x.cmp(y),
            (Pattern::B256(x), Pattern::B256(y)) => x.cmp(y),
            (Pattern::Boolean(x), Pattern::Boolean(y)) => x.cmp(y),
            (Pattern::Numeric(x), Pattern::Numeric(y)) => x.cmp(y),
            (Pattern::String(x), Pattern::String(y)) => x.cmp(y),
            (Pattern::Struct(x), Pattern::Struct(y)) => x.cmp(y),
            (Pattern::Enum(x), Pattern::Enum(y)) => x.cmp(y),
            (Pattern::Tuple(x), Pattern::Tuple(y)) => x.cmp(y),
            (Pattern::Or(x), Pattern::Or(y)) => x.cmp(y),
            (x, y) => x.discriminant_value().cmp(&y.discriminant_value()),
        }
    }
}

impl std::cmp::PartialOrd for Pattern {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StructPattern {
    struct_name: String,
    fields: Vec<(String, Pattern)>,
}

impl StructPattern {
    pub(crate) fn new(struct_name: String, fields: Vec<(String, Pattern)>) -> Self {
        StructPattern {
            struct_name,
            fields,
        }
    }

    pub(crate) fn struct_name(&self) -> &String {
        &self.struct_name
    }

    pub(crate) fn fields(&self) -> &Vec<(String, Pattern)> {
        &self.fields
    }
}

impl fmt::Display for StructPattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut builder = String::new();
        builder.push_str(self.struct_name.as_str());
        builder.push_str(" { ");
        let mut start_of_wildcard_tail = None;
        for (i, (_, pat)) in self.fields.iter().enumerate().rev() {
            match (pat, start_of_wildcard_tail) {
                (Pattern::Wildcard, None) => {}
                (_, None) => start_of_wildcard_tail = Some(i + 1),
                (_, _) => {}
            }
        }
        let s: String = match start_of_wildcard_tail {
            Some(start_of_wildcard_tail) => {
                let (front, rest) = self.fields.split_at(start_of_wildcard_tail);
                let mut inner_builder = front
                    .iter()
                    .map(|(name, field)| -> Result<_, fmt::Error> {
                        let mut inner_builder = String::new();
                        inner_builder.push_str(name);
                        inner_builder.push_str(": ");
                        write!(inner_builder, "{}", field)?;
                        Ok(inner_builder)
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                if !rest.is_empty() {
                    inner_builder.push_str(", ...");
                }
                inner_builder
            }
            None => self
                .fields
                .iter()
                .map(|(name, field)| -> Result<_, fmt::Error> {
                    let mut inner_builder = String::new();
                    inner_builder.push_str(name);
                    inner_builder.push_str(": ");
                    write!(inner_builder, "{}", field)?;
                    Ok(inner_builder)
                })
                .collect::<Result<Vec<_>, _>>()?
                .join(", "),
        };
        builder.push_str(&s);
        builder.push_str(" }");
        write!(f, "{}", builder)
    }
}

impl std::cmp::Ord for StructPattern {
    fn cmp(&self, other: &Self) -> Ordering {
        use Ordering::*;

        match self.struct_name.cmp(&other.struct_name) {
            Equal => self.fields.cmp(&other.fields),
            res => res,
        }
    }
}

impl std::cmp::PartialOrd for StructPattern {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EnumPattern {
    pub(crate) enum_name: String,
    pub(crate) variant_name: String,
    pub(crate) value: Box<Pattern>,
}

impl std::cmp::Ord for EnumPattern {
    fn cmp(&self, other: &Self) -> Ordering {
        use Ordering::*;

        match (
            self.enum_name.cmp(&other.enum_name),
            self.variant_name.cmp(&other.variant_name),
            (*self.value).cmp(&*other.value),
        ) {
            // enum name is the first element to order by
            (Less, _, _) => Less,
            (Greater, _, _) => Greater,

            // variant name is the second element to order by
            (Equal, Less, _) => Less,
            (Equal, Greater, _) => Greater,

            // value is the last element to order by
            (Equal, Equal, Less) => Less,
            (Equal, Equal, Equal) => Equal,
            (Equal, Equal, Greater) => Greater,
        }
    }
}

impl std::cmp::PartialOrd for EnumPattern {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for EnumPattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut builder = String::new();
        builder.push_str(self.enum_name.as_str());
        builder.push_str("::");
        builder.push_str(self.variant_name.as_str());
        builder.push('(');
        builder.push_str(&self.value.to_string());
        builder.push(')');
        write!(f, "{}", builder)
    }
}

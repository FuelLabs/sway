use crate::error::*;
use crate::semantic_analysis::TypedExpression;
use crate::types::{IntegerBits, ResolvedType};
use crate::Span;
use crate::{error::*, semantic_analysis::ast_node::TypedStructField, CallPath, Ident};
use derivative::Derivative;
use std::collections::HashMap;
use std::iter::FromIterator;

pub trait TypeEngine<'sc> {
    type TypeId;
    type TypeInfo;
    type ResolvedType;
    type Error;
    /// Insert a new bit of inference information about a specific type. Receive a [Self::TypeId]
    /// representing that information in return.
    fn insert(&mut self, info: Self::TypeInfo) -> Self::TypeId;
    /// Attempt to unify two type ids into one equivalence class. Throw an error if it is impossible.
    fn unify(
        &mut self,
        a: Self::TypeId,
        b: Self::TypeId,
        span: &Span<'sc>,
    ) -> Result<Option<Warning<'sc>>, Self::Error>;
    /// Like `unify`, but also takes a self type in case either type is Self.
    fn unify_with_self(
        &mut self,
        a: Self::TypeId,
        b: Self::TypeId,
        self_type: Self::TypeId,
        span: &Span<'sc>,
    ) -> Result<Option<Warning<'sc>>, Self::Error>;
    /// Attempt to reconstruct a concrete type from the given type term ID. This
    /// may fail if we don't yet have enough information to figure out what the
    /// type is.
    fn resolve(
        &self,
        id: Self::TypeId,
        span: &Span<'sc>,
    ) -> Result<Self::ResolvedType, Self::Error>;

    /// Looks up a type id and asserts that it is known. Panics if it is not
    fn look_up_type_id(&self, id: Self::TypeId) -> ResolvedType<'sc>;
}

/// A concrete type that has been fully inferred
#[derive(Debug)]
enum Type {
    Num,
    Bool,
    List(Box<Type>),
    Func(Box<Type>, Box<Type>),
}

/// A identifier to uniquely refer to our type terms
pub type TypeId = usize;

pub(crate) trait FriendlyTypeString {
    fn friendly_type_str(&self) -> String;
}

impl FriendlyTypeString for TypeId {
    fn friendly_type_str(&self) -> String {
        todo!("global engine")
    }
}

/// Type information without an associated value, used for type inferencing and definition.
#[derive(Derivative)]
#[derivative(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TypeInfo<'sc> {
    Unknown,
    Str(u64),
    UnsignedInteger(IntegerBits),
    Enum {
        name: Ident<'sc>,
        variant_types: Vec<TypeId>,
    },
    Struct {
        name: Ident<'sc>,
        fields: Vec<TypedStructField<'sc>>,
    },
    Boolean,
    /// A custom type could be a struct or similar if the name is in scope,
    /// or just a generic parameter if it is not.
    /// At parse time, there is no sense of scope, so this determination is not made
    /// until the semantic analysis stage.
    Custom {
        name: crate::Ident<'sc>,
    },
    /// For the type inference engine to use when a type references another type
    Ref(TypeId),

    Unit,
    /// Represents a type which contains methods to issue a contract call.
    /// The specific contract is identified via the `Ident` within.
    ContractCaller {
        abi_name: CallPath<'sc>,
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        address: Box<TypedExpression<'sc>>,
    },
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

impl Default for TypeInfo<'_> {
    fn default() -> Self {
        TypeInfo::Unknown
    }
}

impl<'sc> TypeInfo<'sc> {
    pub(crate) fn parse_from_pair(
        input: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<'sc, Self> {
        let mut r#type = input.into_inner();
        Self::parse_from_pair_inner(r#type.next().unwrap(), config)
    }

    pub(crate) fn parse_from_pair_inner(
        input: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
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
                            path: config.map(|config| config.dir_of_code.clone())
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

    pub(crate) fn friendly_type_str(&self) -> String {
        use TypeInfo::*;
        match self {
            Unknown => "unknown".into(),
            Str(x) => format!("str[{}]", x),
            UnsignedInteger(x) => match x {
                IntegerBits::Eight => "u8",
                IntegerBits::Sixteen => "u16",
                IntegerBits::ThirtyTwo => "u32",
                IntegerBits::SixtyFour => "u64",
            }
            .into(),
            Boolean => "bool".into(),
            Custom { name } => format!("{}", name.primary_name),
            Ref(id) => format!("T{}", id),
            Unit => "()".into(),
            SelfType => "Self".into(),
            Byte => "byte".into(),
            B256 => "b256".into(),
            Numeric => "numeric".into(),
            Contract => "contract".into(),
            ErrorRecovery => "unknown due to error".into(),
            Enum { name, .. } => format!("enum {}", name.primary_name),
            Struct { name, .. } => format!("struct {}", name.primary_name),
            ContractCaller { abi_name, .. } => {
                format!("contract caller {}", abi_name.suffix.primary_name)
            }
        }
    }
}

#[derive(Default, Clone, Debug)]
pub(crate) struct Engine<'sc> {
    id_counter: usize, // Used to generate unique IDs
    vars: HashMap<TypeId, TypeInfo<'sc>>,
}

impl<'sc> TypeEngine<'sc> for Engine<'sc> {
    type TypeId = usize;
    type TypeInfo = TypeInfo<'sc>;
    type ResolvedType = ResolvedType<'sc>;
    type Error = TypeError<'sc>;
    /// Create a new type term with whatever we have about its type
    fn insert(&mut self, info: TypeInfo<'sc>) -> TypeId {
        // Generate a new ID for our type term
        self.id_counter += 1;
        let id = self.id_counter;
        self.vars.insert(id, info);
        id
    }

    fn unify_with_self(
        &mut self,
        a: Self::TypeId,
        b: Self::TypeId,
        self_type: Self::TypeId,
        span: &Span<'sc>,
    ) -> Result<Option<Warning<'sc>>, Self::Error> {
        let a = if self.vars[&a] == TypeInfo::SelfType {
            self_type
        } else {
            a
        };
        let b = if self.vars[&b] == TypeInfo::SelfType {
            self_type
        } else {
            b
        };

        self.unify(a, b, span)
    }
    /// Make the types of two type terms equivalent (or produce an error if
    /// there is a conflict between them)
    fn unify(
        &mut self,
        a: Self::TypeId,
        b: Self::TypeId,
        span: &Span<'sc>,
    ) -> Result<Option<Warning<'sc>>, Self::Error> {
        use TypeInfo::*;
        match (self.vars[&a].clone(), self.vars[&b].clone()) {
            // Follow any references
            (Ref(a), _) => self.unify(a, b, span),
            (_, Ref(b)) => self.unify(a, b, span),

            // When we don't know anything about either term, assume that
            // they match and make the one we know nothing about reference the
            // one we may know something about
            (Unknown, _) => {
                self.vars.insert(a, TypeInfo::Ref(b));
                Ok(None)
            }
            (_, Unknown) => {
                self.vars.insert(b, TypeInfo::Ref(a));
                Ok(None)
            }

            // Primitives are trivial to unify
            (Numeric, Numeric) => Ok(None),
            (Boolean, Boolean) => Ok(None),
            (B256, B256) => Ok(None),
            (Byte, Byte) => Ok(None),
            (UnsignedInteger(x), UnsignedInteger(y)) => match numeric_cast_compat(x, y) {
                NumericCastCompatResult::CastableWithWarning(warn) => {
                    // cast the one on the right to the one on the left
                    self.vars.insert(a, UnsignedInteger(x));
                    Ok(Some(warn))
                }
                // do nothing if compatible
                NumericCastCompatResult::Compatible => Ok(None),
            },
            (Numeric, b @ UnsignedInteger(_)) => {
                self.vars.insert(a, b);
                Ok(None)
            }
            (a @ UnsignedInteger(_), Numeric) => {
                self.vars.insert(b, a);
                Ok(None)
            }
            (Enum { .. }, _) | (_, Enum { .. }) => todo!("enum ty"),
            (Struct { .. }, _) | (_, Struct { .. }) => todo!("struct ty"),

            // When unifying complex types, we must check their sub-types. This
            // can be trivially implemented for tuples, sum types, etc.
            // (List(a_item), List(b_item)) => self.unify(a_item, b_item),
            // this can be used for curried function types but we might not want that
            // (Func(a_i, a_o), Func(b_i, b_o)) => {
            //     self.unify(a_i, b_i).and_then(|_| self.unify(a_o, b_o))
            // }

            // If no previous attempts to unify were successful, raise an error
            (a, b) => Err(TypeError::MismatchedType {
                expected: a.friendly_type_str(),
                received: b.friendly_type_str(),
                help_text: Default::default(),
                span: span.clone(),
            }),
        }
    }

    fn resolve(
        &self,
        id: Self::TypeId,
        span: &Span<'sc>,
    ) -> Result<Self::ResolvedType, Self::Error> {
        use TypeInfo::*;
        match self.vars[&id] {
            Unknown => Err(TypeError::UnknownType { span: span.clone() }),
            Ref(id) => self.resolve(id, span),
            // defaults to u64
            Numeric => Ok(ResolvedType::UnsignedInteger(IntegerBits::SixtyFour)),
            Boolean => Ok(ResolvedType::Boolean),
            // List(item) => todo!("Ok(ResolvedType::List(Box::new(self.reconstruct(item)?)))"),
            // Func(i, o) => Ok(ResolvedType::Function {
            //     from: Box::new(self.resolve(i)?),
            //     to: Box::new(self.resolve(o)?),
            // }),
            UnsignedInteger(x) => Ok(ResolvedType::UnsignedInteger(x)),
            ref a => todo!("{:?}", a),
        }
    }
    fn look_up_type_id(&self, id: TypeId) -> ResolvedType<'sc> {
        self.resolve(
            id,
            &Span {
                span: pest::Span::new(
                    "because we \"expect\" here, we don't need this error span.",
                    0,
                    0,
                )
                .unwrap(),
                path: Default::default(),
            },
        )
        .expect("Internal error: type ID did not exist in type engine")
    }
}

enum NumericCastCompatResult<'sc> {
    Compatible,
    CastableWithWarning(Warning<'sc>),
}
fn numeric_cast_compat<'sc>(a: IntegerBits, b: IntegerBits) -> NumericCastCompatResult<'sc> {
    // if this is a downcast, warn for loss of precision. if upcast, then no warning.
    use IntegerBits::*;
    match (a, b) {
        // these should generate a downcast warning
        (Eight, Sixteen)
        | (Eight, ThirtyTwo)
        | (Eight, SixtyFour)
        | (Sixteen, ThirtyTwo)
        | (Sixteen, SixtyFour)
        | (ThirtyTwo, SixtyFour) => {
            NumericCastCompatResult::CastableWithWarning(Warning::LossOfPrecision {
                initial_type: a,
                cast_to: b,
            })
        }
        // upcasting is ok, so everything else is ok
        _ => NumericCastCompatResult::Compatible,
    }
}

#[test]
fn basic_numeric_unknown() {
    let mut engine = Engine::default();

    let sp = Span {
        span: pest::Span::new(" ", 0, 0).unwrap(),
        path: None,
    };
    // numerics
    let id = engine.insert(TypeInfo::Numeric);
    let id2 = engine.insert(TypeInfo::UnsignedInteger(IntegerBits::Eight));

    // Unify them together...
    engine.unify(id, id2, &sp).unwrap();

    assert_eq!(
        engine.resolve(id, &sp).unwrap(),
        ResolvedType::UnsignedInteger(IntegerBits::Eight)
    );
}
#[test]
fn chain_of_refs() {
    let mut engine = Engine::default();
    let sp = Span {
        span: pest::Span::new(" ", 0, 0).unwrap(),
        path: None,
    };
    // numerics
    let id = engine.insert(TypeInfo::Numeric);
    let id2 = engine.insert(TypeInfo::Ref(id));
    let id3 = engine.insert(TypeInfo::Ref(id));
    let id4 = engine.insert(TypeInfo::UnsignedInteger(IntegerBits::Eight));

    // Unify them together...
    engine.unify(id4, id2, &sp).unwrap();

    assert_eq!(
        engine.resolve(id3, &sp).unwrap(),
        ResolvedType::UnsignedInteger(IntegerBits::Eight)
    );
}
#[test]
fn chain_of_refs_2() {
    let mut engine = Engine::default();
    let sp = Span {
        span: pest::Span::new(" ", 0, 0).unwrap(),
        path: None,
    };
    // numerics
    let id = engine.insert(TypeInfo::Numeric);
    let id2 = engine.insert(TypeInfo::Ref(id));
    let id3 = engine.insert(TypeInfo::Ref(id));
    let id4 = engine.insert(TypeInfo::UnsignedInteger(IntegerBits::Eight));

    // Unify them together...
    engine.unify(id2, id4, &sp).unwrap();

    assert_eq!(
        engine.resolve(id3, &sp).unwrap(),
        ResolvedType::UnsignedInteger(IntegerBits::Eight)
    );
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
    err(vec![], vec![CompileError::UnknownType { span }])
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

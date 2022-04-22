use super::*;

use crate::{
    build_config::BuildConfig,
    semantic_analysis::ast_node::{TypedEnumVariant, TypedStructField},
    CallPath, Ident, Rule, TypeArgument, TypeParameter,
};

use sway_types::span::Span;

use derivative::Derivative;
use pest::iterators::Pair;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Hash, PartialEq)]
pub enum AbiName {
    Deferred,
    Known(CallPath),
}

impl std::fmt::Display for AbiName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            &(match self {
                AbiName::Deferred => "for unspecified ABI".to_string(),
                AbiName::Known(cp) => cp.to_string(),
            }),
        )
    }
}
/// Type information without an associated value, used for type inferencing and definition.
// TODO use idents instead of Strings when we have arena spans
#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub enum TypeInfo {
    Unknown,
    UnknownGeneric {
        name: Ident,
    },
    Str(u64),
    UnsignedInteger(IntegerBits),
    Enum {
        name: Ident,
        variant_types: Vec<TypedEnumVariant>,
    },
    Struct {
        name: Ident,
        fields: Vec<TypedStructField>,
    },
    Boolean,
    /// For the type inference engine to use when a type references another type
    Ref(TypeId),

    Tuple(Vec<TypeArgument>),
    /// Represents a type which contains methods to issue a contract call.
    /// The specific contract is identified via the `Ident` within.
    ContractCaller {
        abi_name: AbiName,
        // this is raw source code to be evaluated later.
        // TODO(static span): we can just use `TypedExpression` here or something more elegant
        // `TypedExpression` requires implementing a lot of `Hash` all over the place, not the
        // best...
        address: String,
    },
    /// A custom type could be a struct or similar if the name is in scope,
    /// or just a generic parameter if it is not.
    /// At parse time, there is no sense of scope, so this determination is not made
    /// until the semantic analysis stage.
    Custom {
        name: Ident,
        type_arguments: Vec<TypeArgument>,
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
    // Static, constant size arrays.
    Array(TypeId, usize),
    /// Represents the entire storage declaration struct
    /// Stored without initializers here, as typed struct fields,
    /// so type checking is able to treat it as a struct with fields.
    Storage {
        fields: Vec<TypedStructField>,
    },
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl Hash for TypeInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            TypeInfo::Str(len) => {
                state.write_u8(1);
                len.hash(state);
            }
            TypeInfo::UnsignedInteger(bits) => {
                state.write_u8(2);
                bits.hash(state);
            }
            TypeInfo::Numeric => {
                state.write_u8(3);
            }
            TypeInfo::Boolean => {
                state.write_u8(4);
            }
            TypeInfo::Tuple(fields) => {
                state.write_u8(5);
                fields.hash(state);
            }
            TypeInfo::Byte => {
                state.write_u8(6);
            }
            TypeInfo::B256 => {
                state.write_u8(7);
            }
            TypeInfo::Enum {
                name,
                variant_types,
            } => {
                state.write_u8(8);
                name.hash(state);
                variant_types.hash(state);
            }
            TypeInfo::Struct { name, fields } => {
                state.write_u8(9);
                name.hash(state);
                fields.hash(state);
            }
            TypeInfo::ContractCaller { abi_name, address } => {
                state.write_u8(10);
                abi_name.hash(state);
                address.hash(state);
            }
            TypeInfo::Contract => {
                state.write_u8(11);
            }
            TypeInfo::ErrorRecovery => {
                state.write_u8(12);
            }
            TypeInfo::Unknown => {
                state.write_u8(13);
            }
            TypeInfo::SelfType => {
                state.write_u8(14);
            }
            TypeInfo::UnknownGeneric { name } => {
                state.write_u8(15);
                name.hash(state);
            }
            TypeInfo::Custom {
                name,
                type_arguments,
            } => {
                state.write_u8(16);
                name.hash(state);
                type_arguments.hash(state);
            }
            TypeInfo::Ref(id) => {
                state.write_u8(17);
                look_up_type_id(*id).hash(state);
            }
            TypeInfo::Array(elem_ty, count) => {
                state.write_u8(18);
                look_up_type_id(*elem_ty).hash(state);
                count.hash(state);
            }
            TypeInfo::Storage { fields } => {
                state.write_u8(19);
                fields.hash(state);
            }
        }
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypeInfo {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unknown, Self::Unknown) => true,
            (Self::Boolean, Self::Boolean) => true,
            (Self::SelfType, Self::SelfType) => true,
            (Self::Byte, Self::Byte) => true,
            (Self::B256, Self::B256) => true,
            (Self::Numeric, Self::Numeric) => true,
            (Self::Contract, Self::Contract) => true,
            (Self::ErrorRecovery, Self::ErrorRecovery) => true,
            (Self::UnknownGeneric { name: l }, Self::UnknownGeneric { name: r }) => l == r,
            (
                Self::Custom {
                    name: l_name,
                    type_arguments: l_type_args,
                },
                Self::Custom {
                    name: r_name,
                    type_arguments: r_type_args,
                },
            ) => l_name == r_name && l_type_args == r_type_args,
            (Self::Str(l), Self::Str(r)) => l == r,
            (Self::UnsignedInteger(l), Self::UnsignedInteger(r)) => l == r,
            (
                Self::Enum {
                    name: l_name,
                    variant_types: l_variant_types,
                    ..
                },
                Self::Enum {
                    name: r_name,
                    variant_types: r_variant_types,
                    ..
                },
            ) => l_name == r_name && l_variant_types == r_variant_types,
            (
                Self::Struct {
                    name: l_name,
                    fields: l_fields,
                    ..
                },
                Self::Struct {
                    name: r_name,
                    fields: r_fields,
                    ..
                },
            ) => l_name == r_name && l_fields == r_fields,
            (Self::Ref(l), Self::Ref(r)) => look_up_type_id(*l) == look_up_type_id(*r),
            (Self::Tuple(l), Self::Tuple(r)) => l
                .iter()
                .zip(r.iter())
                .map(|(l, r)| look_up_type_id(l.type_id) == look_up_type_id(r.type_id))
                .all(|x| x),
            (
                Self::ContractCaller {
                    abi_name: l_abi_name,
                    address: l_address,
                },
                Self::ContractCaller {
                    abi_name: r_abi_name,
                    address: r_address,
                },
            ) => l_abi_name == r_abi_name && l_address == r_address,
            (Self::Array(l0, l1), Self::Array(r0, r1)) => {
                look_up_type_id(*l0) == look_up_type_id(*r0) && l1 == r1
            }
            (TypeInfo::Storage { fields: l_fields }, TypeInfo::Storage { fields: r_fields }) => {
                l_fields == r_fields
            }
            _ => false,
        }
    }
}

impl Eq for TypeInfo {}

impl Default for TypeInfo {
    fn default() -> Self {
        TypeInfo::Unknown
    }
}

impl TypeInfo {
    pub(crate) fn parse_from_pair(
        type_name_pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut iter = type_name_pair.into_inner();
        let input = iter.next().unwrap();
        let span = Span::from_pest(
            input.as_span(),
            config.map(|config| config.dir_of_code.clone()),
        );
        let type_info = match input.as_rule() {
            Rule::str_type => {
                let type_info = check!(
                    parse_str_type(input.as_str(), span),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                type_info
            }
            Rule::ident => {
                let type_info = check!(
                    TypeInfo::pair_as_str_to_type_info(input, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                match iter.next() {
                    Some(types) => match type_info {
                        TypeInfo::Custom { name, .. } => {
                            let type_arguments = check!(
                                TypeArgument::parse_arguments_from_pair(types, config),
                                vec!(),
                                warnings,
                                errors
                            );
                            TypeInfo::Custom {
                                name,
                                type_arguments,
                            }
                        }
                        _ => unimplemented!(),
                    },
                    None => type_info,
                }
            }
            Rule::contract_caller_type => check!(
                parse_contract_caller_type(input, config),
                return err(warnings, errors),
                warnings,
                errors
            ),
            Rule::array_type => {
                let mut array_inner_iter = input.into_inner();
                let elem_type_info = match array_inner_iter.next() {
                    None => {
                        errors.push(CompileError::Internal(
                            "Missing array element type while parsing array type.",
                            span,
                        ));
                        return err(warnings, errors);
                    }
                    Some(array_elem_type_pair) => {
                        check!(
                            Self::parse_from_pair(array_elem_type_pair, config),
                            TypeInfo::ErrorRecovery,
                            warnings,
                            errors
                        )
                    }
                };
                let elem_count: usize = match array_inner_iter.next() {
                    None => {
                        errors.push(CompileError::Internal(
                            "Missing array element count while parsing array type.",
                            span,
                        ));
                        return err(warnings, errors);
                    }
                    Some(array_elem_count_pair) => {
                        match array_elem_count_pair.as_rule() {
                            Rule::basic_integer => {
                                // Parse the count directly to a usize.
                                check!(
                                    array_elem_count_pair
                                        .as_str()
                                        .trim()
                                        .replace('_', "")
                                        .parse::<usize>()
                                        // Could probably just .unwrap() here since it will succeed.
                                        .map_or_else(
                                            |_err| {
                                                err(
                                                    Vec::new(),
                                                    vec![CompileError::Internal(
                                                        "Failed to parse array elem count as \
                                                        integer while parsing array type.",
                                                        span,
                                                    )],
                                                )
                                            },
                                            |count| ok(count, Vec::new(), Vec::new()),
                                        ),
                                    return err(warnings, errors),
                                    warnings,
                                    errors
                                )
                            }
                            _otherwise => {
                                errors.push(CompileError::Internal(
                                    "Unexpected token for array element count \
                                    while parsing array type.",
                                    span,
                                ));
                                return err(warnings, errors);
                            }
                        }
                    }
                };
                TypeInfo::Array(insert_type(elem_type_info), elem_count)
            }
            Rule::tuple_type => {
                let fields = check!(
                    TypeArgument::parse_arguments_from_pair(input, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                TypeInfo::Tuple(fields)
            }
            _ => {
                errors.push(CompileError::Internal(
                    "Unexpected token while parsing inner type.",
                    span,
                ));
                return err(warnings, errors);
            }
        };
        ok(type_info, warnings, errors)
    }

    pub(crate) fn parse_from_type_param_pair(
        input: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let type_info = match input.as_rule() {
            Rule::type_param => check!(
                Self::pair_as_str_to_type_info(input, config),
                return err(warnings, errors),
                warnings,
                errors
            ),
            _ => {
                let span = Span::from_pest(
                    input.as_span(),
                    config.map(|config| config.dir_of_code.clone()),
                );
                errors.push(CompileError::Internal(
                    "Unexpected token while parsing type.",
                    span,
                ));
                return err(warnings, errors);
            }
        };
        ok(type_info, warnings, errors)
    }

    pub fn pair_as_str_to_type_info(
        input: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Self> {
        let warnings = vec![];
        let errors = vec![];
        let span = Span::from_pest(
            input.as_span(),
            config.map(|config| config.dir_of_code.clone()),
        );
        ok(
            match input.as_str().trim() {
                "u8" => TypeInfo::UnsignedInteger(IntegerBits::Eight),
                "u16" => TypeInfo::UnsignedInteger(IntegerBits::Sixteen),
                "u32" => TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo),
                "u64" => TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
                "bool" => TypeInfo::Boolean,
                "unit" => TypeInfo::Tuple(Vec::new()),
                "byte" => TypeInfo::Byte,
                "b256" => TypeInfo::B256,
                "Self" | "self" => TypeInfo::SelfType,
                "Contract" => TypeInfo::Contract,
                _other => TypeInfo::Custom {
                    name: Ident::new(span),
                    type_arguments: vec![],
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
            UnknownGeneric { name, .. } => name.to_string(),
            Str(x) => format!("str[{}]", x),
            UnsignedInteger(x) => match x {
                IntegerBits::Eight => "u8",
                IntegerBits::Sixteen => "u16",
                IntegerBits::ThirtyTwo => "u32",
                IntegerBits::SixtyFour => "u64",
            }
            .into(),
            Boolean => "bool".into(),
            Custom { name, .. } => format!("unresolved {}", name.as_str()),
            Ref(id) => format!("T{} ({})", id, (*id).friendly_type_str()),
            Tuple(fields) => {
                let field_strs = fields
                    .iter()
                    .map(|field| field.friendly_type_str())
                    .collect::<Vec<String>>();
                format!("({})", field_strs.join(", "))
            }
            SelfType => "Self".into(),
            Byte => "byte".into(),
            B256 => "b256".into(),
            Numeric => "numeric".into(),
            Contract => "contract".into(),
            ErrorRecovery => "unknown due to error".into(),
            Enum {
                name,
                variant_types,
                ..
            } => print_inner_types(
                format!("enum {}", name),
                variant_types.iter().map(|x| x.r#type),
            ),
            Struct { name, fields, .. } => print_inner_types(
                format!("struct {}", name),
                fields.iter().map(|field| field.r#type),
            ),
            ContractCaller { abi_name, .. } => {
                format!("contract caller {}", abi_name)
            }
            Array(elem_ty, count) => format!("[{}; {}]", elem_ty.friendly_type_str(), count),
            Storage { .. } => "contract storage".into(),
        }
    }

    pub(crate) fn json_abi_str(&self) -> String {
        use TypeInfo::*;
        match self {
            Unknown => "unknown".into(),
            UnknownGeneric { name, .. } => name.to_string(),
            Str(x) => format!("str[{}]", x),
            UnsignedInteger(x) => match x {
                IntegerBits::Eight => "u8",
                IntegerBits::Sixteen => "u16",
                IntegerBits::ThirtyTwo => "u32",
                IntegerBits::SixtyFour => "u64",
            }
            .into(),
            Boolean => "bool".into(),
            Custom { name, .. } => format!("unresolved {}", name.as_str()),
            Ref(id) => format!("T{} ({})", id, (*id).json_abi_str()),
            Tuple(fields) => {
                let field_strs = fields
                    .iter()
                    .map(|field| field.json_abi_str())
                    .collect::<Vec<String>>();
                format!("({})", field_strs.join(", "))
            }
            SelfType => "Self".into(),
            Byte => "byte".into(),
            B256 => "b256".into(),
            Numeric => "numeric".into(),
            Contract => "contract".into(),
            ErrorRecovery => "unknown due to error".into(),
            Enum { name, .. } => {
                format!("enum {}", name)
            }
            Struct { name, .. } => {
                format!("struct {}", name)
            }
            ContractCaller { abi_name, .. } => {
                format!("contract caller {}", abi_name)
            }
            Array(elem_ty, count) => format!("[{}; {}]", elem_ty.json_abi_str(), count),
            Storage { .. } => "contract storage".into(),
        }
    }

    /// maps a type to a name that is used when constructing function selectors
    pub(crate) fn to_selector_name(&self, error_msg_span: &Span) -> CompileResult<String> {
        use TypeInfo::*;
        let name = match self {
            Str(len) => format!("str[{}]", len),
            UnsignedInteger(bits) => {
                use IntegerBits::*;
                match bits {
                    Eight => "u8",
                    Sixteen => "u16",
                    ThirtyTwo => "u32",
                    SixtyFour => "u64",
                }
                .into()
            }
            Boolean => "bool".into(),

            Tuple(fields) => {
                let field_names = {
                    let names = fields
                        .iter()
                        .map(|field_type| {
                            resolve_type(field_type.type_id, error_msg_span)
                                .expect("unreachable?")
                                .to_selector_name(error_msg_span)
                        })
                        .collect::<Vec<CompileResult<String>>>();
                    let mut buf = vec![];
                    for name in names {
                        match name.value {
                            Some(value) => buf.push(value),
                            None => return name,
                        }
                    }
                    buf
                };

                format!("({})", field_names.join(","))
            }
            Byte => "byte".into(),
            B256 => "b256".into(),
            Struct { fields, .. } => {
                let names = fields
                    .iter()
                    .map(|field| {
                        resolve_type(field.r#type, error_msg_span)
                            .expect("unreachable?")
                            .to_selector_name(error_msg_span)
                    })
                    .collect::<Vec<CompileResult<String>>>();
                let mut buf = vec![];
                for name in names {
                    match name.value {
                        Some(value) => buf.push(value),
                        None => return name,
                    }
                }
                format!("s({})", buf.join(","))
            }
            Enum { variant_types, .. } => {
                let variant_names = {
                    let names = variant_types
                        .iter()
                        .map(|ty| {
                            let ty = match resolve_type(ty.r#type, error_msg_span) {
                                Err(e) => return err(vec![], vec![e.into()]),
                                Ok(ty) => ty,
                            };
                            ty.to_selector_name(error_msg_span)
                        })
                        .collect::<Vec<CompileResult<String>>>();
                    let mut buf = vec![];
                    for name in names {
                        match name.value {
                            Some(value) => buf.push(value),
                            None => return name,
                        }
                    }
                    buf
                };

                format!("e({})", variant_names.join(","))
            }
            _ => {
                return err(
                    vec![],
                    vec![CompileError::InvalidAbiType {
                        span: error_msg_span.clone(),
                    }],
                )
            }
        };
        ok(name, vec![], vec![])
    }

    pub(crate) fn size_in_bytes(&self, err_span: &Span) -> Result<u64, CompileError> {
        Ok(self.size_in_words(err_span)? * 8)
    }

    /// Calculates the stack size of this type, to be used when allocating stack memory for it.
    pub(crate) fn size_in_words(&self, err_span: &Span) -> Result<u64, CompileError> {
        match self {
            // Each char is a byte, so the size is the num of characters / 8
            // rounded up to the nearest word
            TypeInfo::Str(len) => Ok((len + 7) / 8),
            // Since things are unpacked, all unsigned integers are 64 bits.....for now
            TypeInfo::UnsignedInteger(_) | TypeInfo::Numeric => Ok(1),
            TypeInfo::Boolean => Ok(1),
            TypeInfo::Tuple(fields) => Ok(fields
                .iter()
                .map(|field_type| {
                    resolve_type(field_type.type_id, err_span)
                        .expect("should be unreachable?")
                        .size_in_words(err_span)
                })
                .collect::<Result<Vec<u64>, _>>()?
                .iter()
                .sum()),
            TypeInfo::Byte => Ok(1),
            TypeInfo::B256 => Ok(4),
            TypeInfo::Enum { variant_types, .. } => {
                // the size of an enum is one word (for the tag) plus the maximum size
                // of any individual variant
                Ok(1 + variant_types
                    .iter()
                    .map(|x| -> Result<_, _> { look_up_type_id(x.r#type).size_in_words(err_span) })
                    .collect::<Result<Vec<u64>, _>>()?
                    .into_iter()
                    .max()
                    .unwrap_or(0))
            }
            TypeInfo::Struct { fields, .. } => Ok(fields
                .iter()
                .map(|field| -> Result<_, _> {
                    resolve_type(field.r#type, err_span)
                        .expect("should be unreachable?")
                        .size_in_words(err_span)
                })
                .collect::<Result<Vec<u64>, _>>()?
                .iter()
                .sum()),
            // `ContractCaller` types are unsized and used only in the type system for
            // calling methods
            TypeInfo::ContractCaller { .. } => Ok(0),
            TypeInfo::Contract => unreachable!("contract types are never instantiated"),
            TypeInfo::ErrorRecovery => unreachable!(),
            TypeInfo::Unknown
            | TypeInfo::Custom { .. }
            | TypeInfo::SelfType
            | TypeInfo::UnknownGeneric { .. } => Err(CompileError::UnableToInferGeneric {
                ty: self.friendly_type_str(),
                span: err_span.clone(),
            }),
            TypeInfo::Ref(id) => look_up_type_id(*id).size_in_words(err_span),
            TypeInfo::Array(elem_ty, count) => {
                Ok(look_up_type_id(*elem_ty).size_in_words(err_span)? * *count as u64)
            }
            TypeInfo::Storage { .. } => Ok(0),
        }
    }

    pub(crate) fn is_copy_type(&self, err_span: &Span) -> Result<bool, CompileError> {
        match self {
            // Copy types.
            TypeInfo::UnsignedInteger(_)
            | TypeInfo::Numeric
            | TypeInfo::Boolean
            | TypeInfo::Byte => Ok(true),
            TypeInfo::Tuple(_) if self.is_unit() => Ok(true),

            // Unknown types.
            TypeInfo::Unknown
            | TypeInfo::Custom { .. }
            | TypeInfo::SelfType
            | TypeInfo::UnknownGeneric { .. } => Err(CompileError::UnableToInferGeneric {
                ty: self.friendly_type_str(),
                span: err_span.clone(),
            }),

            // Otherwise default to non-copy.
            _otherwise => Ok(false),
        }
    }

    pub fn is_uninhabited(&self) -> bool {
        match self {
            TypeInfo::Enum { variant_types, .. } => variant_types
                .iter()
                .all(|variant_type| look_up_type_id(variant_type.r#type).is_uninhabited()),
            TypeInfo::Struct { fields, .. } => fields
                .iter()
                .any(|field| look_up_type_id(field.r#type).is_uninhabited()),
            TypeInfo::Tuple(fields) => fields
                .iter()
                .any(|field_type| look_up_type_id(field_type.type_id).is_uninhabited()),
            _ => false,
        }
    }

    pub fn is_zero_sized(&self) -> bool {
        match self {
            TypeInfo::Enum { variant_types, .. } => {
                let mut found_unit_variant = false;
                for variant_type in variant_types {
                    let type_info = look_up_type_id(variant_type.r#type);
                    if type_info.is_uninhabited() {
                        continue;
                    }
                    if type_info.is_zero_sized() && !found_unit_variant {
                        found_unit_variant = true;
                        continue;
                    }
                    return false;
                }
                true
            }
            TypeInfo::Struct { fields, .. } => {
                let mut all_zero_sized = true;
                for field in fields {
                    let type_info = look_up_type_id(field.r#type);
                    if type_info.is_uninhabited() {
                        return true;
                    }
                    if !type_info.is_zero_sized() {
                        all_zero_sized = false;
                    }
                }
                all_zero_sized
            }
            TypeInfo::Tuple(fields) => {
                let mut all_zero_sized = true;
                for field in fields {
                    let field_type = look_up_type_id(field.type_id);
                    if field_type.is_uninhabited() {
                        return true;
                    }
                    if !field_type.is_zero_sized() {
                        all_zero_sized = false;
                    }
                }
                all_zero_sized
            }
            _ => false,
        }
    }

    pub fn is_unit(&self) -> bool {
        match self {
            TypeInfo::Tuple(fields) => fields.is_empty(),
            _ => false,
        }
    }

    pub(crate) fn matches_type_parameter(
        &self,
        mapping: &[(TypeParameter, TypeId)],
    ) -> Option<TypeId> {
        use TypeInfo::*;
        match self {
            TypeInfo::Custom { .. } => {
                for (param, ty_id) in mapping.iter() {
                    let param_type_info = look_up_type_id(param.type_id);
                    if param_type_info == *self {
                        return Some(*ty_id);
                    }
                }
                None
            }
            TypeInfo::UnknownGeneric { name, .. } => {
                for (param, ty_id) in mapping.iter() {
                    if let TypeInfo::Custom {
                        name: other_name, ..
                    } = look_up_type_id(param.type_id)
                    {
                        if name.clone() == other_name {
                            return Some(*ty_id);
                        }
                    }
                }
                None
            }
            TypeInfo::Struct { fields, name } => {
                let mut new_fields = fields.clone();
                for new_field in new_fields.iter_mut() {
                    if let Some(matching_id) =
                        look_up_type_id(new_field.r#type).matches_type_parameter(mapping)
                    {
                        new_field.r#type = insert_type(TypeInfo::Ref(matching_id));
                    }
                }
                Some(insert_type(TypeInfo::Struct {
                    fields: new_fields,
                    name: name.clone(),
                }))
            }
            TypeInfo::Enum {
                variant_types,
                name,
            } => {
                let mut new_variants = variant_types.clone();
                for new_variant in new_variants.iter_mut() {
                    if let Some(matching_id) =
                        look_up_type_id(new_variant.r#type).matches_type_parameter(mapping)
                    {
                        new_variant.r#type = insert_type(TypeInfo::Ref(matching_id));
                    }
                }
                Some(insert_type(TypeInfo::Enum {
                    variant_types: new_variants,
                    name: name.clone(),
                }))
            }
            TypeInfo::Array(ary_ty_id, count) => look_up_type_id(*ary_ty_id)
                .matches_type_parameter(mapping)
                .map(|matching_id| insert_type(TypeInfo::Array(matching_id, *count))),
            TypeInfo::Tuple(fields) => {
                let mut new_fields = Vec::new();
                let mut index = 0;
                while index < fields.len() {
                    let new_field_id_opt =
                        look_up_type_id(fields[index].type_id).matches_type_parameter(mapping);
                    if let Some(new_field_id) = new_field_id_opt {
                        new_fields.extend(fields[..index].iter().cloned());
                        new_fields.push(TypeArgument {
                            type_id: insert_type(TypeInfo::Ref(new_field_id)),
                            span: fields[index].span.clone(),
                        });
                        index += 1;
                        break;
                    }
                    index += 1;
                }
                while index < fields.len() {
                    let new_field = match look_up_type_id(fields[index].type_id)
                        .matches_type_parameter(mapping)
                    {
                        Some(new_field_id) => TypeArgument {
                            type_id: insert_type(TypeInfo::Ref(new_field_id)),
                            span: fields[index].span.clone(),
                        },
                        None => fields[index].clone(),
                    };
                    new_fields.push(new_field);
                    index += 1;
                }
                if new_fields.is_empty() {
                    None
                } else {
                    Some(insert_type(TypeInfo::Tuple(new_fields)))
                }
            }
            Unknown
            | Str(..)
            | UnsignedInteger(..)
            | Boolean
            | Ref(..)
            | ContractCaller { .. }
            | SelfType
            | Byte
            | B256
            | Numeric
            | Contract
            | Storage { .. }
            | ErrorRecovery => None,
        }
    }
}

fn print_inner_types(name: String, inner_types: impl Iterator<Item = TypeId>) -> String {
    format!(
        "{}<{}>",
        name,
        inner_types
            .map(|x| x.friendly_type_str())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn parse_contract_caller_type(
    raw: Pair<Rule>,
    config: Option<&BuildConfig>,
) -> CompileResult<TypeInfo> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let abi_path = match raw.into_inner().next() {
        Some(x) => x,
        None => {
            return ok(
                TypeInfo::ContractCaller {
                    address: Default::default(),
                    abi_name: AbiName::Deferred,
                },
                warnings,
                errors,
            )
        }
    };
    let abi_path = check!(
        CallPath::parse_from_pair(abi_path, config),
        return err(warnings, errors),
        warnings,
        errors
    );

    ok(
        TypeInfo::ContractCaller {
            address: Default::default(),
            abi_name: AbiName::Known(abi_path),
        },
        warnings,
        errors,
    )
}

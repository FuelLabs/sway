use super::*;
use crate::{
    language::{ty, CallPath},
    Ident,
};
use sway_error::error::CompileError;
use sway_types::{integer_bits::IntegerBits, span::Span};

use std::{
    collections::HashSet,
    fmt,
    hash::{Hash, Hasher},
};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum AbiName {
    Deferred,
    Known(CallPath),
}

impl fmt::Display for AbiName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            &(match self {
                AbiName::Deferred => "for unspecified ABI".to_string(),
                AbiName::Known(cp) => cp.to_string(),
            }),
        )
    }
}

/// A slow set primitive using `==` to check for containment.
#[derive(Clone)]
pub struct VecSet<T>(pub Vec<T>);

impl<T: fmt::Debug> fmt::Debug for VecSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> core::ops::Deref for VecSet<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<T: PartialEqWithTypeEngine> VecSet<T> {
    pub fn is_subset(&self, rhs: &Self, type_engine: &TypeEngine) -> bool {
        self.0.len() <= rhs.0.len()
            && self
                .0
                .iter()
                .all(|x| rhs.0.iter().any(|y| x.eq(y, type_engine)))
    }
}

impl<T: PartialEqWithTypeEngine> PartialEqWithTypeEngine for VecSet<T> {
    fn eq(&self, rhs: &Self, type_engine: &TypeEngine) -> bool {
        self.is_subset(rhs, type_engine) && rhs.is_subset(self, type_engine)
    }
}

/// Type information without an associated value, used for type inferencing and definition.
// TODO use idents instead of Strings when we have arena spans
#[derive(Debug, Clone)]
pub enum TypeInfo {
    Unknown,
    UnknownGeneric {
        name: Ident,
        // NOTE(Centril): Used to be BTreeSet; need to revert back later. Must be sorted!
        trait_constraints: VecSet<TraitConstraint>,
    },
    Str(u64),
    UnsignedInteger(IntegerBits),
    Enum {
        name: Ident,
        type_parameters: Vec<TypeParameter>,
        variant_types: Vec<ty::TyEnumVariant>,
    },
    Struct {
        name: Ident,
        type_parameters: Vec<TypeParameter>,
        fields: Vec<ty::TyStructField>,
    },
    Boolean,
    Tuple(Vec<TypeArgument>),
    /// Represents a type which contains methods to issue a contract call.
    /// The specific contract is identified via the `Ident` within.
    ContractCaller {
        abi_name: AbiName,
        // boxed for size
        address: Option<Box<ty::TyExpression>>,
    },
    /// A custom type could be a struct or similar if the name is in scope,
    /// or just a generic parameter if it is not.
    /// At parse time, there is no sense of scope, so this determination is not made
    /// until the semantic analysis stage.
    Custom {
        name: Ident,
        type_arguments: Option<Vec<TypeArgument>>,
    },
    SelfType,
    B256,
    /// This means that specific type of a number is not yet known. It will be
    /// determined via inference at a later time.
    Numeric,
    Contract,
    // used for recovering from errors in the ast
    ErrorRecovery,
    // Static, constant size arrays. The second `TypeId` below contains the initial type ID
    // which could be generic.
    // TODO: change this to a struct instead of a tuple
    Array(TypeId, usize, TypeId),
    /// Represents the entire storage declaration struct
    /// Stored without initializers here, as typed struct fields,
    /// so type checking is able to treat it as a struct with fields.
    Storage {
        fields: Vec<ty::TyStructField>,
    },
    /// Raw untyped pointers.
    /// These are represented in memory as u64 but are a different type since pointers only make
    /// sense in the context they were created in. Users can obtain pointers via standard library
    /// functions such `alloc` or `stack_ptr`. These functions are implemented using asm blocks
    /// which can create pointers by (eg.) reading logically-pointer-valued registers, using the
    /// gtf instruction, or manipulating u64s.
    RawUntypedPtr,
    RawUntypedSlice,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl HashWithTypeEngine for TypeInfo {
    fn hash<H: Hasher>(&self, state: &mut H, type_engine: &TypeEngine) {
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
                fields.hash(state, type_engine);
            }
            TypeInfo::B256 => {
                state.write_u8(6);
            }
            TypeInfo::Enum {
                name,
                variant_types,
                type_parameters,
            } => {
                state.write_u8(7);
                name.hash(state);
                variant_types.hash(state, type_engine);
                type_parameters.hash(state, type_engine);
            }
            TypeInfo::Struct {
                name,
                fields,
                type_parameters,
            } => {
                state.write_u8(8);
                name.hash(state);
                fields.hash(state, type_engine);
                type_parameters.hash(state, type_engine);
            }
            TypeInfo::ContractCaller { abi_name, address } => {
                state.write_u8(9);
                abi_name.hash(state);
                let address = address
                    .as_ref()
                    .map(|x| x.span.as_str().to_string())
                    .unwrap_or_default();
                address.hash(state);
            }
            TypeInfo::Contract => {
                state.write_u8(10);
            }
            TypeInfo::ErrorRecovery => {
                state.write_u8(11);
            }
            TypeInfo::Unknown => {
                state.write_u8(12);
            }
            TypeInfo::SelfType => {
                state.write_u8(13);
            }
            TypeInfo::UnknownGeneric {
                name,
                trait_constraints,
            } => {
                state.write_u8(14);
                name.hash(state);
                trait_constraints.hash(state, type_engine);
            }
            TypeInfo::Custom {
                name,
                type_arguments,
            } => {
                state.write_u8(15);
                name.hash(state);
                type_arguments.as_deref().hash(state, type_engine);
            }
            TypeInfo::Storage { fields } => {
                state.write_u8(16);
                fields.hash(state, type_engine);
            }
            TypeInfo::Array(elem_ty, count, _) => {
                state.write_u8(17);
                type_engine
                    .look_up_type_id(*elem_ty)
                    .hash(state, type_engine);
                count.hash(state);
            }
            TypeInfo::RawUntypedPtr => {
                state.write_u8(18);
            }
            TypeInfo::RawUntypedSlice => {
                state.write_u8(19);
            }
        }
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl EqWithTypeEngine for TypeInfo {}
impl PartialEqWithTypeEngine for TypeInfo {
    fn eq(&self, other: &Self, type_engine: &TypeEngine) -> bool {
        match (self, other) {
            (Self::Unknown, Self::Unknown)
            | (Self::Boolean, Self::Boolean)
            | (Self::SelfType, Self::SelfType)
            | (Self::B256, Self::B256)
            | (Self::Numeric, Self::Numeric)
            | (Self::Contract, Self::Contract)
            | (Self::ErrorRecovery, Self::ErrorRecovery) => true,
            (
                Self::UnknownGeneric {
                    name: l,
                    trait_constraints: ltc,
                },
                Self::UnknownGeneric {
                    name: r,
                    trait_constraints: rtc,
                },
            ) => l == r && ltc.eq(&rtc, type_engine),
            (
                Self::Custom {
                    name: l_name,
                    type_arguments: l_type_args,
                },
                Self::Custom {
                    name: r_name,
                    type_arguments: r_type_args,
                },
            ) => {
                l_name == r_name
                    && l_type_args
                        .as_deref()
                        .eq(&r_type_args.as_deref(), type_engine)
            }
            (Self::Str(l), Self::Str(r)) => l == r,
            (Self::UnsignedInteger(l), Self::UnsignedInteger(r)) => l == r,
            (
                Self::Enum {
                    name: l_name,
                    variant_types: l_variant_types,
                    type_parameters: l_type_parameters,
                },
                Self::Enum {
                    name: r_name,
                    variant_types: r_variant_types,
                    type_parameters: r_type_parameters,
                },
            ) => {
                l_name == r_name
                    && l_variant_types.eq(r_variant_types, type_engine)
                    && l_type_parameters.eq(r_type_parameters, type_engine)
            }
            (
                Self::Struct {
                    name: l_name,
                    fields: l_fields,
                    type_parameters: l_type_parameters,
                },
                Self::Struct {
                    name: r_name,
                    fields: r_fields,
                    type_parameters: r_type_parameters,
                },
            ) => {
                l_name == r_name
                    && l_fields.eq(r_fields, type_engine)
                    && l_type_parameters.eq(r_type_parameters, type_engine)
            }
            (Self::Tuple(l), Self::Tuple(r)) => l
                .iter()
                .zip(r.iter())
                .map(|(l, r)| {
                    type_engine
                        .look_up_type_id(l.type_id)
                        .eq(&type_engine.look_up_type_id(r.type_id), type_engine)
                })
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
            ) => {
                l_abi_name == r_abi_name
                    && l_address.as_deref().eq(&r_address.as_deref(), type_engine)
            }
            (Self::Array(l0, l1, _), Self::Array(r0, r1, _)) => {
                type_engine
                    .look_up_type_id(*l0)
                    .eq(&type_engine.look_up_type_id(*r0), type_engine)
                    && l1 == r1
            }
            (TypeInfo::Storage { fields: l_fields }, TypeInfo::Storage { fields: r_fields }) => {
                l_fields.eq(r_fields, type_engine)
            }
            (TypeInfo::RawUntypedPtr, TypeInfo::RawUntypedPtr) => true,
            (TypeInfo::RawUntypedSlice, TypeInfo::RawUntypedSlice) => true,
            _ => false,
        }
    }
}

impl Default for TypeInfo {
    fn default() -> Self {
        TypeInfo::Unknown
    }
}

impl DisplayWithTypeEngine for TypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, type_engine: &TypeEngine) -> fmt::Result {
        use TypeInfo::*;
        let s = match self {
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
            Tuple(fields) => {
                let field_strs = fields
                    .iter()
                    .map(|field| type_engine.help_out(field).to_string())
                    .collect::<Vec<String>>();
                format!("({})", field_strs.join(", "))
            }
            SelfType => "Self".into(),
            B256 => "b256".into(),
            Numeric => "numeric".into(),
            Contract => "contract".into(),
            ErrorRecovery => "unknown due to error".into(),
            Enum {
                name,
                type_parameters,
                ..
            } => print_inner_types(
                name.as_str().to_string(),
                type_parameters.iter().map(|x| x.type_id),
            ),
            Struct {
                name,
                type_parameters,
                ..
            } => print_inner_types(
                name.as_str().to_string(),
                type_parameters.iter().map(|x| x.type_id),
            ),
            ContractCaller { abi_name, address } => {
                format!(
                    "contract caller {} ( {} )",
                    abi_name,
                    address
                        .as_ref()
                        .map(|address| address.span.as_str().to_string())
                        .unwrap_or_else(|| "None".into())
                )
            }
            Array(elem_ty, count, _) => format!("[{}; {}]", type_engine.help_out(elem_ty), count),
            Storage { .. } => "contract storage".into(),
            RawUntypedPtr => "raw untyped ptr".into(),
            RawUntypedSlice => "raw untyped slice".into(),
        };
        write!(f, "{}", s)
    }
}

impl UnconstrainedTypeParameters for TypeInfo {
    fn type_parameter_is_unconstrained(
        &self,
        type_engine: &TypeEngine,
        type_parameter: &TypeParameter,
    ) -> bool {
        let type_parameter_info = type_engine.look_up_type_id(type_parameter.type_id);
        match self {
            TypeInfo::UnknownGeneric {
                trait_constraints, ..
            } => {
                self.eq(&type_parameter_info, type_engine)
                    || trait_constraints
                        .iter()
                        .flat_map(|trait_constraint| {
                            trait_constraint.type_arguments.iter().map(|type_arg| {
                                type_arg
                                    .type_id
                                    .type_parameter_is_unconstrained(type_engine, type_parameter)
                            })
                        })
                        .any(|x| x)
            }
            TypeInfo::Enum {
                type_parameters,
                variant_types,
                ..
            } => {
                let unconstrained_in_type_parameters = type_parameters
                    .iter()
                    .map(|type_param| {
                        type_param
                            .type_id
                            .type_parameter_is_unconstrained(type_engine, type_parameter)
                    })
                    .any(|x| x);
                let unconstrained_in_variants = variant_types
                    .iter()
                    .map(|variant| {
                        variant
                            .type_id
                            .type_parameter_is_unconstrained(type_engine, type_parameter)
                    })
                    .any(|x| x);
                unconstrained_in_type_parameters || unconstrained_in_variants
            }
            TypeInfo::Struct {
                type_parameters,
                fields,
                ..
            } => {
                let unconstrained_in_type_parameters = type_parameters
                    .iter()
                    .map(|type_param| {
                        type_param
                            .type_id
                            .type_parameter_is_unconstrained(type_engine, type_parameter)
                    })
                    .any(|x| x);
                let unconstrained_in_fields = fields
                    .iter()
                    .map(|field| {
                        field
                            .type_id
                            .type_parameter_is_unconstrained(type_engine, type_parameter)
                    })
                    .any(|x| x);
                unconstrained_in_type_parameters || unconstrained_in_fields
            }
            TypeInfo::Tuple(elems) => elems
                .iter()
                .map(|elem| {
                    elem.type_id
                        .type_parameter_is_unconstrained(type_engine, type_parameter)
                })
                .any(|x| x),
            TypeInfo::Custom { type_arguments, .. } => type_arguments
                .clone()
                .unwrap_or_default()
                .iter()
                .map(|type_arg| {
                    type_arg
                        .type_id
                        .type_parameter_is_unconstrained(type_engine, type_parameter)
                })
                .any(|x| x),
            TypeInfo::Array(elem, _, _) => {
                elem.type_parameter_is_unconstrained(type_engine, type_parameter)
            }
            TypeInfo::Unknown
            | TypeInfo::Str(_)
            | TypeInfo::UnsignedInteger(_)
            | TypeInfo::Boolean
            | TypeInfo::ContractCaller { .. }
            | TypeInfo::SelfType
            | TypeInfo::B256
            | TypeInfo::Numeric
            | TypeInfo::Contract
            | TypeInfo::ErrorRecovery
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice
            | TypeInfo::Storage { .. } => false,
        }
    }
}

impl TypeInfo {
    pub fn json_abi_str(&self, type_engine: &TypeEngine) -> String {
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
            Custom { name, .. } => name.to_string(),
            Tuple(fields) => {
                let field_strs = fields
                    .iter()
                    .map(|field| field.json_abi_str(type_engine))
                    .collect::<Vec<String>>();
                format!("({})", field_strs.join(", "))
            }
            SelfType => "Self".into(),
            B256 => "b256".into(),
            Numeric => "u64".into(), // u64 is the default
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
            Array(elem_ty, count, _) => {
                format!("[{}; {}]", elem_ty.json_abi_str(type_engine), count)
            }
            Storage { .. } => "contract storage".into(),
            RawUntypedPtr => "raw untyped ptr".into(),
            RawUntypedSlice => "raw untyped slice".into(),
        }
    }

    /// maps a type to a name that is used when constructing function selectors
    pub(crate) fn to_selector_name(
        &self,
        type_engine: &TypeEngine,
        error_msg_span: &Span,
    ) -> CompileResult<String> {
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
                            type_engine
                                .to_typeinfo(field_type.type_id, error_msg_span)
                                .expect("unreachable?")
                                .to_selector_name(type_engine, error_msg_span)
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
            B256 => "b256".into(),
            Struct {
                fields,
                type_parameters,
                ..
            } => {
                let field_names = {
                    let names = fields
                        .iter()
                        .map(|ty| {
                            let ty = match type_engine.to_typeinfo(ty.type_id, error_msg_span) {
                                Err(e) => return err(vec![], vec![e.into()]),
                                Ok(ty) => ty,
                            };
                            ty.to_selector_name(type_engine, error_msg_span)
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

                let type_arguments = {
                    let type_arguments = type_parameters
                        .iter()
                        .map(|ty| {
                            let ty = match type_engine.to_typeinfo(ty.type_id, error_msg_span) {
                                Err(e) => return err(vec![], vec![e.into()]),
                                Ok(ty) => ty,
                            };
                            ty.to_selector_name(type_engine, error_msg_span)
                        })
                        .collect::<Vec<CompileResult<String>>>();
                    let mut buf = vec![];
                    for arg in type_arguments {
                        match arg.value {
                            Some(value) => buf.push(value),
                            None => return arg,
                        }
                    }
                    buf
                };

                if type_arguments.is_empty() {
                    format!("s({})", field_names.join(","))
                } else {
                    format!("s<{}>({})", type_arguments.join(","), field_names.join(","))
                }
            }
            Enum {
                variant_types,
                type_parameters,
                ..
            } => {
                let variant_names = {
                    let names = variant_types
                        .iter()
                        .map(|ty| {
                            let ty = match type_engine.to_typeinfo(ty.type_id, error_msg_span) {
                                Err(e) => return err(vec![], vec![e.into()]),
                                Ok(ty) => ty,
                            };
                            ty.to_selector_name(type_engine, error_msg_span)
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

                let type_arguments = {
                    let type_arguments = type_parameters
                        .iter()
                        .map(|ty| {
                            let ty = match type_engine.to_typeinfo(ty.type_id, error_msg_span) {
                                Err(e) => return err(vec![], vec![e.into()]),
                                Ok(ty) => ty,
                            };
                            ty.to_selector_name(type_engine, error_msg_span)
                        })
                        .collect::<Vec<CompileResult<String>>>();
                    let mut buf = vec![];
                    for arg in type_arguments {
                        match arg.value {
                            Some(value) => buf.push(value),
                            None => return arg,
                        }
                    }
                    buf
                };
                if type_arguments.is_empty() {
                    format!("e({})", variant_names.join(","))
                } else {
                    format!(
                        "e<{}>({})",
                        type_arguments.join(","),
                        variant_names.join(",")
                    )
                }
            }
            Array(type_id, size, _) => {
                let name = type_engine
                    .look_up_type_id(*type_id)
                    .to_selector_name(type_engine, error_msg_span);
                let name = match name.value {
                    Some(name) => name,
                    None => return name,
                };
                format!("a[{};{}]", name, size)
            }
            RawUntypedPtr => "rawptr".to_string(),
            RawUntypedSlice => "rawslice".to_string(),
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

    pub fn is_uninhabited(&self, type_engine: &TypeEngine) -> bool {
        let id_uninhabited = |id| type_engine.look_up_type_id(id).is_uninhabited(type_engine);

        match self {
            TypeInfo::Enum { variant_types, .. } => variant_types
                .iter()
                .all(|variant_type| id_uninhabited(variant_type.type_id)),
            TypeInfo::Struct { fields, .. } => {
                fields.iter().any(|field| id_uninhabited(field.type_id))
            }
            TypeInfo::Tuple(fields) => fields
                .iter()
                .any(|field_type| id_uninhabited(field_type.type_id)),
            TypeInfo::Array(type_id, size, _) => *size > 0 && id_uninhabited(*type_id),
            _ => false,
        }
    }

    pub fn is_zero_sized(&self, type_engine: &TypeEngine) -> bool {
        match self {
            TypeInfo::Enum { variant_types, .. } => {
                let mut found_unit_variant = false;
                for variant_type in variant_types {
                    let type_info = type_engine.look_up_type_id(variant_type.type_id);
                    if type_info.is_uninhabited(type_engine) {
                        continue;
                    }
                    if type_info.is_zero_sized(type_engine) && !found_unit_variant {
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
                    let type_info = type_engine.look_up_type_id(field.type_id);
                    if type_info.is_uninhabited(type_engine) {
                        return true;
                    }
                    if !type_info.is_zero_sized(type_engine) {
                        all_zero_sized = false;
                    }
                }
                all_zero_sized
            }
            TypeInfo::Tuple(fields) => {
                let mut all_zero_sized = true;
                for field in fields {
                    let field_type = type_engine.look_up_type_id(field.type_id);
                    if field_type.is_uninhabited(type_engine) {
                        return true;
                    }
                    if !field_type.is_zero_sized(type_engine) {
                        all_zero_sized = false;
                    }
                }
                all_zero_sized
            }
            TypeInfo::Array(type_id, size, _) => {
                *size == 0
                    || type_engine
                        .look_up_type_id(*type_id)
                        .is_zero_sized(type_engine)
            }
            _ => false,
        }
    }

    pub fn can_safely_ignore(&self, type_engine: &TypeEngine) -> bool {
        if self.is_zero_sized(type_engine) {
            return true;
        }
        match self {
            TypeInfo::Tuple(fields) => fields.iter().all(|type_argument| {
                type_engine
                    .look_up_type_id(type_argument.type_id)
                    .can_safely_ignore(type_engine)
            }),
            TypeInfo::Array(type_id, size, _) => {
                *size == 0
                    || type_engine
                        .look_up_type_id(*type_id)
                        .can_safely_ignore(type_engine)
            }
            TypeInfo::ErrorRecovery => true,
            TypeInfo::Unknown => true,
            _ => false,
        }
    }

    pub fn is_unit(&self) -> bool {
        match self {
            TypeInfo::Tuple(fields) => fields.is_empty(),
            _ => false,
        }
    }

    pub fn is_copy_type(&self) -> bool {
        matches!(self, TypeInfo::Boolean | TypeInfo::UnsignedInteger(_)) || self.is_unit()
    }

    pub(crate) fn apply_type_arguments(
        self,
        type_arguments: Vec<TypeArgument>,
        span: &Span,
    ) -> CompileResult<TypeInfo> {
        let warnings = vec![];
        let mut errors = vec![];
        if type_arguments.is_empty() {
            return ok(self, warnings, errors);
        }
        match self {
            TypeInfo::Enum { .. } | TypeInfo::Struct { .. } => {
                errors.push(CompileError::Internal(
                    "did not expect to apply type arguments to this type",
                    span.clone(),
                ));
                err(warnings, errors)
            }
            TypeInfo::Custom {
                name,
                type_arguments: other_type_arguments,
            } => {
                if other_type_arguments.is_some() {
                    errors.push(CompileError::TypeArgumentsNotAllowed { span: span.clone() });
                    err(warnings, errors)
                } else {
                    let type_info = TypeInfo::Custom {
                        name,
                        type_arguments: Some(type_arguments),
                    };
                    ok(type_info, warnings, errors)
                }
            }
            TypeInfo::Unknown
            | TypeInfo::UnknownGeneric { .. }
            | TypeInfo::Str(_)
            | TypeInfo::UnsignedInteger(_)
            | TypeInfo::Boolean
            | TypeInfo::Tuple(_)
            | TypeInfo::ContractCaller { .. }
            | TypeInfo::SelfType
            | TypeInfo::B256
            | TypeInfo::Numeric
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice
            | TypeInfo::Contract
            | TypeInfo::ErrorRecovery
            | TypeInfo::Array(_, _, _)
            | TypeInfo::Storage { .. } => {
                errors.push(CompileError::TypeArgumentsNotAllowed { span: span.clone() });
                err(warnings, errors)
            }
        }
    }

    /// Given a `TypeInfo` `self`, analyze `self` and return all inner
    /// `TypeId`'s of `self`, not including `self`.
    pub(crate) fn extract_inner_types(&self, type_engine: &TypeEngine) -> HashSet<TypeId> {
        let helper = |type_id: TypeId| {
            let mut inner_types = HashSet::new();
            match type_engine.look_up_type_id(type_id) {
                TypeInfo::Enum {
                    type_parameters,
                    variant_types,
                    ..
                } => {
                    inner_types.insert(type_id);
                    for type_param in type_parameters.iter() {
                        inner_types.extend(
                            type_engine
                                .look_up_type_id(type_param.type_id)
                                .extract_inner_types(type_engine),
                        );
                    }
                    for variant in variant_types.iter() {
                        inner_types.extend(
                            type_engine
                                .look_up_type_id(variant.type_id)
                                .extract_inner_types(type_engine),
                        );
                    }
                }
                TypeInfo::Struct {
                    type_parameters,
                    fields,
                    ..
                } => {
                    inner_types.insert(type_id);
                    for type_param in type_parameters.iter() {
                        inner_types.extend(
                            type_engine
                                .look_up_type_id(type_param.type_id)
                                .extract_inner_types(type_engine),
                        );
                    }
                    for field in fields.iter() {
                        inner_types.extend(
                            type_engine
                                .look_up_type_id(field.type_id)
                                .extract_inner_types(type_engine),
                        );
                    }
                }
                TypeInfo::Custom { type_arguments, .. } => {
                    inner_types.insert(type_id);
                    if let Some(type_arguments) = type_arguments {
                        for type_arg in type_arguments.iter() {
                            inner_types.extend(
                                type_engine
                                    .look_up_type_id(type_arg.type_id)
                                    .extract_inner_types(type_engine),
                            );
                        }
                    }
                }
                TypeInfo::Array(type_id, _, _) => {
                    inner_types.insert(type_id);
                    inner_types.extend(
                        type_engine
                            .look_up_type_id(type_id)
                            .extract_inner_types(type_engine),
                    );
                }
                TypeInfo::Tuple(elems) => {
                    inner_types.insert(type_id);
                    for elem in elems.iter() {
                        inner_types.extend(
                            type_engine
                                .look_up_type_id(elem.type_id)
                                .extract_inner_types(type_engine),
                        );
                    }
                }
                TypeInfo::Storage { fields } => {
                    inner_types.insert(type_id);
                    for field in fields.iter() {
                        inner_types.extend(
                            type_engine
                                .look_up_type_id(field.type_id)
                                .extract_inner_types(type_engine),
                        );
                    }
                }
                TypeInfo::Unknown
                | TypeInfo::UnknownGeneric { .. }
                | TypeInfo::Str(_)
                | TypeInfo::UnsignedInteger(_)
                | TypeInfo::Boolean
                | TypeInfo::ContractCaller { .. }
                | TypeInfo::SelfType
                | TypeInfo::B256
                | TypeInfo::Numeric
                | TypeInfo::RawUntypedPtr
                | TypeInfo::RawUntypedSlice
                | TypeInfo::Contract => {
                    inner_types.insert(type_id);
                }
                TypeInfo::ErrorRecovery => {}
            }
            inner_types
        };

        let mut inner_types = HashSet::new();
        match self {
            TypeInfo::Enum {
                type_parameters,
                variant_types,
                ..
            } => {
                for type_param in type_parameters.iter() {
                    inner_types.extend(helper(type_param.type_id));
                }
                for variant in variant_types.iter() {
                    inner_types.extend(helper(variant.type_id));
                }
            }
            TypeInfo::Struct {
                type_parameters,
                fields,
                ..
            } => {
                for type_param in type_parameters.iter() {
                    inner_types.extend(helper(type_param.type_id));
                }
                for field in fields.iter() {
                    inner_types.extend(helper(field.type_id));
                }
            }
            TypeInfo::Custom { type_arguments, .. } => {
                if let Some(type_arguments) = type_arguments {
                    for type_arg in type_arguments.iter() {
                        inner_types.extend(helper(type_arg.type_id));
                    }
                }
            }
            TypeInfo::Array(type_id, _, _) => {
                inner_types.extend(helper(*type_id));
            }
            TypeInfo::Tuple(elems) => {
                for elem in elems.iter() {
                    inner_types.extend(helper(elem.type_id));
                }
            }
            TypeInfo::Storage { fields } => {
                for field in fields.iter() {
                    inner_types.extend(helper(field.type_id));
                }
            }
            TypeInfo::Unknown
            | TypeInfo::UnknownGeneric { .. }
            | TypeInfo::Str(_)
            | TypeInfo::UnsignedInteger(_)
            | TypeInfo::Boolean
            | TypeInfo::ContractCaller { .. }
            | TypeInfo::SelfType
            | TypeInfo::B256
            | TypeInfo::Numeric
            | TypeInfo::Contract
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice
            | TypeInfo::ErrorRecovery => {}
        }
        inner_types
    }

    /// Given a `TypeInfo` `self`, check to see if `self` is currently
    /// supported in match expressions, and return an error if it is not.
    pub(crate) fn expect_is_supported_in_match_expressions(
        &self,
        span: &Span,
    ) -> CompileResult<()> {
        let warnings = vec![];
        let mut errors = vec![];
        match self {
            TypeInfo::UnsignedInteger(_)
            | TypeInfo::Enum { .. }
            | TypeInfo::Struct { .. }
            | TypeInfo::Boolean
            | TypeInfo::Tuple(_)
            | TypeInfo::B256
            | TypeInfo::UnknownGeneric { .. }
            | TypeInfo::Numeric => ok((), warnings, errors),
            TypeInfo::Unknown
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice
            | TypeInfo::ContractCaller { .. }
            | TypeInfo::Custom { .. }
            | TypeInfo::SelfType
            | TypeInfo::Str(_)
            | TypeInfo::Contract
            | TypeInfo::ErrorRecovery
            | TypeInfo::Array(_, _, _)
            | TypeInfo::Storage { .. } => {
                errors.push(CompileError::Unimplemented(
                    "matching on this type is unsupported right now",
                    span.clone(),
                ));
                err(warnings, errors)
            }
        }
    }

    /// Given a `TypeInfo` `self`, check to see if `self` is currently
    /// supported in `impl` blocks in the "type implementing for" position.
    pub(crate) fn expect_is_supported_in_impl_blocks_self(&self, span: &Span) -> CompileResult<()> {
        let warnings = vec![];
        let mut errors = vec![];
        match self {
            TypeInfo::UnsignedInteger(_)
            | TypeInfo::Enum { .. }
            | TypeInfo::Struct { .. }
            | TypeInfo::Boolean
            | TypeInfo::Tuple(_)
            | TypeInfo::B256
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice
            | TypeInfo::Custom { .. }
            | TypeInfo::Str(_)
            | TypeInfo::Array(_, _, _)
            | TypeInfo::Contract
            | TypeInfo::Numeric => ok((), warnings, errors),
            TypeInfo::Unknown
            | TypeInfo::UnknownGeneric { .. }
            | TypeInfo::ContractCaller { .. }
            | TypeInfo::SelfType
            | TypeInfo::ErrorRecovery
            | TypeInfo::Storage { .. } => {
                errors.push(CompileError::Unimplemented(
                    "implementing traits on this type is unsupported right now",
                    span.clone(),
                ));
                err(warnings, errors)
            }
        }
    }

    /// Given a `TypeInfo` `self`, analyze `self` and return all nested
    /// `TypeInfo`'s found in `self`, including `self`.
    pub(crate) fn extract_nested_types(
        self,
        type_engine: &TypeEngine,
        span: &Span,
    ) -> CompileResult<Vec<TypeInfo>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut all_nested_types = vec![self.clone()];
        match self {
            TypeInfo::Enum {
                variant_types,
                type_parameters,
                ..
            } => {
                for type_parameter in type_parameters.iter() {
                    let mut nested_types = check!(
                        type_engine
                            .look_up_type_id(type_parameter.type_id)
                            .extract_nested_types(type_engine, span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    all_nested_types.append(&mut nested_types);
                }
                for variant_type in variant_types.iter() {
                    let mut nested_types = check!(
                        type_engine
                            .look_up_type_id(variant_type.type_id)
                            .extract_nested_types(type_engine, span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    all_nested_types.append(&mut nested_types);
                }
            }
            TypeInfo::Struct {
                fields,
                type_parameters,
                ..
            } => {
                for type_parameter in type_parameters.iter() {
                    let mut nested_types = check!(
                        type_engine
                            .look_up_type_id(type_parameter.type_id)
                            .extract_nested_types(type_engine, span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    all_nested_types.append(&mut nested_types);
                }
                for field in fields.iter() {
                    let mut nested_types = check!(
                        type_engine
                            .look_up_type_id(field.type_id)
                            .extract_nested_types(type_engine, span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    all_nested_types.append(&mut nested_types);
                }
            }
            TypeInfo::Tuple(type_arguments) => {
                for type_argument in type_arguments.iter() {
                    let mut nested_types = check!(
                        type_engine
                            .look_up_type_id(type_argument.type_id)
                            .extract_nested_types(type_engine, span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    all_nested_types.append(&mut nested_types);
                }
            }
            TypeInfo::Array(type_id, _, _) => {
                let mut nested_types = check!(
                    type_engine
                        .look_up_type_id(type_id)
                        .extract_nested_types(type_engine, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                all_nested_types.append(&mut nested_types);
            }
            TypeInfo::Storage { fields } => {
                for field in fields.iter() {
                    let mut nested_types = check!(
                        type_engine
                            .look_up_type_id(field.type_id)
                            .extract_nested_types(type_engine, span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    all_nested_types.append(&mut nested_types);
                }
            }
            TypeInfo::UnknownGeneric {
                trait_constraints, ..
            } => {
                for trait_constraint in trait_constraints.iter() {
                    for type_arg in trait_constraint.type_arguments.iter() {
                        let mut nested_types = check!(
                            type_engine
                                .look_up_type_id(type_arg.type_id)
                                .extract_nested_types(type_engine, span),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );
                        all_nested_types.append(&mut nested_types);
                    }
                }
            }
            TypeInfo::Unknown
            | TypeInfo::Str(_)
            | TypeInfo::UnsignedInteger(_)
            | TypeInfo::Boolean
            | TypeInfo::ContractCaller { .. }
            | TypeInfo::B256
            | TypeInfo::Numeric
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice
            | TypeInfo::Contract
            | TypeInfo::ErrorRecovery => {}
            TypeInfo::Custom { .. } | TypeInfo::SelfType => {
                errors.push(CompileError::Internal(
                    "did not expect to find this type here",
                    span.clone(),
                ));
                return err(warnings, errors);
            }
        }
        ok(all_nested_types, warnings, errors)
    }

    pub(crate) fn extract_nested_generics<'a>(
        &self,
        type_engine: &'a TypeEngine,
        span: &Span,
    ) -> CompileResult<HashSet<WithTypeEngine<'a, TypeInfo>>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let nested_types = check!(
            self.clone().extract_nested_types(type_engine, span),
            return err(warnings, errors),
            warnings,
            errors
        );
        let generics = HashSet::from_iter(
            nested_types
                .into_iter()
                .filter(|x| matches!(x, TypeInfo::UnknownGeneric { .. }))
                .map(|thing| WithTypeEngine {
                    thing,
                    engine: type_engine,
                }),
        );
        ok(generics, warnings, errors)
    }

    /// Given two `TypeInfo`'s `self` and `other`, check to see if `self` is
    /// unidirectionally a subset of `other`.
    ///
    /// `self` is a subset of `other` if it can be generalized over `other`.
    /// For example, the generic `T` is a subset of the generic `F` because
    /// anything of the type `T` could also be of the type `F` (minus any
    /// external context that may make this statement untrue).
    ///
    /// Given:
    ///
    /// ```ignore
    /// struct Data<T, F> {
    ///   x: T,
    ///   y: F,
    /// }
    /// ```
    ///
    /// the type `Data<T, F>` is a subset of any generic type.
    ///
    /// Given:
    ///
    /// ```ignore
    /// struct Data<T, F> {
    ///   x: T,
    ///   y: F,
    /// }
    ///
    /// impl<T> Data<T, T> { }
    /// ```
    ///
    /// the type `Data<T, T>` is a subset of `Data<T, F>`, but _`Data<T, F>` is
    /// not a subset of `Data<T, T>`_.
    ///
    /// Given:
    ///
    /// ```ignore
    /// struct Data<T, F> {
    ///   x: T,
    ///   y: F,
    /// }
    ///
    /// impl<T> Data<T, T> { }
    ///
    /// fn dummy() {
    ///     // the type of foo is Data<bool, u64>
    ///     let foo = Data {
    ///         x: true,
    ///         y: 1u64
    ///     };
    ///     // the type of bar is Data<u8, u8>
    ///     let bar = Data {
    ///         x: 0u8,
    ///         y: 0u8
    ///     };
    /// }
    /// ```
    ///
    /// then:
    ///
    /// | type:             | is subset of:                                | is not a subset of: |
    /// |-------------------|----------------------------------------------|---------------------|
    /// | `Data<T, T>`      | `Data<T, F>`, any generic type               |                     |
    /// | `Data<T, F>`      | any generic type                             | `Data<T, T>`        |
    /// | `Data<bool, u64>` | `Data<T, F>`, any generic type               | `Data<T, T>`        |
    /// | `Data<u8, u8>`    | `Data<T, T>`, `Data<T, F>`, any generic type |                     |
    ///
    /// For generic types with trait constraints, the generic type `self` is a
    /// subset of the generic type `other` when the trait constraints of
    /// `other` are a subset of the trait constraints of `self`. This is a bit
    /// unintuitive, but you can think of it this way---a generic type `self`
    /// can be generalized over `other` when `other` has no methods
    /// that `self` doesn't have. These methods are coming from the trait
    /// constraints---if the trait constraints of `other` are a subset of the
    /// trait constraints of `self`, then we know that `other` has unique
    /// methods.
    pub(crate) fn is_subset_of(&self, other: &TypeInfo, type_engine: &TypeEngine) -> bool {
        // handle the generics cases
        match (self, other) {
            (
                Self::UnknownGeneric {
                    trait_constraints: ltc,
                    ..
                },
                Self::UnknownGeneric {
                    trait_constraints: rtc,
                    ..
                },
            ) => {
                return rtc.is_subset(ltc, type_engine);
            }
            // any type is the subset of a generic
            (_, Self::UnknownGeneric { .. }) => {
                return true;
            }
            _ => {}
        }

        self.is_subset_inner(other, type_engine)
    }

    /// Given two `TypeInfo`'s `self` and `other`, checks to see if `self` is
    /// unidirectionally a subset of `other`, excluding consideration of generic
    /// types (like in the `is_subset_of` method).
    pub(crate) fn is_subset_of_for_item_import(
        &self,
        other: &TypeInfo,
        type_engine: &TypeEngine,
    ) -> bool {
        self.is_subset_inner(other, type_engine)
    }

    fn is_subset_inner(&self, other: &TypeInfo, type_engine: &TypeEngine) -> bool {
        match (self, other) {
            (Self::Array(l0, l1, _), Self::Array(r0, r1, _)) => {
                type_engine
                    .look_up_type_id(*l0)
                    .is_subset_of(&type_engine.look_up_type_id(*r0), type_engine)
                    && l1 == r1
            }
            (
                Self::Custom {
                    name: l_name,
                    type_arguments: l_type_args,
                },
                Self::Custom {
                    name: r_name,
                    type_arguments: r_type_args,
                },
            ) => {
                let l_types = l_type_args
                    .as_ref()
                    .unwrap_or(&vec![])
                    .iter()
                    .map(|x| type_engine.look_up_type_id(x.type_id))
                    .collect::<Vec<_>>();
                let r_types = r_type_args
                    .as_ref()
                    .unwrap_or(&vec![])
                    .iter()
                    .map(|x| type_engine.look_up_type_id(x.type_id))
                    .collect::<Vec<_>>();
                l_name == r_name && types_are_subset_of(type_engine, &l_types, &r_types)
            }
            (
                Self::Enum {
                    name: l_name,
                    variant_types: l_variant_types,
                    type_parameters: l_type_parameters,
                },
                Self::Enum {
                    name: r_name,
                    variant_types: r_variant_types,
                    type_parameters: r_type_parameters,
                },
            ) => {
                let l_names = l_variant_types
                    .iter()
                    .map(|x| x.name.clone())
                    .collect::<Vec<_>>();
                let r_names = r_variant_types
                    .iter()
                    .map(|x| x.name.clone())
                    .collect::<Vec<_>>();
                let l_types = l_type_parameters
                    .iter()
                    .map(|x| type_engine.look_up_type_id(x.type_id))
                    .collect::<Vec<_>>();
                let r_types = r_type_parameters
                    .iter()
                    .map(|x| type_engine.look_up_type_id(x.type_id))
                    .collect::<Vec<_>>();
                l_name == r_name
                    && l_names == r_names
                    && types_are_subset_of(type_engine, &l_types, &r_types)
            }
            (
                Self::Struct {
                    name: l_name,
                    fields: l_fields,
                    type_parameters: l_type_parameters,
                },
                Self::Struct {
                    name: r_name,
                    fields: r_fields,
                    type_parameters: r_type_parameters,
                },
            ) => {
                let l_names = l_fields.iter().map(|x| x.name.clone()).collect::<Vec<_>>();
                let r_names = r_fields.iter().map(|x| x.name.clone()).collect::<Vec<_>>();
                let l_types = l_type_parameters
                    .iter()
                    .map(|x| type_engine.look_up_type_id(x.type_id))
                    .collect::<Vec<_>>();
                let r_types = r_type_parameters
                    .iter()
                    .map(|x| type_engine.look_up_type_id(x.type_id))
                    .collect::<Vec<_>>();
                l_name == r_name
                    && l_names == r_names
                    && types_are_subset_of(type_engine, &l_types, &r_types)
            }
            (Self::Tuple(l_types), Self::Tuple(r_types)) => {
                let l_types = l_types
                    .iter()
                    .map(|x| type_engine.look_up_type_id(x.type_id))
                    .collect::<Vec<_>>();
                let r_types = r_types
                    .iter()
                    .map(|x| type_engine.look_up_type_id(x.type_id))
                    .collect::<Vec<_>>();
                types_are_subset_of(type_engine, &l_types, &r_types)
            }
            (a, b) => a.eq(b, type_engine),
        }
    }

    /// Given a `TypeInfo` `self` and a list of `Ident`'s `subfields`,
    /// iterate through the elements of `subfields` as `subfield`,
    /// and recursively apply `subfield` to `self`.
    ///
    /// Returns a [ty::TyStructField] when all `subfields` could be
    /// applied without error.
    ///
    /// Returns an error when subfields could not be applied:
    /// 1) in the case where `self` is not a `TypeInfo::Struct`
    /// 2) in the case where `subfields` is empty
    /// 3) in the case where a `subfield` does not exist on `self`
    pub(crate) fn apply_subfields(
        &self,
        type_engine: &TypeEngine,
        subfields: &[Ident],
        span: &Span,
    ) -> CompileResult<ty::TyStructField> {
        let mut warnings = vec![];
        let mut errors = vec![];
        match (self, subfields.split_first()) {
            (TypeInfo::Struct { .. }, None) => err(warnings, errors),
            (TypeInfo::Struct { name, fields, .. }, Some((first, rest))) => {
                let field = match fields
                    .iter()
                    .find(|field| field.name.as_str() == first.as_str())
                {
                    Some(field) => field.clone(),
                    None => {
                        // gather available fields for the error message
                        let available_fields =
                            fields.iter().map(|x| x.name.as_str()).collect::<Vec<_>>();
                        errors.push(CompileError::FieldNotFound {
                            field_name: first.clone(),
                            struct_name: name.clone(),
                            available_fields: available_fields.join(", "),
                        });
                        return err(warnings, errors);
                    }
                };
                let field = if rest.is_empty() {
                    field
                } else {
                    check!(
                        type_engine.look_up_type_id(field.type_id).apply_subfields(
                            type_engine,
                            rest,
                            span
                        ),
                        return err(warnings, errors),
                        warnings,
                        errors
                    )
                };
                ok(field, warnings, errors)
            }
            (TypeInfo::ErrorRecovery, _) => {
                // dont create a new error in this case
                err(warnings, errors)
            }
            (type_info, _) => {
                errors.push(CompileError::FieldAccessOnNonStruct {
                    actually: type_engine.help_out(type_info).to_string(),
                    span: span.clone(),
                });
                err(warnings, errors)
            }
        }
    }

    pub(crate) fn can_change(&self) -> bool {
        // TODO: there might be an optimization here that if the type params hold
        // only non-dynamic types, then it doesn't matter that there are type params
        match self {
            TypeInfo::Enum {
                type_parameters, ..
            } => !type_parameters.is_empty(),
            TypeInfo::Struct {
                type_parameters, ..
            } => !type_parameters.is_empty(),
            TypeInfo::Str(_)
            | TypeInfo::UnsignedInteger(_)
            | TypeInfo::Boolean
            | TypeInfo::B256
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice
            | TypeInfo::ErrorRecovery => false,
            TypeInfo::Unknown
            | TypeInfo::UnknownGeneric { .. }
            | TypeInfo::ContractCaller { .. }
            | TypeInfo::Custom { .. }
            | TypeInfo::SelfType
            | TypeInfo::Tuple(_)
            | TypeInfo::Array(_, _, _)
            | TypeInfo::Contract
            | TypeInfo::Storage { .. }
            | TypeInfo::Numeric => true,
        }
    }

    /// Given a `TypeInfo` `self`, expect that `self` is a `TypeInfo::Tuple`,
    /// and return its contents.
    ///
    /// Returns an error if `self` is not a `TypeInfo::Tuple`.
    pub(crate) fn expect_tuple(
        &self,
        type_engine: &TypeEngine,
        debug_string: impl Into<String>,
        debug_span: &Span,
    ) -> CompileResult<&Vec<TypeArgument>> {
        let warnings = vec![];
        let errors = vec![];
        match self {
            TypeInfo::Tuple(elems) => ok(elems, warnings, errors),
            TypeInfo::ErrorRecovery => err(warnings, errors),
            a => err(
                vec![],
                vec![CompileError::NotATuple {
                    name: debug_string.into(),
                    span: debug_span.clone(),
                    actually: type_engine.help_out(a).to_string(),
                }],
            ),
        }
    }

    /// Given a `TypeInfo` `self`, expect that `self` is a `TypeInfo::Enum`,
    /// and return its contents.
    ///
    /// Returns an error if `self` is not a `TypeInfo::Enum`.
    pub(crate) fn expect_enum(
        &self,
        type_engine: &TypeEngine,
        debug_string: impl Into<String>,
        debug_span: &Span,
    ) -> CompileResult<(&Ident, &Vec<ty::TyEnumVariant>)> {
        let warnings = vec![];
        let errors = vec![];
        match self {
            TypeInfo::Enum {
                name,
                variant_types,
                ..
            } => ok((name, variant_types), warnings, errors),
            TypeInfo::ErrorRecovery => err(warnings, errors),
            a => err(
                vec![],
                vec![CompileError::NotAnEnum {
                    name: debug_string.into(),
                    span: debug_span.clone(),
                    actually: type_engine.help_out(a).to_string(),
                }],
            ),
        }
    }

    /// Given a `TypeInfo` `self`, expect that `self` is a `TypeInfo::Struct`,
    /// and return its contents.
    ///
    /// Returns an error if `self` is not a `TypeInfo::Struct`.
    #[allow(dead_code)]
    pub(crate) fn expect_struct(
        &self,
        type_engine: &TypeEngine,
        debug_span: &Span,
    ) -> CompileResult<(&Ident, &Vec<ty::TyStructField>)> {
        let warnings = vec![];
        let errors = vec![];
        match self {
            TypeInfo::Struct { name, fields, .. } => ok((name, fields), warnings, errors),
            TypeInfo::ErrorRecovery => err(warnings, errors),
            a => err(
                vec![],
                vec![CompileError::NotAStruct {
                    span: debug_span.clone(),
                    actually: type_engine.help_out(a).to_string(),
                }],
            ),
        }
    }
}

/// Given two lists of `TypeInfo`'s `left` and `right`, check to see if
/// `left` is a subset of `right`.
///
/// `left` is a subset of `right` if the following invariants are true:
/// 1. `left` and and `right` are of the same length _n_
/// 2. For every _i_ in [0, n), `left`ᵢ is a subset of `right`ᵢ
/// 3. The elements of `left` satisfy the trait constraints of `right`
///
/// A property that falls of out these constraints are that if `left` and
/// `right` are empty, then `left` is a subset of `right`.
///
/// Given:
///
/// ```ignore
/// left:   [T]
/// right:  [T, F]
/// ```
///
/// `left` is not a subset of `right` because it violates invariant #1.
///
/// Given:
///
/// ```ignore
/// left:   [T, F]
/// right:  [bool, F]
/// ```
///
/// `left` is not a subset of `right` because it violates invariant #2.
///
/// Given:
///
/// ```ignore
/// left:   [T, F]
/// right:  [T, T]
/// ```
///
/// `left` is not a subset of `right` because it violates invariant #3.
///
/// Given:
///
/// ```ignore
/// left:   [T, T]
/// right:  [T, F]
/// ```
///
/// `left` is a subset of `right`.
///
/// Given:
///
/// ```ignore
/// left:   [bool, T]
/// right:  [T, F]
/// ```
///
/// `left` is a subset of `right`.
///
/// Given:
///
/// ```ignore
/// left:   [Data<T, T>, Data<T, F>]
/// right:  [Data<T, F>, Data<T, F>]
/// ```
///
/// `left` is a subset of `right`.
///
fn types_are_subset_of(type_engine: &TypeEngine, left: &[TypeInfo], right: &[TypeInfo]) -> bool {
    // invariant 1. `left` and and `right` are of the same length _n_
    if left.len() != right.len() {
        return false;
    }

    // if `left` and `right` are empty, `left` is inherently a subset of `right`
    if left.is_empty() && right.is_empty() {
        return true;
    }

    // invariant 2. For every _i_ in [0, n), `left`ᵢ is a subset of `right`ᵢ
    for (l, r) in left.iter().zip(right.iter()) {
        if !l.is_subset_of(r, type_engine) {
            return false;
        }
    }

    // invariant 3. The elements of `left` satisfy the constraints of `right`
    let mut constraints = vec![];
    for i in 0..(right.len() - 1) {
        for j in (i + 1)..right.len() {
            let a = right.get(i).unwrap();
            let b = right.get(j).unwrap();
            if a.eq(b, type_engine) {
                // if a and b are the same type
                constraints.push((i, j));
            }
        }
    }
    for (i, j) in constraints.into_iter() {
        let a = left.get(i).unwrap();
        let b = left.get(j).unwrap();
        if !a.eq(b, type_engine) {
            return false;
        }
    }

    // if all of the invariants are met, then `self` is a subset of `other`!
    true
}

fn print_inner_types(name: String, inner_types: impl Iterator<Item = TypeId>) -> String {
    let inner_types = inner_types.map(|x| x.to_string()).collect::<Vec<_>>();
    format!(
        "{}{}",
        name,
        if inner_types.is_empty() {
            "".into()
        } else {
            format!("<{}>", inner_types.join(", "))
        }
    )
}

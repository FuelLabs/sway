use crate::{
    decl_engine::{DeclEngine, DeclRefEnum, DeclRefStruct},
    engine_threading::*,
    error::*,
    language::{ty, CallPath},
    type_system::priv_prelude::*,
    Ident,
};
use sway_error::error::CompileError;
use sway_types::{integer_bits::IntegerBits, span::Span, Spanned};

use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashSet},
    fmt,
    hash::{Hash, Hasher},
};

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
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
        &self.0
    }
}

impl<T: PartialEqWithEngines> VecSet<T> {
    pub fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.0.len() <= other.0.len()
            && self
                .0
                .iter()
                .all(|x| other.0.iter().any(|y| x.eq(y, engines)))
    }
}

impl<T: PartialEqWithEngines> PartialEqWithEngines for VecSet<T> {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.eq(other, engines) && other.eq(self, engines)
    }
}

/// Type information without an associated value, used for type inferencing and definition.
#[derive(Debug, Clone, Default)]
pub enum TypeInfo {
    #[default]
    Unknown,
    /// Represents a type parameter.
    ///
    /// The equivalent type in the Rust compiler is:
    /// https://doc.rust-lang.org/nightly/nightly-rustc/src/rustc_type_ir/sty.rs.html#190
    UnknownGeneric {
        name: Ident,
        // NOTE(Centril): Used to be BTreeSet; need to revert back later. Must be sorted!
        trait_constraints: VecSet<TraitConstraint>,
    },
    /// Represents a type that will be inferred by the Sway compiler. This type
    /// is created when the user writes code that creates a new ADT that has
    /// type parameters in it's definition, before type inference can determine
    /// what the type of those type parameters are.
    ///
    /// This type would also be created in a case where the user wrote a type
    /// annotation with a wildcard type, like:
    /// `let v: Vec<_> = iter.collect();`. However, this is not yet implemented
    /// in Sway.
    ///
    /// The equivalent type in the Rust compiler is:
    /// https://doc.rust-lang.org/nightly/nightly-rustc/src/rustc_type_ir/sty.rs.html#208
    Placeholder(TypeParameter),
    /// Represents a type created from a type parameter.
    ///
    /// NOTE: This type is *not used yet*.
    // https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/enum.TyKind.html#variant.Param
    TypeParam(usize),
    Str(Length),
    UnsignedInteger(IntegerBits),
    Enum(DeclRefEnum),
    Struct(DeclRefStruct),
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
        call_path: CallPath,
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
    // Static, constant size arrays.
    Array(TypeArgument, Length),
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
    /// Type Alias. This type and the type `ty` it encapsulates always coerce. They are effectively
    /// interchangeable
    Alias {
        name: Ident,
        ty: TypeArgument,
    },
}

impl HashWithEngines for TypeInfo {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        self.discriminant_value().hash(state);
        match self {
            TypeInfo::Str(len) => {
                len.hash(state);
            }
            TypeInfo::UnsignedInteger(bits) => {
                bits.hash(state);
            }
            TypeInfo::Tuple(fields) => {
                fields.hash(state, engines);
            }
            TypeInfo::Enum(decl_ref) => {
                decl_ref.hash(state, engines);
            }
            TypeInfo::Struct(decl_ref) => {
                decl_ref.hash(state, engines);
            }
            TypeInfo::ContractCaller { abi_name, address } => {
                abi_name.hash(state);
                let address = address
                    .as_ref()
                    .map(|x| x.span.as_str().to_string())
                    .unwrap_or_default();
                address.hash(state);
            }
            TypeInfo::UnknownGeneric {
                name,
                trait_constraints,
            } => {
                name.hash(state);
                trait_constraints.hash(state, engines);
            }
            TypeInfo::Custom {
                call_path,
                type_arguments,
            } => {
                call_path.hash(state);
                type_arguments.as_deref().hash(state, engines);
            }
            TypeInfo::Storage { fields } => {
                fields.hash(state, engines);
            }
            TypeInfo::Array(elem_ty, count) => {
                elem_ty.hash(state, engines);
                count.hash(state);
            }
            TypeInfo::Placeholder(ty) => {
                ty.hash(state, engines);
            }
            TypeInfo::TypeParam(n) => {
                n.hash(state);
            }
            TypeInfo::Alias { name, ty } => {
                name.hash(state);
                ty.hash(state, engines);
            }
            TypeInfo::Numeric
            | TypeInfo::Boolean
            | TypeInfo::B256
            | TypeInfo::Contract
            | TypeInfo::ErrorRecovery
            | TypeInfo::Unknown
            | TypeInfo::SelfType
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice => {}
        }
    }
}

impl EqWithEngines for TypeInfo {}
impl PartialEqWithEngines for TypeInfo {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let type_engine = engines.te();
        match (self, other) {
            (
                Self::UnknownGeneric {
                    name: l,
                    trait_constraints: ltc,
                },
                Self::UnknownGeneric {
                    name: r,
                    trait_constraints: rtc,
                },
            ) => l == r && ltc.eq(rtc, engines),
            (Self::Placeholder(l), Self::Placeholder(r)) => l.eq(r, engines),
            (Self::TypeParam(l), Self::TypeParam(r)) => l == r,
            (
                Self::Custom {
                    call_path: l_name,
                    type_arguments: l_type_args,
                },
                Self::Custom {
                    call_path: r_name,
                    type_arguments: r_type_args,
                },
            ) => {
                l_name.suffix == r_name.suffix
                    && l_type_args.as_deref().eq(&r_type_args.as_deref(), engines)
            }
            (Self::Str(l), Self::Str(r)) => l.val() == r.val(),
            (Self::UnsignedInteger(l), Self::UnsignedInteger(r)) => l == r,
            (Self::Enum(l_decl_ref), Self::Enum(r_decl_ref)) => {
                let l_decl = engines.de().get_enum(l_decl_ref);
                let r_decl = engines.de().get_enum(r_decl_ref);
                l_decl.call_path.suffix == r_decl.call_path.suffix
                    && l_decl.call_path.suffix.span() == r_decl.call_path.suffix.span()
                    && l_decl.variants.eq(&r_decl.variants, engines)
                    && l_decl.type_parameters.eq(&r_decl.type_parameters, engines)
            }
            (Self::Struct(l_decl_ref), Self::Struct(r_decl_ref)) => {
                let l_decl = engines.de().get_struct(l_decl_ref);
                let r_decl = engines.de().get_struct(r_decl_ref);
                l_decl.call_path.suffix == r_decl.call_path.suffix
                    && l_decl.call_path.suffix.span() == r_decl.call_path.suffix.span()
                    && l_decl.fields.eq(&r_decl.fields, engines)
                    && l_decl.type_parameters.eq(&r_decl.type_parameters, engines)
            }
            (Self::Tuple(l), Self::Tuple(r)) => l
                .iter()
                .zip(r.iter())
                .map(|(l, r)| {
                    type_engine
                        .get(l.type_id)
                        .eq(&type_engine.get(r.type_id), engines)
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
                l_abi_name == r_abi_name && l_address.as_deref().eq(&r_address.as_deref(), engines)
            }
            (Self::Array(l0, l1), Self::Array(r0, r1)) => {
                type_engine
                    .get(l0.type_id)
                    .eq(&type_engine.get(r0.type_id), engines)
                    && l1.val() == r1.val()
            }
            (TypeInfo::Storage { fields: l_fields }, TypeInfo::Storage { fields: r_fields }) => {
                l_fields.eq(r_fields, engines)
            }
            (
                Self::Alias {
                    name: l_name,
                    ty: l_ty,
                },
                Self::Alias {
                    name: r_name,
                    ty: r_ty,
                },
            ) => {
                l_name == r_name
                    && type_engine
                        .get(l_ty.type_id)
                        .eq(&type_engine.get(r_ty.type_id), engines)
            }
            (l, r) => l.discriminant_value() == r.discriminant_value(),
        }
    }
}

impl OrdWithEngines for TypeInfo {
    fn cmp(&self, other: &Self, engines: Engines<'_>) -> Ordering {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        match (self, other) {
            (
                Self::UnknownGeneric {
                    name: l,
                    trait_constraints: ltc,
                },
                Self::UnknownGeneric {
                    name: r,
                    trait_constraints: rtc,
                },
            ) => l.cmp(r).then_with(|| ltc.cmp(rtc, engines)),
            (Self::Placeholder(l), Self::Placeholder(r)) => l.cmp(r, engines),
            (
                Self::Custom {
                    call_path: l_call_path,
                    type_arguments: l_type_args,
                },
                Self::Custom {
                    call_path: r_call_path,
                    type_arguments: r_type_args,
                },
            ) => l_call_path
                .suffix
                .cmp(&r_call_path.suffix)
                .then_with(|| l_type_args.as_deref().cmp(&r_type_args.as_deref(), engines)),
            (Self::Str(l), Self::Str(r)) => l.val().cmp(&r.val()),
            (Self::UnsignedInteger(l), Self::UnsignedInteger(r)) => l.cmp(r),
            (Self::Enum(l_decl_ref), Self::Enum(r_decl_ref)) => {
                let l_decl = decl_engine.get_enum(l_decl_ref);
                let r_decl = decl_engine.get_enum(r_decl_ref);
                l_decl
                    .call_path
                    .suffix
                    .cmp(&r_decl.call_path.suffix)
                    .then_with(|| l_decl.type_parameters.cmp(&r_decl.type_parameters, engines))
                    .then_with(|| l_decl.variants.cmp(&r_decl.variants, engines))
            }
            (Self::Struct(l_decl_ref), Self::Struct(r_decl_ref)) => {
                let l_decl = decl_engine.get_struct(l_decl_ref);
                let r_decl = decl_engine.get_struct(r_decl_ref);
                l_decl
                    .call_path
                    .suffix
                    .cmp(&r_decl.call_path.suffix)
                    .then_with(|| l_decl.type_parameters.cmp(&r_decl.type_parameters, engines))
                    .then_with(|| l_decl.fields.cmp(&r_decl.fields, engines))
            }
            (Self::Tuple(l), Self::Tuple(r)) => l.cmp(r, engines),
            (
                Self::ContractCaller {
                    abi_name: l_abi_name,
                    address: _,
                },
                Self::ContractCaller {
                    abi_name: r_abi_name,
                    address: _,
                },
            ) => {
                // NOTE: we assume all contract callers are unique
                l_abi_name.cmp(r_abi_name)
            }
            (Self::Array(l0, l1), Self::Array(r0, r1)) => type_engine
                .get(l0.type_id)
                .cmp(&type_engine.get(r0.type_id), engines)
                .then_with(|| l1.val().cmp(&r1.val())),
            (TypeInfo::Storage { fields: l_fields }, TypeInfo::Storage { fields: r_fields }) => {
                l_fields.cmp(r_fields, engines)
            }
            (
                Self::Alias {
                    name: l_name,
                    ty: l_ty,
                },
                Self::Alias {
                    name: r_name,
                    ty: r_ty,
                },
            ) => type_engine
                .get(l_ty.type_id)
                .cmp(&type_engine.get(r_ty.type_id), engines)
                .then_with(|| l_name.cmp(r_name)),

            (l, r) => l.discriminant_value().cmp(&r.discriminant_value()),
        }
    }
}

impl DisplayWithEngines for TypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: Engines<'_>) -> fmt::Result {
        use TypeInfo::*;
        let s = match self {
            Unknown => "unknown".into(),
            UnknownGeneric { name, .. } => name.to_string(),
            Placeholder(_) => "_".to_string(),
            TypeParam(n) => format!("typeparam({n})"),
            Str(x) => format!("str[{}]", x.val()),
            UnsignedInteger(x) => match x {
                IntegerBits::Eight => "u8",
                IntegerBits::Sixteen => "u16",
                IntegerBits::ThirtyTwo => "u32",
                IntegerBits::SixtyFour => "u64",
            }
            .into(),
            Boolean => "bool".into(),
            Custom { call_path, .. } => {
                format!("unresolved {}", call_path.suffix.as_str())
            }
            Tuple(fields) => {
                let field_strs = fields
                    .iter()
                    .map(|field| engines.help_out(field).to_string())
                    .collect::<Vec<String>>();
                format!("({})", field_strs.join(", "))
            }
            SelfType => "Self".into(),
            B256 => "b256".into(),
            Numeric => "numeric".into(),
            Contract => "contract".into(),
            ErrorRecovery => "unknown due to error".into(),
            Enum(decl_ref) => {
                let decl = engines.de().get_enum(decl_ref);
                print_inner_types(
                    engines,
                    decl.call_path.suffix.as_str().to_string(),
                    decl.type_parameters.iter().map(|x| x.type_id),
                )
            }
            Struct(decl_ref) => {
                let decl = engines.de().get_struct(decl_ref);
                print_inner_types(
                    engines,
                    decl.call_path.suffix.as_str().to_string(),
                    decl.type_parameters.iter().map(|x| x.type_id),
                )
            }
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
            Array(elem_ty, count) => {
                format!("[{}; {}]", engines.help_out(elem_ty), count.val())
            }
            Storage { .. } => "contract storage".into(),
            RawUntypedPtr => "raw untyped ptr".into(),
            RawUntypedSlice => "raw untyped slice".into(),
            Alias { name, ty } => {
                format!("type {} = {}", name, engines.help_out(ty))
            }
        };
        write!(f, "{s}")
    }
}

impl TypeInfo {
    pub fn display_name(&self) -> String {
        match self {
            TypeInfo::UnknownGeneric { name, .. } => name.to_string(),
            TypeInfo::Placeholder(type_param) => type_param.name_ident.to_string(),
            TypeInfo::Enum(decl_ref) => decl_ref.name().clone().to_string(),
            TypeInfo::Struct(decl_ref) => decl_ref.name().clone().to_string(),
            TypeInfo::ContractCaller { abi_name, .. } => abi_name.to_string(),
            TypeInfo::Custom { call_path, .. } => call_path.to_string(),
            TypeInfo::Storage { .. } => "storage".into(),
            _ => format!("{self:?}"),
        }
    }
}

impl TypeInfo {
    /// Returns a discriminant for the variant.
    // NOTE: This is approach is not the most straightforward, but is needed
    // because of this missing feature on Rust's `Discriminant` type:
    // https://github.com/rust-lang/rust/pull/106418
    fn discriminant_value(&self) -> u8 {
        match self {
            TypeInfo::Unknown => 0,
            TypeInfo::UnknownGeneric { .. } => 1,
            TypeInfo::Placeholder(_) => 2,
            TypeInfo::Str(_) => 3,
            TypeInfo::UnsignedInteger(_) => 4,
            TypeInfo::Enum { .. } => 5,
            TypeInfo::Struct { .. } => 6,
            TypeInfo::Boolean => 7,
            TypeInfo::Tuple(_) => 8,
            TypeInfo::ContractCaller { .. } => 9,
            TypeInfo::Custom { .. } => 10,
            TypeInfo::SelfType => 11,
            TypeInfo::B256 => 12,
            TypeInfo::Numeric => 13,
            TypeInfo::Contract => 14,
            TypeInfo::ErrorRecovery => 15,
            TypeInfo::Array(_, _) => 16,
            TypeInfo::Storage { .. } => 17,
            TypeInfo::RawUntypedPtr => 18,
            TypeInfo::RawUntypedSlice => 19,
            TypeInfo::TypeParam(_) => 20,
            TypeInfo::Alias { .. } => 21,
        }
    }

    /// maps a type to a name that is used when constructing function selectors
    pub(crate) fn to_selector_name(
        &self,
        type_engine: &TypeEngine,
        decl_engine: &DeclEngine,
        error_msg_span: &Span,
    ) -> CompileResult<String> {
        use TypeInfo::*;
        let name = match self {
            Str(len) => format!("str[{}]", len.val()),
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
                                .to_selector_name(type_engine, decl_engine, error_msg_span)
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
            Struct(decl_ref) => {
                let decl = decl_engine.get_struct(decl_ref);
                let field_names = {
                    let names = decl
                        .fields
                        .iter()
                        .map(|ty| {
                            let ty = match type_engine
                                .to_typeinfo(ty.type_argument.type_id, error_msg_span)
                            {
                                Err(e) => return err(vec![], vec![e.into()]),
                                Ok(ty) => ty,
                            };
                            ty.to_selector_name(type_engine, decl_engine, error_msg_span)
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
                    let type_arguments = decl
                        .type_parameters
                        .iter()
                        .map(|ty| {
                            let ty = match type_engine.to_typeinfo(ty.type_id, error_msg_span) {
                                Err(e) => return err(vec![], vec![e.into()]),
                                Ok(ty) => ty,
                            };
                            ty.to_selector_name(type_engine, decl_engine, error_msg_span)
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
            Enum(decl_ref) => {
                let decl = decl_engine.get_enum(decl_ref);
                let variant_names = {
                    let names = decl
                        .variants
                        .iter()
                        .map(|ty| {
                            let ty = match type_engine
                                .to_typeinfo(ty.type_argument.type_id, error_msg_span)
                            {
                                Err(e) => return err(vec![], vec![e.into()]),
                                Ok(ty) => ty,
                            };
                            ty.to_selector_name(type_engine, decl_engine, error_msg_span)
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
                    let type_arguments = decl
                        .type_parameters
                        .iter()
                        .map(|ty| {
                            let ty = match type_engine.to_typeinfo(ty.type_id, error_msg_span) {
                                Err(e) => return err(vec![], vec![e.into()]),
                                Ok(ty) => ty,
                            };
                            ty.to_selector_name(type_engine, decl_engine, error_msg_span)
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
            Array(elem_ty, length) => {
                let name = type_engine.get(elem_ty.type_id).to_selector_name(
                    type_engine,
                    decl_engine,
                    error_msg_span,
                );
                let name = match name.value {
                    Some(name) => name,
                    None => return name,
                };
                format!("a[{};{}]", name, length.val())
            }
            RawUntypedPtr => "rawptr".to_string(),
            RawUntypedSlice => "rawslice".to_string(),
            Alias { ty, .. } => {
                let name = type_engine.get(ty.type_id).to_selector_name(
                    type_engine,
                    decl_engine,
                    error_msg_span,
                );
                match name.value {
                    Some(name) => name,
                    None => return name,
                }
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

    pub fn is_uninhabited(&self, type_engine: &TypeEngine, decl_engine: &DeclEngine) -> bool {
        let id_uninhabited = |id| type_engine.get(id).is_uninhabited(type_engine, decl_engine);

        match self {
            TypeInfo::Enum(decl_ref) => decl_engine
                .get_enum(decl_ref)
                .variants
                .iter()
                .all(|variant_type| id_uninhabited(variant_type.type_argument.type_id)),
            TypeInfo::Struct(decl_ref) => decl_engine
                .get_struct(decl_ref)
                .fields
                .iter()
                .any(|field| id_uninhabited(field.type_argument.type_id)),
            TypeInfo::Tuple(fields) => fields
                .iter()
                .any(|field_type| id_uninhabited(field_type.type_id)),
            TypeInfo::Array(elem_ty, length) => length.val() > 0 && id_uninhabited(elem_ty.type_id),
            _ => false,
        }
    }

    pub fn is_zero_sized(&self, type_engine: &TypeEngine, decl_engine: &DeclEngine) -> bool {
        match self {
            TypeInfo::Enum(decl_ref) => {
                let decl = decl_engine.get_enum(decl_ref);
                let mut found_unit_variant = false;
                for variant_type in decl.variants {
                    let type_info = type_engine.get(variant_type.type_argument.type_id);
                    if type_info.is_uninhabited(type_engine, decl_engine) {
                        continue;
                    }
                    if type_info.is_zero_sized(type_engine, decl_engine) && !found_unit_variant {
                        found_unit_variant = true;
                        continue;
                    }
                    return false;
                }
                true
            }
            TypeInfo::Struct(decl_ref) => {
                let decl = decl_engine.get_struct(decl_ref);
                let mut all_zero_sized = true;
                for field in decl.fields {
                    let type_info = type_engine.get(field.type_argument.type_id);
                    if type_info.is_uninhabited(type_engine, decl_engine) {
                        return true;
                    }
                    if !type_info.is_zero_sized(type_engine, decl_engine) {
                        all_zero_sized = false;
                    }
                }
                all_zero_sized
            }
            TypeInfo::Tuple(fields) => {
                let mut all_zero_sized = true;
                for field in fields {
                    let field_type = type_engine.get(field.type_id);
                    if field_type.is_uninhabited(type_engine, decl_engine) {
                        return true;
                    }
                    if !field_type.is_zero_sized(type_engine, decl_engine) {
                        all_zero_sized = false;
                    }
                }
                all_zero_sized
            }
            TypeInfo::Array(elem_ty, length) => {
                length.val() == 0
                    || type_engine
                        .get(elem_ty.type_id)
                        .is_zero_sized(type_engine, decl_engine)
            }
            _ => false,
        }
    }

    pub fn can_safely_ignore(&self, type_engine: &TypeEngine, decl_engine: &DeclEngine) -> bool {
        if self.is_zero_sized(type_engine, decl_engine) {
            return true;
        }
        match self {
            TypeInfo::Tuple(fields) => fields.iter().all(|type_argument| {
                type_engine
                    .get(type_argument.type_id)
                    .can_safely_ignore(type_engine, decl_engine)
            }),
            TypeInfo::Array(elem_ty, length) => {
                length.val() == 0
                    || type_engine
                        .get(elem_ty.type_id)
                        .can_safely_ignore(type_engine, decl_engine)
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
        matches!(
            self,
            TypeInfo::Boolean | TypeInfo::UnsignedInteger(_) | TypeInfo::RawUntypedPtr
        ) || self.is_unit()
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
                call_path,
                type_arguments: other_type_arguments,
            } => {
                if other_type_arguments.is_some() {
                    errors.push(CompileError::TypeArgumentsNotAllowed { span: span.clone() });
                    err(warnings, errors)
                } else {
                    let type_info = TypeInfo::Custom {
                        call_path,
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
            | TypeInfo::Array(_, _)
            | TypeInfo::Storage { .. }
            | TypeInfo::Placeholder(_)
            | TypeInfo::TypeParam(_)
            | TypeInfo::Alias { .. } => {
                errors.push(CompileError::TypeArgumentsNotAllowed { span: span.clone() });
                err(warnings, errors)
            }
        }
    }

    /// Given a `TypeInfo` `self`, analyze `self` and return all inner
    /// `TypeId`'s of `self`, not including `self`.
    pub(crate) fn extract_inner_types(&self, engines: Engines<'_>) -> BTreeSet<TypeId> {
        fn filter_fn(_type_info: &TypeInfo) -> bool {
            true
        }
        self.extract_any(engines, &filter_fn)
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
            | TypeInfo::Numeric
            | TypeInfo::Alias { .. } => ok((), warnings, errors),
            TypeInfo::Unknown
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice
            | TypeInfo::ContractCaller { .. }
            | TypeInfo::Custom { .. }
            | TypeInfo::SelfType
            | TypeInfo::Str(_)
            | TypeInfo::Contract
            | TypeInfo::Array(_, _)
            | TypeInfo::Storage { .. }
            | TypeInfo::Placeholder(_)
            | TypeInfo::TypeParam(_) => {
                errors.push(CompileError::Unimplemented(
                    "matching on this type is unsupported right now",
                    span.clone(),
                ));
                err(warnings, errors)
            }
            TypeInfo::ErrorRecovery => {
                // return an error but don't create a new error message
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
            | TypeInfo::Array(_, _)
            | TypeInfo::Contract
            | TypeInfo::Numeric
            | TypeInfo::Alias { .. } => ok((), warnings, errors),
            TypeInfo::Unknown
            | TypeInfo::UnknownGeneric { .. }
            | TypeInfo::ContractCaller { .. }
            | TypeInfo::SelfType
            | TypeInfo::Storage { .. }
            | TypeInfo::Placeholder(_)
            | TypeInfo::TypeParam(_) => {
                errors.push(CompileError::Unimplemented(
                    "implementing traits on this type is unsupported right now",
                    span.clone(),
                ));
                err(warnings, errors)
            }
            TypeInfo::ErrorRecovery => {
                // return an error but don't create a new error message
                err(warnings, errors)
            }
        }
    }

    /// Given a `TypeInfo` `self`, analyze `self` and return all nested
    /// `TypeInfo`'s found in `self`, including `self`.
    pub(crate) fn extract_nested_types(self, engines: Engines<'_>) -> Vec<TypeInfo> {
        let type_engine = engines.te();
        let mut inner_types: Vec<TypeInfo> = self
            .extract_inner_types(engines)
            .into_iter()
            .map(|type_id| type_engine.get(type_id))
            .collect();
        inner_types.push(self);
        inner_types
    }

    pub(crate) fn extract_any<F>(&self, engines: Engines<'_>, filter_fn: &F) -> BTreeSet<TypeId>
    where
        F: Fn(&TypeInfo) -> bool,
    {
        let decl_engine = engines.de();
        let mut found: BTreeSet<TypeId> = BTreeSet::new();
        match self {
            TypeInfo::Unknown
            | TypeInfo::Placeholder(_)
            | TypeInfo::TypeParam(_)
            | TypeInfo::Str(_)
            | TypeInfo::UnsignedInteger(_)
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice
            | TypeInfo::Boolean
            | TypeInfo::SelfType
            | TypeInfo::B256
            | TypeInfo::Numeric
            | TypeInfo::Contract
            | TypeInfo::ErrorRecovery => {}
            TypeInfo::Enum(enum_ref) => {
                let enum_decl = decl_engine.get_enum(enum_ref);
                for type_param in enum_decl.type_parameters.iter() {
                    found.extend(
                        type_param
                            .type_id
                            .extract_any_including_self(engines, filter_fn),
                    );
                }
                for variant in enum_decl.variants.iter() {
                    found.extend(
                        variant
                            .type_argument
                            .type_id
                            .extract_any_including_self(engines, filter_fn),
                    );
                }
            }
            TypeInfo::Struct(struct_ref) => {
                let struct_decl = decl_engine.get_struct(struct_ref);
                for type_param in struct_decl.type_parameters.iter() {
                    found.extend(
                        type_param
                            .type_id
                            .extract_any_including_self(engines, filter_fn),
                    );
                }
                for field in struct_decl.fields.iter() {
                    found.extend(
                        field
                            .type_argument
                            .type_id
                            .extract_any_including_self(engines, filter_fn),
                    );
                }
            }
            TypeInfo::Tuple(elems) => {
                for elem in elems.iter() {
                    found.extend(elem.type_id.extract_any_including_self(engines, filter_fn));
                }
            }
            TypeInfo::ContractCaller {
                abi_name: _,
                address,
            } => {
                if let Some(address) = address {
                    found.extend(
                        address
                            .return_type
                            .extract_any_including_self(engines, filter_fn),
                    );
                }
            }
            TypeInfo::Custom {
                call_path: _,
                type_arguments,
            } => {
                if let Some(type_arguments) = type_arguments {
                    for type_arg in type_arguments.iter() {
                        found.extend(
                            type_arg
                                .type_id
                                .extract_any_including_self(engines, filter_fn),
                        );
                    }
                }
            }
            TypeInfo::Array(ty, _) => {
                found.extend(ty.type_id.extract_any_including_self(engines, filter_fn));
            }
            TypeInfo::Storage { fields } => {
                for field in fields.iter() {
                    found.extend(
                        field
                            .type_argument
                            .type_id
                            .extract_any_including_self(engines, filter_fn),
                    );
                }
            }
            TypeInfo::Alias { name: _, ty } => {
                found.extend(ty.type_id.extract_any_including_self(engines, filter_fn));
            }
            TypeInfo::UnknownGeneric {
                name: _,
                trait_constraints,
            } => {
                for trait_constraint in trait_constraints.iter() {
                    for type_arg in trait_constraint.type_arguments.iter() {
                        found.extend(
                            type_arg
                                .type_id
                                .extract_any_including_self(engines, filter_fn),
                        );
                    }
                }
            }
        }
        found
    }

    pub(crate) fn extract_nested_generics<'a>(
        &self,
        engines: Engines<'a>,
    ) -> HashSet<WithEngines<'a, TypeInfo>> {
        let nested_types = self.clone().extract_nested_types(engines);
        HashSet::from_iter(
            nested_types
                .into_iter()
                .filter(|x| matches!(x, TypeInfo::UnknownGeneric { .. }))
                .map(|thing| WithEngines::new(thing, engines)),
        )
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
    pub(crate) fn is_subset_of(&self, other: &TypeInfo, engines: Engines<'_>) -> bool {
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
                return rtc.eq(ltc, engines);
            }
            // any type is the subset of a generic
            (_, Self::UnknownGeneric { .. }) => {
                return true;
            }
            _ => {}
        }

        self.is_subset_inner(other, engines)
    }

    /// Given two `TypeInfo`'s `self` and `other`, checks to see if `self` is
    /// unidirectionally a subset of `other`, excluding consideration of generic
    /// types (like in the `is_subset_of` method).
    pub(crate) fn is_subset_of_for_item_import(
        &self,
        other: &TypeInfo,
        engines: Engines<'_>,
    ) -> bool {
        self.is_subset_inner(other, engines)
    }

    fn is_subset_inner(&self, other: &TypeInfo, engines: Engines<'_>) -> bool {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        match (self, other) {
            (Self::Array(l0, l1), Self::Array(r0, r1)) => {
                type_engine
                    .get(l0.type_id)
                    .is_subset_of(&type_engine.get(r0.type_id), engines)
                    && l1.val() == r1.val()
            }
            (
                Self::Custom {
                    call_path: l_name,
                    type_arguments: l_type_args,
                },
                Self::Custom {
                    call_path: r_name,
                    type_arguments: r_type_args,
                },
            ) => {
                let l_types = l_type_args
                    .as_ref()
                    .unwrap_or(&vec![])
                    .iter()
                    .map(|x| type_engine.get(x.type_id))
                    .collect::<Vec<_>>();
                let r_types = r_type_args
                    .as_ref()
                    .unwrap_or(&vec![])
                    .iter()
                    .map(|x| type_engine.get(x.type_id))
                    .collect::<Vec<_>>();
                l_name.suffix == r_name.suffix && types_are_subset_of(engines, &l_types, &r_types)
            }
            (Self::Enum(l_decl_ref), Self::Enum(r_decl_ref)) => {
                let l_decl = decl_engine.get_enum(l_decl_ref);
                let r_decl = decl_engine.get_enum(r_decl_ref);
                let l_names = l_decl
                    .variants
                    .iter()
                    .map(|x| x.name.clone())
                    .collect::<Vec<_>>();
                let r_names = r_decl
                    .variants
                    .iter()
                    .map(|x| x.name.clone())
                    .collect::<Vec<_>>();
                let l_types = l_decl
                    .type_parameters
                    .iter()
                    .map(|x| type_engine.get(x.type_id))
                    .collect::<Vec<_>>();
                let r_types = r_decl
                    .type_parameters
                    .iter()
                    .map(|x| type_engine.get(x.type_id))
                    .collect::<Vec<_>>();
                l_decl_ref.name().clone() == r_decl_ref.name().clone()
                    && l_names == r_names
                    && types_are_subset_of(engines, &l_types, &r_types)
            }
            (Self::Struct(l_decl_ref), Self::Struct(r_decl_ref)) => {
                let l_decl = decl_engine.get_struct(l_decl_ref);
                let r_decl = decl_engine.get_struct(r_decl_ref);
                let l_names = l_decl
                    .fields
                    .iter()
                    .map(|x| x.name.clone())
                    .collect::<Vec<_>>();
                let r_names = r_decl
                    .fields
                    .iter()
                    .map(|x| x.name.clone())
                    .collect::<Vec<_>>();
                let l_types = l_decl
                    .type_parameters
                    .iter()
                    .map(|x| type_engine.get(x.type_id))
                    .collect::<Vec<_>>();
                let r_types = r_decl
                    .type_parameters
                    .iter()
                    .map(|x| type_engine.get(x.type_id))
                    .collect::<Vec<_>>();
                l_decl_ref.name().clone() == r_decl_ref.name().clone()
                    && l_names == r_names
                    && types_are_subset_of(engines, &l_types, &r_types)
            }
            (Self::Tuple(l_types), Self::Tuple(r_types)) => {
                let l_types = l_types
                    .iter()
                    .map(|x| type_engine.get(x.type_id))
                    .collect::<Vec<_>>();
                let r_types = r_types
                    .iter()
                    .map(|x| type_engine.get(x.type_id))
                    .collect::<Vec<_>>();
                types_are_subset_of(engines, &l_types, &r_types)
            }
            (Self::Alias { ty: l_ty, .. }, Self::Alias { ty: r_ty, .. }) => type_engine
                .get(l_ty.type_id)
                .is_subset_of(&type_engine.get(r_ty.type_id), engines),
            (a, b) => a.eq(b, engines),
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
        engines: Engines<'_>,
        subfields: &[Ident],
        span: &Span,
    ) -> CompileResult<ty::TyStructField> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let type_engine = engines.te();
        let decl_engine = engines.de();
        match (self, subfields.split_first()) {
            (TypeInfo::Struct { .. } | TypeInfo::Alias { .. }, None) => err(warnings, errors),
            (TypeInfo::Struct(decl_ref), Some((first, rest))) => {
                let decl = decl_engine.get_struct(decl_ref);
                let field = match decl
                    .fields
                    .iter()
                    .find(|field| field.name.as_str() == first.as_str())
                {
                    Some(field) => field.clone(),
                    None => {
                        // gather available fields for the error message
                        let available_fields = decl
                            .fields
                            .iter()
                            .map(|x| x.name.as_str())
                            .collect::<Vec<_>>();
                        errors.push(CompileError::FieldNotFound {
                            field_name: first.clone(),
                            struct_name: decl.call_path.suffix.clone(),
                            available_fields: available_fields.join(", "),
                            span: first.span(),
                        });
                        return err(warnings, errors);
                    }
                };
                let field = if rest.is_empty() {
                    field
                } else {
                    check!(
                        type_engine
                            .get(field.type_argument.type_id)
                            .apply_subfields(engines, rest, span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    )
                };
                ok(field, warnings, errors)
            }
            (
                TypeInfo::Alias {
                    ty: TypeArgument { type_id, .. },
                    ..
                },
                _,
            ) => type_engine
                .get(*type_id)
                .apply_subfields(engines, subfields, span),
            (TypeInfo::ErrorRecovery, _) => {
                // dont create a new error in this case
                err(warnings, errors)
            }
            (type_info, _) => {
                errors.push(CompileError::FieldAccessOnNonStruct {
                    actually: engines.help_out(type_info).to_string(),
                    span: span.clone(),
                });
                err(warnings, errors)
            }
        }
    }

    pub(crate) fn can_change(&self, decl_engine: &DeclEngine) -> bool {
        // TODO: there might be an optimization here that if the type params hold
        // only non-dynamic types, then it doesn't matter that there are type params
        match self {
            TypeInfo::Enum(decl_ref) => {
                let decl = decl_engine.get_enum(decl_ref);
                !decl.type_parameters.is_empty()
            }
            TypeInfo::Struct(decl_ref) => {
                let decl = decl_engine.get_struct(decl_ref);
                !decl.type_parameters.is_empty()
            }
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
            | TypeInfo::Array(_, _)
            | TypeInfo::Contract
            | TypeInfo::Storage { .. }
            | TypeInfo::Numeric
            | TypeInfo::Placeholder(_)
            | TypeInfo::TypeParam(_)
            | TypeInfo::Alias { .. } => true,
        }
    }

    /// Checks if a given [TypeInfo] has a valid constructor.
    pub(crate) fn has_valid_constructor(&self, decl_engine: &DeclEngine) -> bool {
        match self {
            TypeInfo::Unknown => false,
            TypeInfo::Enum(decl_ref) => {
                let decl = decl_engine.get_enum(decl_ref);
                !decl.variants.is_empty()
            }
            _ => true,
        }
    }

    /// Given a `TypeInfo` `self`, expect that `self` is a `TypeInfo::Tuple`, or a
    /// `TypeInfo::Alias` of a tuple type. Also, return the contents of the tuple.
    ///
    /// Note that this works recursively. That is, it supports situations where a tuple has a chain
    /// of aliases such as:
    ///
    /// ```
    /// type Alias1 = (u64, u64);
    /// type Alias2 = Alias1;
    ///
    /// fn foo(t: Alias2) {
    ///     let x = t.0;
    /// }
    /// ```
    ///
    /// Returns an error if `self` is not a `TypeInfo::Tuple` or a `TypeInfo::Alias` of a tuple
    /// type, transitively.
    pub(crate) fn expect_tuple(
        &self,
        engines: Engines<'_>,
        debug_string: impl Into<String>,
        debug_span: &Span,
    ) -> CompileResult<Vec<TypeArgument>> {
        let warnings = vec![];
        let errors = vec![];
        match self {
            TypeInfo::Tuple(elems) => ok(elems.to_vec(), warnings, errors),
            TypeInfo::Alias {
                ty: TypeArgument { type_id, .. },
                ..
            } => engines
                .te()
                .get(*type_id)
                .expect_tuple(engines, debug_string, debug_span),
            TypeInfo::ErrorRecovery => err(warnings, errors),
            a => err(
                vec![],
                vec![CompileError::NotATuple {
                    name: debug_string.into(),
                    span: debug_span.clone(),
                    actually: engines.help_out(a).to_string(),
                }],
            ),
        }
    }

    /// Given a `TypeInfo` `self`, expect that `self` is a `TypeInfo::Enum`, or a `TypeInfo::Alias`
    /// of a enum type. Also, return the contents of the enum.
    ///
    /// Note that this works recursively. That is, it supports situations where a enum has a chain
    /// of aliases such as:
    ///
    /// ```rust,ignore
    /// enum MyEnum { X: () }
    /// type Alias1 = MyEnum;
    /// type Alias2 = Alias1;
    ///
    /// let e = Alias2::X;
    /// ```
    ///
    /// Returns an error if `self` is not a `TypeInfo::Enum` or a `TypeInfo::Alias` of a enum type,
    /// transitively.
    pub(crate) fn expect_enum(
        &self,
        engines: Engines<'_>,
        debug_string: impl Into<String>,
        debug_span: &Span,
    ) -> CompileResult<DeclRefEnum> {
        let warnings = vec![];
        let errors = vec![];
        match self {
            TypeInfo::Enum(decl_ref) => ok(decl_ref.clone(), warnings, errors),
            TypeInfo::Alias {
                ty: TypeArgument { type_id, .. },
                ..
            } => engines
                .te()
                .get(*type_id)
                .expect_enum(engines, debug_string, debug_span),
            TypeInfo::ErrorRecovery => err(warnings, errors),
            a => err(
                vec![],
                vec![CompileError::NotAnEnum {
                    name: debug_string.into(),
                    span: debug_span.clone(),
                    actually: engines.help_out(a).to_string(),
                }],
            ),
        }
    }

    /// Given a `TypeInfo` `self`, expect that `self` is a `TypeInfo::Struct`, or a
    /// `TypeInfo::Alias` of a struct type. Also, return the contents of the struct.
    ///
    /// Note that this works recursively. That is, it supports situations where a struct has a
    /// chain of aliases such as:
    ///
    /// ```
    /// struct MyStruct { x: u64 }
    /// type Alias1 = MyStruct;
    /// type Alias2 = Alias1;
    ///
    /// let s = Alias2 { x: 0 };
    /// ```
    ///
    /// Returns an error if `self` is not a `TypeInfo::Struct` or a `TypeInfo::Alias` of a struct
    /// type, transitively.
    #[allow(dead_code)]
    pub(crate) fn expect_struct(
        &self,
        engines: Engines<'_>,
        debug_span: &Span,
    ) -> CompileResult<DeclRefStruct> {
        let warnings = vec![];
        let errors = vec![];
        match self {
            TypeInfo::Struct(decl_ref) => ok(decl_ref.clone(), warnings, errors),
            TypeInfo::Alias {
                ty: TypeArgument { type_id, .. },
                ..
            } => engines
                .te()
                .get(*type_id)
                .expect_struct(engines, debug_span),
            TypeInfo::ErrorRecovery => err(warnings, errors),
            a => err(
                vec![],
                vec![CompileError::NotAStruct {
                    span: debug_span.clone(),
                    actually: engines.help_out(a).to_string(),
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
fn types_are_subset_of(engines: Engines<'_>, left: &[TypeInfo], right: &[TypeInfo]) -> bool {
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
        if !l.is_subset_of(r, engines) {
            return false;
        }
    }

    // invariant 3. The elements of `left` satisfy the constraints of `right`
    let mut constraints = vec![];
    for i in 0..(right.len() - 1) {
        for j in (i + 1)..right.len() {
            let a = right.get(i).unwrap();
            let b = right.get(j).unwrap();
            if a.eq(b, engines) {
                // if a and b are the same type
                constraints.push((i, j));
            }
        }
    }
    for (i, j) in constraints.into_iter() {
        let a = left.get(i).unwrap();
        let b = left.get(j).unwrap();
        if !a.eq(b, engines) {
            return false;
        }
    }

    // if all of the invariants are met, then `self` is a subset of `other`!
    true
}

fn print_inner_types(
    engines: Engines<'_>,
    name: String,
    inner_types: impl Iterator<Item = TypeId>,
) -> String {
    let inner_types = inner_types
        .map(|x| engines.help_out(x).to_string())
        .collect::<Vec<_>>();
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

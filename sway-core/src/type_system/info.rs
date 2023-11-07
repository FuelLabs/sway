use crate::{
    decl_engine::{DeclEngine, DeclRefEnum, DeclRefStruct},
    engine_threading::*,
    language::{ty, CallPath, QualifiedCallPath},
    type_system::priv_prelude::*,
    Ident,
};
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{integer_bits::IntegerBits, span::Span, SourceId, Spanned};

use std::{
    cmp::Ordering,
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
    pub fn eq(&self, other: &Self, engines: &Engines) -> bool {
        self.0.len() <= other.0.len()
            && self
                .0
                .iter()
                .all(|x| other.0.iter().any(|y| x.eq(y, engines)))
    }
}

impl<T: PartialEqWithEngines> PartialEqWithEngines for VecSet<T> {
    fn eq(&self, other: &Self, engines: &Engines) -> bool {
        self.eq(other, engines) && other.eq(self, engines)
    }
}

/// Encapsulates type information and its optional source identifier.
#[derive(Debug, Default, Clone)]
pub struct TypeSourceInfo {
    pub(crate) type_info: TypeInfo,
    /// The source id that created this type.
    pub(crate) source_id: Option<SourceId>,
}

impl HashWithEngines for TypeSourceInfo {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        self.type_info.hash(state, engines);
        self.source_id.hash(state);
    }
}

impl EqWithEngines for TypeSourceInfo {}
impl PartialEqWithEngines for TypeSourceInfo {
    fn eq(&self, other: &Self, engines: &Engines) -> bool {
        self.type_info.eq(&other.type_info, engines) && self.source_id == other.source_id
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
    StringSlice,
    StringArray(Length),
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
        qualified_call_path: QualifiedCallPath,
        type_arguments: Option<Vec<TypeArgument>>,
        /// When root_type_id contains some type id then the call path applies
        /// to the specified root_type_id as root.
        /// This is used by associated types which should produce a TypeInfo::Custom
        /// such as Self::T.
        root_type_id: Option<TypeId>,
    },
    B256,
    /// This means that specific type of a number is not yet known. It will be
    /// determined via inference at a later time.
    Numeric,
    Contract,
    // used for recovering from errors in the ast
    ErrorRecovery(ErrorEmitted),
    // Static, constant size arrays.
    Array(TypeArgument, Length),
    /// Represents the entire storage declaration struct
    /// Stored without initializers here, as typed struct fields,
    /// so type checking is able to treat it as a struct with fields.
    Storage {
        fields: Vec<ty::TyStructField>,
    },
    /// Pointers.
    /// These are represented in memory as u64 but are a different type since pointers only make
    /// sense in the context they were created in. Users can obtain pointers via standard library
    /// functions such `alloc` or `stack_ptr`. These functions are implemented using asm blocks
    /// which can create pointers by (eg.) reading logically-pointer-valued registers, using the
    /// gtf instruction, or manipulating u64s.
    RawUntypedPtr,
    RawUntypedSlice,
    Ptr(TypeArgument),
    Slice(TypeArgument),
    /// Type Alias. This type and the type `ty` it encapsulates always coerce. They are effectively
    /// interchangeable
    Alias {
        name: Ident,
        ty: TypeArgument,
    },
    TraitType {
        name: Ident,
        trait_type_id: TypeId,
    },
}

impl HashWithEngines for TypeInfo {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        self.discriminant_value().hash(state);
        match self {
            TypeInfo::StringArray(len) => {
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
                trait_constraints: _,
            } => {
                name.hash(state);
                // Do not hash trait_constraints as those can point back to this type_info
                // This avoids infinite hash loop. More collisions should occur but
                // Eq implementations can disambiguate.
                //trait_constraints.hash(state, engines);
            }
            TypeInfo::Custom {
                qualified_call_path: call_path,
                type_arguments,
                root_type_id,
            } => {
                call_path.hash(state, engines);
                type_arguments.as_deref().hash(state, engines);
                root_type_id.hash(state);
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
            TypeInfo::Ptr(ty) => {
                ty.hash(state, engines);
            }
            TypeInfo::Slice(ty) => {
                ty.hash(state, engines);
            }
            TypeInfo::TraitType {
                name,
                trait_type_id,
            } => {
                name.hash(state);
                trait_type_id.hash(state);
            }
            TypeInfo::StringSlice
            | TypeInfo::Numeric
            | TypeInfo::Boolean
            | TypeInfo::B256
            | TypeInfo::Contract
            | TypeInfo::ErrorRecovery(_)
            | TypeInfo::Unknown
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice => {}
        }
    }
}

impl EqWithEngines for TypeInfo {}
impl PartialEqWithEngines for TypeInfo {
    fn eq(&self, other: &Self, engines: &Engines) -> bool {
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
                    qualified_call_path: l_name,
                    type_arguments: l_type_args,
                    root_type_id: l_root_type_id,
                },
                Self::Custom {
                    qualified_call_path: r_name,
                    type_arguments: r_type_args,
                    root_type_id: r_root_type_id,
                },
            ) => {
                l_name.call_path.suffix == r_name.call_path.suffix
                    && l_name
                        .qualified_path_root
                        .eq(&r_name.qualified_path_root, engines)
                    && l_type_args.as_deref().eq(&r_type_args.as_deref(), engines)
                    && l_root_type_id.eq(r_root_type_id)
            }
            (Self::StringSlice, Self::StringSlice) => true,
            (Self::StringArray(l), Self::StringArray(r)) => l.val() == r.val(),
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
            (
                TypeInfo::TraitType {
                    name: l_name,
                    trait_type_id: l_trait_type_id,
                },
                TypeInfo::TraitType {
                    name: r_name,
                    trait_type_id: r_trait_type_id,
                },
            ) => {
                l_name == r_name
                    && type_engine
                        .get(*l_trait_type_id)
                        .eq(&type_engine.get(*r_trait_type_id), engines)
            }
            (l, r) => l.discriminant_value() == r.discriminant_value(),
        }
    }
}

impl OrdWithEngines for TypeInfo {
    fn cmp(&self, other: &Self, engines: &Engines) -> Ordering {
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
                    qualified_call_path: l_call_path,
                    type_arguments: l_type_args,
                    root_type_id: l_root_type_id,
                },
                Self::Custom {
                    qualified_call_path: r_call_path,
                    type_arguments: r_type_args,
                    root_type_id: r_root_type_id,
                },
            ) => l_call_path
                .call_path
                .suffix
                .cmp(&r_call_path.call_path.suffix)
                .then_with(|| {
                    l_call_path
                        .qualified_path_root
                        .cmp(&r_call_path.qualified_path_root, engines)
                })
                .then_with(|| l_type_args.as_deref().cmp(&r_type_args.as_deref(), engines))
                .then_with(|| l_root_type_id.cmp(r_root_type_id)),
            (Self::StringArray(l), Self::StringArray(r)) => l.val().cmp(&r.val()),
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
            (
                Self::TraitType {
                    name: l_name,
                    trait_type_id: l_trait_type_id,
                },
                Self::TraitType {
                    name: r_name,
                    trait_type_id: r_trait_type_id,
                },
            ) => l_trait_type_id
                .cmp(r_trait_type_id)
                .then_with(|| l_name.cmp(r_name)),

            (l, r) => l.discriminant_value().cmp(&r.discriminant_value()),
        }
    }
}

impl DisplayWithEngines for TypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        use TypeInfo::*;
        let s = match self {
            Unknown => "{unknown}".into(),
            UnknownGeneric { name, .. } => name.to_string(),
            Placeholder(type_param) => type_param.name_ident.to_string(),
            TypeParam(n) => format!("{n}"),
            StringSlice => "str".into(),
            StringArray(x) => format!("str[{}]", x.val()),
            UnsignedInteger(x) => match x {
                IntegerBits::Eight => "u8",
                IntegerBits::Sixteen => "u16",
                IntegerBits::ThirtyTwo => "u32",
                IntegerBits::SixtyFour => "u64",
                IntegerBits::V256 => "u256",
            }
            .into(),
            Boolean => "bool".into(),
            Custom {
                qualified_call_path: call_path,
                ..
            } => call_path.call_path.suffix.to_string(),
            Tuple(fields) => {
                let field_strs = fields
                    .iter()
                    .map(|field| engines.help_out(field).to_string())
                    .collect::<Vec<String>>();
                format!("({})", field_strs.join(", "))
            }
            B256 => "b256".into(),
            Numeric => "numeric".into(),
            Contract => "contract".into(),
            ErrorRecovery(_) => "unknown".into(),
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
            ContractCaller { abi_name, .. } => format!("ContractCaller<{abi_name}>"),
            Array(elem_ty, count) => {
                format!("[{}; {}]", engines.help_out(elem_ty), count.val())
            }
            Storage { .. } => "storage".into(),
            RawUntypedPtr => "pointer".into(),
            RawUntypedSlice => "slice".into(),
            Ptr(ty) => {
                format!("__ptr[{}]", engines.help_out(ty))
            }
            Slice(ty) => {
                format!("__slice[{}]", engines.help_out(ty))
            }
            Alias { name, .. } => name.to_string(),
            TraitType {
                name,
                trait_type_id,
            } => format!("trait type {}::{}", engines.help_out(trait_type_id), name),
        };
        write!(f, "{s}")
    }
}

impl DebugWithEngines for TypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        use TypeInfo::*;
        let s = match self {
            Unknown => "unknown".into(),
            UnknownGeneric { name, .. } => name.to_string(),
            Placeholder(t) => format!("placeholder({:?})", engines.help_out(t)),
            TypeParam(n) => format!("typeparam({n})"),
            StringSlice => "str".into(),
            StringArray(x) => format!("str[{}]", x.val()),
            UnsignedInteger(x) => match x {
                IntegerBits::Eight => "u8",
                IntegerBits::Sixteen => "u16",
                IntegerBits::ThirtyTwo => "u32",
                IntegerBits::SixtyFour => "u64",
                IntegerBits::V256 => "u256",
            }
            .into(),
            Boolean => "bool".into(),
            Custom {
                qualified_call_path: call_path,
                type_arguments,
                ..
            } => {
                let mut s = "".to_string();
                if let Some(type_arguments) = type_arguments {
                    if !type_arguments.is_empty() {
                        s = format!(
                            "<{}>",
                            type_arguments
                                .iter()
                                .map(|a| format!("{:?}", engines.help_out(a)))
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                    }
                }
                format!("unresolved {}{}", call_path.call_path, s)
            }
            Tuple(fields) => {
                let field_strs = fields
                    .iter()
                    .map(|field| format!("{:?}", engines.help_out(field)))
                    .collect::<Vec<String>>();
                format!("({})", field_strs.join(", "))
            }
            B256 => "b256".into(),
            Numeric => "numeric".into(),
            Contract => "contract".into(),
            ErrorRecovery(_) => "unknown due to error".into(),
            Enum(decl_ref) => {
                let decl = engines.de().get_enum(decl_ref);
                print_inner_types_debug(
                    engines,
                    decl.call_path.suffix.as_str().to_string(),
                    decl.type_parameters.iter().map(|x| x.type_id),
                )
            }
            Struct(decl_ref) => {
                let decl = engines.de().get_struct(decl_ref);
                print_inner_types_debug(
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
                format!("[{:?}; {}]", engines.help_out(elem_ty), count.val())
            }
            Storage { .. } => "contract storage".into(),
            RawUntypedPtr => "raw untyped ptr".into(),
            RawUntypedSlice => "raw untyped slice".into(),
            Ptr(ty) => {
                format!("__ptr[{:?}]", engines.help_out(ty))
            }
            Slice(ty) => {
                format!("__slice[{:?}]", engines.help_out(ty))
            }
            Alias { name, ty } => {
                format!("type {} = {:?}", name, engines.help_out(ty))
            }
            TraitType {
                name,
                trait_type_id,
            } => format!("trait type {}::{}", engines.help_out(trait_type_id), name),
        };
        write!(f, "{s}")
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
            TypeInfo::StringArray(_) => 3,
            TypeInfo::UnsignedInteger(_) => 4,
            TypeInfo::Enum { .. } => 5,
            TypeInfo::Struct { .. } => 6,
            TypeInfo::Boolean => 7,
            TypeInfo::Tuple(_) => 8,
            TypeInfo::ContractCaller { .. } => 9,
            TypeInfo::Custom { .. } => 10,
            TypeInfo::B256 => 11,
            TypeInfo::Numeric => 12,
            TypeInfo::Contract => 13,
            TypeInfo::ErrorRecovery(_) => 14,
            TypeInfo::Array(_, _) => 15,
            TypeInfo::Storage { .. } => 16,
            TypeInfo::RawUntypedPtr => 17,
            TypeInfo::RawUntypedSlice => 18,
            TypeInfo::TypeParam(_) => 19,
            TypeInfo::Alias { .. } => 20,
            TypeInfo::Ptr(..) => 21,
            TypeInfo::Slice(..) => 22,
            TypeInfo::StringSlice => 23,
            TypeInfo::TraitType { .. } => 24,
        }
    }

    pub(crate) fn new_self_type(span: Span) -> TypeInfo {
        TypeInfo::Custom {
            qualified_call_path: QualifiedCallPath {
                call_path: CallPath {
                    prefixes: vec![],
                    suffix: Ident::new_with_override("Self".into(), span),
                    is_absolute: false,
                },
                qualified_path_root: None,
            },
            type_arguments: None,
            root_type_id: None,
        }
    }

    pub(crate) fn is_self_type(&self) -> bool {
        match self {
            TypeInfo::UnknownGeneric { name, .. } => {
                name.as_str() == "Self" || name.as_str() == "self"
            }
            TypeInfo::Custom {
                qualified_call_path,
                ..
            } => {
                qualified_call_path.call_path.suffix.as_str() == "Self"
                    || qualified_call_path.call_path.suffix.as_str() == "self"
            }
            _ => false,
        }
    }

    /// maps a type to a name that is used when constructing function selectors
    pub(crate) fn to_selector_name(
        &self,
        handler: &Handler,
        engines: &Engines,
        error_msg_span: &Span,
    ) -> Result<String, ErrorEmitted> {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        use TypeInfo::*;
        let name = match self {
            StringArray(len) => format!("str[{}]", len.val()),
            UnsignedInteger(bits) => {
                use IntegerBits::*;
                match bits {
                    Eight => "u8",
                    Sixteen => "u16",
                    ThirtyTwo => "u32",
                    SixtyFour => "u64",
                    V256 => "u256",
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
                                .to_selector_name(handler, engines, error_msg_span)
                        })
                        .collect::<Vec<Result<String, _>>>();
                    let mut buf = vec![];
                    for name in names {
                        buf.push(name?);
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
                                Err(e) => return Err(handler.emit_err(e.into())),
                                Ok(ty) => ty,
                            };
                            ty.to_selector_name(handler, engines, error_msg_span)
                        })
                        .collect::<Vec<Result<String, _>>>();
                    let mut buf = vec![];
                    for name in names {
                        match name {
                            Ok(value) => buf.push(value),
                            Err(e) => return Err(e),
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
                                Err(e) => return Err(handler.emit_err(e.into())),
                                Ok(ty) => ty,
                            };
                            ty.to_selector_name(handler, engines, error_msg_span)
                        })
                        .collect::<Vec<Result<String, _>>>();
                    let mut buf = vec![];
                    for arg in type_arguments {
                        match arg {
                            Ok(value) => buf.push(value),
                            Err(e) => return Err(e),
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
                                Err(e) => return Err(handler.emit_err(e.into())),
                                Ok(ty) => ty,
                            };
                            ty.to_selector_name(handler, engines, error_msg_span)
                        })
                        .collect::<Vec<Result<String, _>>>();
                    let mut buf = vec![];
                    for name in names {
                        buf.push(name?);
                    }
                    buf
                };

                let type_arguments = {
                    let type_arguments = decl
                        .type_parameters
                        .iter()
                        .map(|ty| {
                            let ty = match type_engine.to_typeinfo(ty.type_id, error_msg_span) {
                                Err(e) => return Err(handler.emit_err(e.into())),
                                Ok(ty) => ty,
                            };
                            ty.to_selector_name(handler, engines, error_msg_span)
                        })
                        .collect::<Vec<Result<String, _>>>();
                    let mut buf = vec![];
                    for arg in type_arguments {
                        buf.push(arg?);
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
                    handler,
                    engines,
                    error_msg_span,
                );
                let name = match name {
                    Ok(name) => name,
                    Err(e) => return Err(e),
                };
                format!("a[{};{}]", name, length.val())
            }
            RawUntypedPtr => "rawptr".to_string(),
            RawUntypedSlice => "rawslice".to_string(),
            Alias { ty, .. } => {
                let name =
                    type_engine
                        .get(ty.type_id)
                        .to_selector_name(handler, engines, error_msg_span);
                name?
            }
            _ => {
                return Err(handler.emit_err(CompileError::InvalidAbiType {
                    span: error_msg_span.clone(),
                }));
            }
        };
        Ok(name)
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
            TypeInfo::ErrorRecovery(_) => true,
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
        // XXX This is FuelVM specific.  We need to find the users of this method and determine
        // whether they're actually asking 'is_aggregate()` or something else.
        matches!(
            self,
            TypeInfo::Boolean
                | TypeInfo::UnsignedInteger(IntegerBits::Eight)
                | TypeInfo::UnsignedInteger(IntegerBits::Sixteen)
                | TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo)
                | TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)
                | TypeInfo::RawUntypedPtr
                | TypeInfo::Numeric
        ) || self.is_unit()
    }

    pub fn is_aggregate_type(&self) -> bool {
        match self {
            TypeInfo::Struct { .. } | TypeInfo::Enum { .. } | TypeInfo::Array { .. } => true,
            TypeInfo::Tuple { .. } => !self.is_unit(),
            _ => false,
        }
    }

    pub(crate) fn apply_type_arguments(
        self,
        handler: &Handler,
        type_arguments: Vec<TypeArgument>,
        span: &Span,
    ) -> Result<TypeInfo, ErrorEmitted> {
        if type_arguments.is_empty() {
            return Ok(self);
        }
        match self {
            TypeInfo::Enum { .. } | TypeInfo::Struct { .. } => {
                Err(handler.emit_err(CompileError::Internal(
                    "did not expect to apply type arguments to this type",
                    span.clone(),
                )))
            }
            TypeInfo::Custom {
                qualified_call_path: call_path,
                type_arguments: other_type_arguments,
                root_type_id,
            } => {
                if other_type_arguments.is_some() {
                    Err(handler
                        .emit_err(CompileError::TypeArgumentsNotAllowed { span: span.clone() }))
                } else {
                    let type_info = TypeInfo::Custom {
                        qualified_call_path: call_path,
                        type_arguments: Some(type_arguments),
                        root_type_id,
                    };
                    Ok(type_info)
                }
            }
            TypeInfo::Unknown
            | TypeInfo::UnknownGeneric { .. }
            | TypeInfo::StringArray(_)
            | TypeInfo::StringSlice
            | TypeInfo::UnsignedInteger(_)
            | TypeInfo::Boolean
            | TypeInfo::Tuple(_)
            | TypeInfo::ContractCaller { .. }
            | TypeInfo::B256
            | TypeInfo::Numeric
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice
            | TypeInfo::Ptr(..)
            | TypeInfo::Slice(..)
            | TypeInfo::Contract
            | TypeInfo::ErrorRecovery(_)
            | TypeInfo::Array(_, _)
            | TypeInfo::Storage { .. }
            | TypeInfo::Placeholder(_)
            | TypeInfo::TypeParam(_)
            | TypeInfo::Alias { .. }
            | TypeInfo::TraitType { .. } => {
                Err(handler.emit_err(CompileError::TypeArgumentsNotAllowed { span: span.clone() }))
            }
        }
    }

    /// Given a `TypeInfo` `self`, check to see if `self` is currently
    /// supported in match expressions, and return an error if it is not.
    pub(crate) fn expect_is_supported_in_match_expressions(
        &self,
        handler: &Handler,
        span: &Span,
    ) -> Result<(), ErrorEmitted> {
        match self {
            TypeInfo::UnsignedInteger(_)
            | TypeInfo::Enum { .. }
            | TypeInfo::Struct { .. }
            | TypeInfo::Boolean
            | TypeInfo::Tuple(_)
            | TypeInfo::B256
            | TypeInfo::UnknownGeneric { .. }
            | TypeInfo::Numeric
            | TypeInfo::Alias { .. } => Ok(()),
            TypeInfo::Unknown
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice
            | TypeInfo::Ptr(..)
            | TypeInfo::Slice(..)
            | TypeInfo::ContractCaller { .. }
            | TypeInfo::Custom { .. }
            | TypeInfo::StringArray(_)
            | TypeInfo::StringSlice
            | TypeInfo::Contract
            | TypeInfo::Array(_, _)
            | TypeInfo::Storage { .. }
            | TypeInfo::Placeholder(_)
            | TypeInfo::TypeParam(_)
            | TypeInfo::TraitType { .. } => Err(handler.emit_err(CompileError::Unimplemented(
                "matching on this type is unsupported right now",
                span.clone(),
            ))),
            TypeInfo::ErrorRecovery(err) => Err(*err),
        }
    }

    /// Given a `TypeInfo` `self`, check to see if `self` is currently
    /// supported in `impl` blocks in the "type implementing for" position.
    pub(crate) fn expect_is_supported_in_impl_blocks_self(
        &self,
        handler: &Handler,
        span: &Span,
    ) -> Result<(), ErrorEmitted> {
        if TypeInfo::is_self_type(self) {
            return Err(handler
                .emit_err(CompileError::SelfIsNotValidAsImplementingFor { span: span.clone() }));
        }
        match self {
            TypeInfo::UnsignedInteger(_)
            | TypeInfo::Enum { .. }
            | TypeInfo::Struct { .. }
            | TypeInfo::Boolean
            | TypeInfo::Tuple(_)
            | TypeInfo::B256
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice
            | TypeInfo::Ptr(_)
            | TypeInfo::Slice(_)
            | TypeInfo::Custom { .. }
            | TypeInfo::StringArray(_)
            | TypeInfo::StringSlice
            | TypeInfo::Array(_, _)
            | TypeInfo::Contract
            | TypeInfo::Numeric
            | TypeInfo::Alias { .. }
            | TypeInfo::UnknownGeneric { .. }
            | TypeInfo::TraitType { .. } => Ok(()),
            TypeInfo::Unknown
            | TypeInfo::ContractCaller { .. }
            | TypeInfo::Storage { .. }
            | TypeInfo::Placeholder(_)
            | TypeInfo::TypeParam(_) => Err(handler.emit_err(CompileError::Unimplemented(
                "implementing traits on this type is unsupported right now",
                span.clone(),
            ))),
            TypeInfo::ErrorRecovery(err) => Err(*err),
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
        handler: &Handler,
        engines: &Engines,
        subfields: &[Ident],
        span: &Span,
    ) -> Result<ty::TyStructField, ErrorEmitted> {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        match (self, subfields.split_first()) {
            (TypeInfo::Struct { .. } | TypeInfo::Alias { .. }, None) => {
                panic!("Trying to apply an empty list of subfields");
            }
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
                        return Err(handler.emit_err(CompileError::FieldNotFound {
                            field_name: first.clone(),
                            struct_name: decl.call_path.suffix.clone(),
                            available_fields: available_fields.join(", "),
                            span: first.span(),
                        }));
                    }
                };
                let field = if rest.is_empty() {
                    field
                } else {
                    type_engine
                        .get(field.type_argument.type_id)
                        .apply_subfields(handler, engines, rest, span)?
                };
                Ok(field)
            }
            (
                TypeInfo::Alias {
                    ty: TypeArgument { type_id, .. },
                    ..
                },
                _,
            ) => type_engine
                .get(*type_id)
                .apply_subfields(handler, engines, subfields, span),
            (TypeInfo::ErrorRecovery(err), _) => Err(*err),
            (type_info, _) => Err(handler.emit_err(CompileError::FieldAccessOnNonStruct {
                actually: format!("{:?}", engines.help_out(type_info)),
                span: span.clone(),
            })),
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
            TypeInfo::StringArray(_)
            | TypeInfo::StringSlice
            | TypeInfo::UnsignedInteger(_)
            | TypeInfo::Boolean
            | TypeInfo::B256
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice
            | TypeInfo::Ptr(..)
            | TypeInfo::Slice(..)
            | TypeInfo::ErrorRecovery(_)
            | TypeInfo::TraitType { .. } => false,
            TypeInfo::Unknown
            | TypeInfo::UnknownGeneric { .. }
            | TypeInfo::ContractCaller { .. }
            | TypeInfo::Custom { .. }
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
        handler: &Handler,
        engines: &Engines,
        debug_string: impl Into<String>,
        debug_span: &Span,
    ) -> Result<Vec<TypeArgument>, ErrorEmitted> {
        match self {
            TypeInfo::Tuple(elems) => Ok(elems.to_vec()),
            TypeInfo::Alias {
                ty: TypeArgument { type_id, .. },
                ..
            } => {
                engines
                    .te()
                    .get(*type_id)
                    .expect_tuple(handler, engines, debug_string, debug_span)
            }
            TypeInfo::ErrorRecovery(err) => Err(*err),
            a => Err(handler.emit_err(CompileError::NotATuple {
                name: debug_string.into(),
                span: debug_span.clone(),
                actually: engines.help_out(a).to_string(),
            })),
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
        handler: &Handler,
        engines: &Engines,
        debug_string: impl Into<String>,
        debug_span: &Span,
    ) -> Result<DeclRefEnum, ErrorEmitted> {
        match self {
            TypeInfo::Enum(decl_ref) => Ok(decl_ref.clone()),
            TypeInfo::Alias {
                ty: TypeArgument { type_id, .. },
                ..
            } => engines
                .te()
                .get(*type_id)
                .expect_enum(handler, engines, debug_string, debug_span),
            TypeInfo::ErrorRecovery(err) => Err(*err),
            a => Err(handler.emit_err(CompileError::NotAnEnum {
                name: debug_string.into(),
                span: debug_span.clone(),
                actually: engines.help_out(a).to_string(),
            })),
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
        handler: &Handler,
        engines: &Engines,
        debug_span: &Span,
    ) -> Result<DeclRefStruct, ErrorEmitted> {
        match self {
            TypeInfo::Struct(decl_ref) => Ok(decl_ref.clone()),
            TypeInfo::Alias {
                ty: TypeArgument { type_id, .. },
                ..
            } => engines
                .te()
                .get(*type_id)
                .expect_struct(handler, engines, debug_span),
            TypeInfo::ErrorRecovery(err) => Err(*err),
            a => Err(handler.emit_err(CompileError::NotAStruct {
                span: debug_span.clone(),
                actually: engines.help_out(a).to_string(),
            })),
        }
    }
}

fn print_inner_types(
    engines: &Engines,
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

fn print_inner_types_debug(
    engines: &Engines,
    name: String,
    inner_types: impl Iterator<Item = TypeId>,
) -> String {
    let inner_types = inner_types
        .map(|x| format!("{:?}", engines.help_out(x)))
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

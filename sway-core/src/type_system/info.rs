use crate::{
    decl_engine::{DeclEngine, DeclEngineGet, DeclRefEnum, DeclRefStruct},
    engine_threading::{
        DebugWithEngines, DisplayWithEngines, Engines, EqWithEngines, HashWithEngines,
        OrdWithEngines, OrdWithEnginesContext, PartialEqWithEngines, PartialEqWithEnginesContext,
    },
    language::{ty, CallPath, QualifiedCallPath},
    type_system::priv_prelude::*,
    Ident,
};
use sway_error::{
    error::{CompileError, InvalidImplementingForType},
    handler::{ErrorEmitted, Handler},
};
use sway_types::{integer_bits::IntegerBits, span::Span, SourceId};

use std::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
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
    pub fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.0.len() <= other.0.len() && self.0.iter().all(|x| other.0.iter().any(|y| x.eq(y, ctx)))
    }
}

impl<T: PartialEqWithEngines> PartialEqWithEngines for VecSet<T> {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.eq(other, ctx) && other.eq(self, ctx)
    }
}

/// Encapsulates type information and its optional source identifier.
#[derive(Debug, Default, Clone)]
pub struct TypeSourceInfo {
    pub(crate) type_info: Arc<TypeInfo>,
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
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.type_info.eq(&other.type_info, ctx) && self.source_id == other.source_id
    }
}

/// Type information without an associated value, used for type inferencing and definition.
#[derive(Debug, Clone, Default)]
pub enum TypeInfo {
    #[default]
    Unknown,
    Never,
    /// Represents a type parameter.
    ///
    /// The equivalent type in the Rust compiler is:
    /// https://doc.rust-lang.org/nightly/nightly-rustc/src/rustc_type_ir/sty.rs.html#190
    UnknownGeneric {
        name: Ident,
        // NOTE(Centril): Used to be BTreeSet; need to revert back later. Must be sorted!
        trait_constraints: VecSet<TraitConstraint>,
        // Methods can have type parameters with unknown generic that extend the trait constraints of a parent unknown generic.
        parent: Option<TypeId>,
        // This is true when the UnknownGeneric is used in a type parameter.
        is_from_type_parameter: bool,
    },
    /// Represents a type that will be inferred by the Sway compiler. This type
    /// is created when the user writes code that creates a new ADT that has
    /// type parameters in it's definition, before type inference can determine
    /// what the type of those type parameters are.
    ///
    /// This type would also be created in a case where the user wrote a type
    /// annotation with a wildcard type, like:
    /// `let v: Vec<_> = iter.collect();`.
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
    Ref {
        to_mutable_value: bool,
        referenced_type: TypeArgument,
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
                parent: _,
                is_from_type_parameter: _,
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
            TypeInfo::Ref {
                to_mutable_value,
                referenced_type: ty,
            } => {
                to_mutable_value.hash(state);
                ty.hash(state, engines);
            }
            TypeInfo::StringSlice
            | TypeInfo::Numeric
            | TypeInfo::Boolean
            | TypeInfo::B256
            | TypeInfo::Contract
            | TypeInfo::ErrorRecovery(_)
            | TypeInfo::Unknown
            | TypeInfo::Never
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice => {}
        }
    }
}

impl EqWithEngines for TypeInfo {}
impl PartialEqWithEngines for TypeInfo {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        let type_engine = ctx.engines().te();
        match (self, other) {
            (
                Self::UnknownGeneric {
                    name: l,
                    trait_constraints: ltc,
                    parent: _,
                    is_from_type_parameter: _,
                },
                Self::UnknownGeneric {
                    name: r,
                    trait_constraints: rtc,
                    parent: _,
                    is_from_type_parameter: _,
                },
            ) => l == r && ltc.eq(rtc, ctx),
            (Self::Placeholder(l), Self::Placeholder(r)) => l.eq(r, ctx),
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
                        .eq(&r_name.qualified_path_root, ctx)
                    && l_type_args.as_deref().eq(&r_type_args.as_deref(), ctx)
                    && l_root_type_id.eq(r_root_type_id)
            }
            (Self::StringSlice, Self::StringSlice) => true,
            (Self::StringArray(l), Self::StringArray(r)) => l.val() == r.val(),
            (Self::UnsignedInteger(l), Self::UnsignedInteger(r)) => l == r,
            (Self::Enum(l_decl_ref), Self::Enum(r_decl_ref)) => {
                let l_decl = ctx.engines().de().get_enum(l_decl_ref);
                let r_decl = ctx.engines().de().get_enum(r_decl_ref);
                assert!(
                    l_decl.call_path.is_absolute && r_decl.call_path.is_absolute,
                    "The call paths of the enum declarations must always be absolute."
                );
                l_decl.call_path == r_decl.call_path
                    && l_decl.variants.eq(&r_decl.variants, ctx)
                    && l_decl.type_parameters.eq(&r_decl.type_parameters, ctx)
            }
            (Self::Struct(l_decl_ref), Self::Struct(r_decl_ref)) => {
                let l_decl = ctx.engines().de().get_struct(l_decl_ref);
                let r_decl = ctx.engines().de().get_struct(r_decl_ref);
                assert!(
                    l_decl.call_path.is_absolute && r_decl.call_path.is_absolute,
                    "The call paths of the struct declarations must always be absolute."
                );
                l_decl.call_path == r_decl.call_path
                    && l_decl.fields.eq(&r_decl.fields, ctx)
                    && l_decl.type_parameters.eq(&r_decl.type_parameters, ctx)
            }
            (Self::Tuple(l), Self::Tuple(r)) => l
                .iter()
                .zip(r.iter())
                .map(|(l, r)| {
                    (l.type_id == r.type_id)
                        || type_engine
                            .get(l.type_id)
                            .eq(&type_engine.get(r.type_id), ctx)
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
            ) => l_abi_name == r_abi_name && l_address.as_deref().eq(&r_address.as_deref(), ctx),
            (Self::Array(l0, l1), Self::Array(r0, r1)) => {
                ((l0.type_id == r0.type_id)
                    || type_engine
                        .get(l0.type_id)
                        .eq(&type_engine.get(r0.type_id), ctx))
                    && l1.val() == r1.val()
            }
            (Self::Storage { fields: l_fields }, Self::Storage { fields: r_fields }) => {
                l_fields.eq(r_fields, ctx)
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
                    && ((l_ty.type_id == r_ty.type_id)
                        || type_engine
                            .get(l_ty.type_id)
                            .eq(&type_engine.get(r_ty.type_id), ctx))
            }
            (
                Self::TraitType {
                    name: l_name,
                    trait_type_id: l_trait_type_id,
                },
                Self::TraitType {
                    name: r_name,
                    trait_type_id: r_trait_type_id,
                },
            ) => {
                l_name == r_name
                    && ((*l_trait_type_id == *r_trait_type_id)
                        || type_engine
                            .get(*l_trait_type_id)
                            .eq(&type_engine.get(*r_trait_type_id), ctx))
            }
            (
                Self::Ref {
                    to_mutable_value: l_to_mut,
                    referenced_type: l_ty,
                },
                Self::Ref {
                    to_mutable_value: r_to_mut,
                    referenced_type: r_ty,
                },
            ) => {
                (l_to_mut == r_to_mut)
                    && ((l_ty.type_id == r_ty.type_id)
                        || type_engine
                            .get(l_ty.type_id)
                            .eq(&type_engine.get(r_ty.type_id), ctx))
            }

            (l, r) => l.discriminant_value() == r.discriminant_value(),
        }
    }
}

impl OrdWithEngines for TypeInfo {
    fn cmp(&self, other: &Self, ctx: &OrdWithEnginesContext) -> Ordering {
        let type_engine = ctx.engines().te();
        let decl_engine = ctx.engines().de();
        match (self, other) {
            (
                Self::UnknownGeneric {
                    name: l,
                    trait_constraints: ltc,
                    parent: _,
                    is_from_type_parameter: _,
                },
                Self::UnknownGeneric {
                    name: r,
                    trait_constraints: rtc,
                    parent: _,
                    is_from_type_parameter: _,
                },
            ) => l.cmp(r).then_with(|| ltc.cmp(rtc, ctx)),
            (Self::Placeholder(l), Self::Placeholder(r)) => l.cmp(r, ctx),
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
                        .cmp(&r_call_path.qualified_path_root, ctx)
                })
                .then_with(|| l_type_args.as_deref().cmp(&r_type_args.as_deref(), ctx))
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
                    .then_with(|| l_decl.type_parameters.cmp(&r_decl.type_parameters, ctx))
                    .then_with(|| l_decl.variants.cmp(&r_decl.variants, ctx))
            }
            (Self::Struct(l_decl_ref), Self::Struct(r_decl_ref)) => {
                let l_decl = decl_engine.get_struct(l_decl_ref);
                let r_decl = decl_engine.get_struct(r_decl_ref);
                l_decl
                    .call_path
                    .suffix
                    .cmp(&r_decl.call_path.suffix)
                    .then_with(|| l_decl.type_parameters.cmp(&r_decl.type_parameters, ctx))
                    .then_with(|| l_decl.fields.cmp(&r_decl.fields, ctx))
            }
            (Self::Tuple(l), Self::Tuple(r)) => l.cmp(r, ctx),
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
                .cmp(&type_engine.get(r0.type_id), ctx)
                .then_with(|| l1.val().cmp(&r1.val())),
            (TypeInfo::Storage { fields: l_fields }, TypeInfo::Storage { fields: r_fields }) => {
                l_fields.cmp(r_fields, ctx)
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
                .cmp(&type_engine.get(r_ty.type_id), ctx)
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
            (
                Self::Ref {
                    to_mutable_value: l_to_mut,
                    referenced_type: l_ty,
                },
                Self::Ref {
                    to_mutable_value: r_to_mut,
                    referenced_type: r_ty,
                },
            ) => l_to_mut.cmp(r_to_mut).then_with(|| {
                type_engine
                    .get(l_ty.type_id)
                    .cmp(&type_engine.get(r_ty.type_id), ctx)
            }),
            (l, r) => l.discriminant_value().cmp(&r.discriminant_value()),
        }
    }
}

impl DisplayWithEngines for TypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        use TypeInfo::*;
        let s = match self {
            Unknown => "{unknown}".into(),
            Never => "!".into(),
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
                    decl.call_path.suffix.as_str(),
                    decl.type_parameters.iter().map(|x| x.type_id),
                )
            }
            Struct(decl_ref) => {
                let decl = engines.de().get_struct(decl_ref);
                print_inner_types(
                    engines,
                    decl.call_path.suffix.as_str(),
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
            Ref {
                to_mutable_value,
                referenced_type: ty,
            } => {
                format!(
                    "&{}{}",
                    if *to_mutable_value { "mut " } else { "" },
                    engines.help_out(ty)
                )
            }
        };
        write!(f, "{s}")
    }
}

impl DebugWithEngines for TypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        use TypeInfo::{
            Alias, Array, Boolean, Contract, ContractCaller, Custom, Enum, ErrorRecovery, Never,
            Numeric, Placeholder, Ptr, RawUntypedPtr, RawUntypedSlice, Ref, Slice, Storage,
            StringArray, StringSlice, Struct, TraitType, Tuple, TypeParam, Unknown, UnknownGeneric,
            UnsignedInteger, B256,
        };
        let s = match self {
            Unknown => "unknown".into(),
            Never => "!".into(),
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
                let mut s = String::new();
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
                    decl.call_path.suffix.as_str(),
                    decl.type_parameters.iter().map(|x| x.type_id),
                )
            }
            Struct(decl_ref) => {
                let decl = engines.de().get_struct(decl_ref);
                print_inner_types_debug(
                    engines,
                    decl.call_path.suffix.as_str(),
                    decl.type_parameters.iter().map(|x| x.type_id),
                )
            }
            ContractCaller { abi_name, address } => {
                format!(
                    "contract caller {} ( {} )",
                    abi_name,
                    address.as_ref().map_or_else(
                        || "None".into(),
                        |address| address.span.as_str().to_string()
                    )
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
            Ref {
                to_mutable_value,
                referenced_type: ty,
            } => {
                format!(
                    "&{}{:?}",
                    if *to_mutable_value { "mut " } else { "" },
                    engines.help_out(ty)
                )
            }
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
            TypeInfo::Ref { .. } => 25,
            TypeInfo::Never => 26,
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

    pub(crate) fn is_bool(&self) -> bool {
        matches!(self, TypeInfo::Boolean)
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
        use TypeInfo::{
            Alias, Array, Boolean, Enum, RawUntypedPtr, RawUntypedSlice, StringArray, Struct,
            Tuple, UnsignedInteger, B256,
        };
        let name = match self {
            StringArray(len) => format!("str[{}]", len.val()),
            UnsignedInteger(bits) => {
                use IntegerBits::{Eight, Sixteen, SixtyFour, ThirtyTwo, V256};
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
            // TODO-IG: No references in ABIs according to the RFC. Or we want to have them?
            // TODO-IG: Depending on that, we need to handle `Ref` here as well.
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
            TypeInfo::Never => true,
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
            TypeInfo::Ptr(ty) => id_uninhabited(ty.type_id),
            TypeInfo::Alias { name: _, ty } => id_uninhabited(ty.type_id),
            TypeInfo::Slice(ty) => id_uninhabited(ty.type_id),
            TypeInfo::Ref {
                to_mutable_value: _,
                referenced_type,
            } => id_uninhabited(referenced_type.type_id),
            _ => false,
        }
    }

    pub fn is_zero_sized(&self, type_engine: &TypeEngine, decl_engine: &DeclEngine) -> bool {
        match self {
            TypeInfo::Enum(decl_ref) => {
                let decl = decl_engine.get_enum(decl_ref);
                let mut found_unit_variant = false;
                for variant_type in &decl.variants {
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
                for field in &decl.fields {
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
            TypeInfo::Never => true,
            _ => false,
        }
    }

    // TODO-IG: Check all the usages of `is_copy_type`.
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
                | TypeInfo::Numeric // TODO-IG: Should Ptr and Ref also be a copy type?
                | TypeInfo::Never
        ) || self.is_unit()
    }

    pub fn is_aggregate_type(&self) -> bool {
        match self {
            TypeInfo::Struct { .. } | TypeInfo::Enum { .. } | TypeInfo::Array { .. } => true,
            TypeInfo::Tuple { .. } => !self.is_unit(),
            _ => false,
        }
    }

    pub fn is_unit(&self) -> bool {
        match self {
            TypeInfo::Tuple(fields) => fields.is_empty(),
            _ => false,
        }
    }

    pub fn is_reference(&self) -> bool {
        matches!(self, TypeInfo::Ref { .. })
    }

    pub fn is_array(&self) -> bool {
        matches!(self, TypeInfo::Array(_, _))
    }

    pub fn is_contract(&self) -> bool {
        matches!(self, TypeInfo::Contract)
    }

    pub fn is_struct(&self) -> bool {
        matches!(self, TypeInfo::Struct(_))
    }

    pub fn is_tuple(&self) -> bool {
        matches!(self, TypeInfo::Tuple(_))
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
            | TypeInfo::Never
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
            | TypeInfo::Ptr(_)
            | TypeInfo::Slice(_)
            | TypeInfo::Contract
            | TypeInfo::ErrorRecovery(_)
            | TypeInfo::Array(_, _)
            | TypeInfo::Storage { .. }
            | TypeInfo::Placeholder(_)
            | TypeInfo::TypeParam(_)
            | TypeInfo::Alias { .. }
            | TypeInfo::TraitType { .. }
            | TypeInfo::Ref { .. } => {
                Err(handler.emit_err(CompileError::TypeArgumentsNotAllowed { span: span.clone() }))
            }
        }
    }

    /// Given a [TypeInfo] `self`, check to see if `self` is currently
    /// supported as a match expression's matched value, and return an error if it is not.
    pub(crate) fn expect_is_supported_in_match_expressions(
        &self,
        handler: &Handler,
        engines: &Engines,
        span: &Span,
    ) -> Result<(), ErrorEmitted> {
        const CURRENTLY_SUPPORTED_TYPES_MESSAGE: [&str; 8] = [
            "Sway currently supports pattern matching on these types:",
            "  - b256",
            "  - boolean",
            "  - enums",
            "  - structs",
            "  - tuples",
            "  - unsigned integers",
            "  - Never type (`!`)",
        ];

        match self {
            TypeInfo::UnsignedInteger(_)
            | TypeInfo::Enum { .. }
            | TypeInfo::Struct { .. }
            | TypeInfo::Boolean
            | TypeInfo::Tuple(_)
            | TypeInfo::B256
            | TypeInfo::UnknownGeneric { .. }
            | TypeInfo::Numeric
            | TypeInfo::Never => Ok(()),
            TypeInfo::Alias { ty, .. } => {
                let ty = engines.te().get(ty.type_id);
                ty.expect_is_supported_in_match_expressions(handler, engines, span)
            }
            TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice
            | TypeInfo::Ptr(..)
            | TypeInfo::Slice(..)
            | TypeInfo::StringArray(_)
            | TypeInfo::StringSlice
            | TypeInfo::Array(_, _) => Err(handler.emit_err(CompileError::Unimplemented {
                feature: format!(
                    "Matched value has type \"{}\". Matching on this type",
                    engines.help_out(self)
                ),
                help: {
                    let mut help = vec![];
                    for line in CURRENTLY_SUPPORTED_TYPES_MESSAGE {
                        help.push(line.to_string());
                    }
                    help
                },
                span: span.clone(),
            })),
            TypeInfo::Ref { .. } => Err(handler.emit_err(CompileError::Unimplemented {
                // TODO-IG: Implement.
                feature: "Using references in match expressions".to_string(),
                help: vec![],
                span: span.clone(),
            })),
            TypeInfo::ErrorRecovery(err) => Err(*err),
            TypeInfo::Unknown
            | TypeInfo::ContractCaller { .. }
            | TypeInfo::Custom { .. }
            | TypeInfo::Contract
            | TypeInfo::Storage { .. }
            | TypeInfo::Placeholder(_)
            | TypeInfo::TypeParam(_)
            | TypeInfo::TraitType { .. } => {
                Err(handler.emit_err(CompileError::MatchedValueIsNotValid {
                    supported_types_message: CURRENTLY_SUPPORTED_TYPES_MESSAGE
                        .into_iter()
                        .collect(),
                    span: span.clone(),
                }))
            }
        }
    }

    /// Given a [TypeInfo] `self`, check to see if `self` is currently
    /// supported in `impl` blocks in the "type implementing for" position.
    pub(crate) fn expect_is_supported_in_impl_blocks_self(
        &self,
        handler: &Handler,
        trait_name: Option<&Ident>,
        span: &Span,
    ) -> Result<(), ErrorEmitted> {
        if TypeInfo::is_self_type(self) {
            return Err(
                handler.emit_err(CompileError::TypeIsNotValidAsImplementingFor {
                    invalid_type: InvalidImplementingForType::SelfType,
                    trait_name: trait_name.map(|name| name.to_string()),
                    span: span.clone(),
                }),
            );
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
            | TypeInfo::TraitType { .. }
            | TypeInfo::Ref { .. }
            | TypeInfo::Never => Ok(()),
            TypeInfo::Unknown if span.as_str() == "_" => Err(handler.emit_err(
                CompileError::TypeIsNotValidAsImplementingFor {
                    invalid_type: InvalidImplementingForType::Placeholder,
                    trait_name: trait_name.map(|name| name.to_string()),
                    span: span.clone(),
                },
            )),
            TypeInfo::Unknown
            | TypeInfo::ContractCaller { .. }
            | TypeInfo::Storage { .. }
            | TypeInfo::Placeholder(_)
            | TypeInfo::TypeParam(_) => Err(handler.emit_err(
                CompileError::TypeIsNotValidAsImplementingFor {
                    invalid_type: InvalidImplementingForType::Other,
                    trait_name: trait_name.map(|name| name.to_string()),
                    span: span.clone(),
                },
            )),
            TypeInfo::ErrorRecovery(err) => Err(*err),
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
            | TypeInfo::Ptr(_)
            | TypeInfo::Slice(_)
            | TypeInfo::ErrorRecovery(_)
            | TypeInfo::TraitType { .. }
            | TypeInfo::Never => false,
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
            | TypeInfo::Alias { .. }
            | TypeInfo::Ref { .. } => true,
        }
    }

    /// Checks if a given [TypeInfo] has a valid constructor.
    pub(crate) fn has_valid_constructor(&self, decl_engine: &DeclEngine) -> bool {
        match self {
            TypeInfo::Unknown => false,
            TypeInfo::Never => false,
            TypeInfo::Enum(decl_ref) => {
                let decl = decl_engine.get_enum(decl_ref);
                !decl.variants.is_empty()
            }
            _ => true,
        }
    }

    /// Given a [TypeInfo] `self`, expect that `self` is a [TypeInfo::Enum], or a [TypeInfo::Alias]
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
    /// Returns an error if `self` is not a [TypeInfo::Enum] or a [TypeInfo::Alias] of a enum type,
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

    /// Given a [TypeInfo] `self`, expect that `self` is a [TypeInfo::Struct], or a
    /// [TypeInfo::Alias] of a struct type. Also, return the contents of the struct.
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
    /// Returns an error if `self` is not a [TypeInfo::Struct] or a [TypeInfo::Alias] of a struct
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

    pub fn is_unknown_generic(&self) -> bool {
        matches!(self, TypeInfo::UnknownGeneric { .. })
    }

    /// Calculate the needed buffer for "abi encoding" the self type. If "inside" this
    /// type there is a custom AbiEncode impl, we cannot calculate the buffer size.
    pub fn abi_encode_size_hint(&self, engines: &Engines) -> AbiEncodeSizeHint {
        // TODO we need to check if this type has a custom AbiEncode impl or not
        // https://github.com/FuelLabs/sway/issues/5727
        // if has_custom_abi_encode_impl {
        //     AbiEncodeSizeHint::CustomImpl
        // }

        match self {
            TypeInfo::Boolean => AbiEncodeSizeHint::Exact(1),
            TypeInfo::UnsignedInteger(IntegerBits::Eight) => AbiEncodeSizeHint::Exact(1),
            TypeInfo::UnsignedInteger(IntegerBits::Sixteen) => AbiEncodeSizeHint::Exact(2),
            TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo) => AbiEncodeSizeHint::Exact(4),
            TypeInfo::UnsignedInteger(IntegerBits::SixtyFour) => AbiEncodeSizeHint::Exact(8),
            // TODO: We should not be receiving Numeric here. All uints
            // should be correctly typed here.
            // https://github.com/FuelLabs/sway/issues/5727
            TypeInfo::Numeric => AbiEncodeSizeHint::Exact(8),
            TypeInfo::UnsignedInteger(IntegerBits::V256) => AbiEncodeSizeHint::Exact(32),
            TypeInfo::B256 => AbiEncodeSizeHint::Exact(32),

            TypeInfo::Slice(_) => AbiEncodeSizeHint::PotentiallyInfinite,
            TypeInfo::RawUntypedSlice => AbiEncodeSizeHint::PotentiallyInfinite,
            TypeInfo::StringSlice => AbiEncodeSizeHint::PotentiallyInfinite,
            TypeInfo::RawUntypedPtr => AbiEncodeSizeHint::PotentiallyInfinite,
            TypeInfo::Ptr(_) => AbiEncodeSizeHint::PotentiallyInfinite,

            TypeInfo::Alias { ty, .. } => {
                let elem_type = engines.te().get(ty.type_id);
                elem_type.abi_encode_size_hint(engines)
            }

            TypeInfo::Array(elem, len) => {
                let elem_type = engines.te().get(elem.type_id);
                let size_hint = elem_type.abi_encode_size_hint(engines);
                size_hint * len.val()
            }

            TypeInfo::StringArray(len) => AbiEncodeSizeHint::Exact(len.val()),

            TypeInfo::Tuple(items) => {
                items
                    .iter()
                    .fold(AbiEncodeSizeHint::Exact(0), |old_size_hint, t| {
                        let field_type = engines.te().get(t.type_id);
                        let field_size_hint = field_type.abi_encode_size_hint(engines);
                        old_size_hint + field_size_hint
                    })
            }

            TypeInfo::Struct(s) => {
                let decl = engines.de().get(s.id());
                decl.fields
                    .iter()
                    .fold(AbiEncodeSizeHint::Exact(0), |old_size_hint, f| {
                        let field_type = engines.te().get(f.type_argument.type_id);
                        let field_size_hint = field_type.abi_encode_size_hint(engines);
                        old_size_hint + field_size_hint
                    })
            }
            TypeInfo::Enum(e) => {
                let decl = engines.de().get(e.id());

                let min = decl
                    .variants
                    .iter()
                    .fold(None, |old_size_hint: Option<AbiEncodeSizeHint>, v| {
                        let variant_type = engines.te().get(v.type_argument.type_id);
                        let current_size_hint = variant_type.abi_encode_size_hint(engines);
                        match old_size_hint {
                            Some(old_size_hint) => Some(old_size_hint.min(current_size_hint)),
                            None => Some(current_size_hint),
                        }
                    })
                    .unwrap_or(AbiEncodeSizeHint::Exact(0));

                let max =
                    decl.variants
                        .iter()
                        .fold(AbiEncodeSizeHint::Exact(0), |old_size_hint, v| {
                            let variant_type = engines.te().get(v.type_argument.type_id);
                            let current_size_hint = variant_type.abi_encode_size_hint(engines);
                            old_size_hint.max(current_size_hint)
                        });

                AbiEncodeSizeHint::range_from_min_max(min, max) + 8
            }

            x => unimplemented!("abi_encode_size_hint for [{}]", engines.help_out(x)),
        }
    }
}

#[derive(Debug)]
pub enum AbiEncodeSizeHint {
    CustomImpl,
    PotentiallyInfinite,
    Exact(usize),
    Range(usize, usize),
}

impl AbiEncodeSizeHint {
    fn range(min: usize, max: usize) -> AbiEncodeSizeHint {
        assert!(min <= max);
        AbiEncodeSizeHint::Range(min, max)
    }

    fn range_from_min_max(a: AbiEncodeSizeHint, b: AbiEncodeSizeHint) -> AbiEncodeSizeHint {
        match (a, b) {
            (AbiEncodeSizeHint::CustomImpl, _) => AbiEncodeSizeHint::CustomImpl,
            (_, AbiEncodeSizeHint::CustomImpl) => AbiEncodeSizeHint::CustomImpl,
            (AbiEncodeSizeHint::PotentiallyInfinite, _) => AbiEncodeSizeHint::PotentiallyInfinite,
            (_, AbiEncodeSizeHint::PotentiallyInfinite) => AbiEncodeSizeHint::PotentiallyInfinite,
            (AbiEncodeSizeHint::Exact(l), AbiEncodeSizeHint::Exact(r)) => {
                let min = l.min(r);
                let max = l.max(r);
                AbiEncodeSizeHint::range(min, max)
            }
            (AbiEncodeSizeHint::Exact(l), AbiEncodeSizeHint::Range(rmin, rmax)) => {
                let min = l.min(rmin);
                let max = l.max(rmax);
                AbiEncodeSizeHint::range(min, max)
            }
            (AbiEncodeSizeHint::Range(lmin, lmax), AbiEncodeSizeHint::Exact(r)) => {
                let min = r.min(lmin);
                let max = r.max(lmax);
                AbiEncodeSizeHint::range(min, max)
            }
            (AbiEncodeSizeHint::Range(lmin, lmax), AbiEncodeSizeHint::Range(rmin, rmax)) => {
                let min = lmin.min(rmin);
                let max = lmax.max(rmax);
                AbiEncodeSizeHint::range(min, max)
            }
        }
    }

    fn min(&self, other: AbiEncodeSizeHint) -> AbiEncodeSizeHint {
        match (self, &other) {
            (AbiEncodeSizeHint::CustomImpl, _) => AbiEncodeSizeHint::CustomImpl,
            (_, AbiEncodeSizeHint::CustomImpl) => AbiEncodeSizeHint::CustomImpl,
            (AbiEncodeSizeHint::PotentiallyInfinite, _) => AbiEncodeSizeHint::PotentiallyInfinite,
            (_, AbiEncodeSizeHint::PotentiallyInfinite) => AbiEncodeSizeHint::PotentiallyInfinite,
            (AbiEncodeSizeHint::Exact(l), AbiEncodeSizeHint::Exact(r)) => {
                AbiEncodeSizeHint::Exact(*l.min(r))
            }
            (AbiEncodeSizeHint::Exact(l), AbiEncodeSizeHint::Range(rmin, _)) => {
                AbiEncodeSizeHint::Exact(*l.min(rmin))
            }
            (AbiEncodeSizeHint::Range(lmin, _), AbiEncodeSizeHint::Exact(r)) => {
                AbiEncodeSizeHint::Exact(*r.min(lmin))
            }
            (AbiEncodeSizeHint::Range(lmin, _), AbiEncodeSizeHint::Range(rmin, _)) => {
                AbiEncodeSizeHint::Exact(*lmin.min(rmin))
            }
        }
    }

    fn max(&self, other: AbiEncodeSizeHint) -> AbiEncodeSizeHint {
        match (self, &other) {
            (AbiEncodeSizeHint::CustomImpl, _) => AbiEncodeSizeHint::CustomImpl,
            (_, AbiEncodeSizeHint::CustomImpl) => AbiEncodeSizeHint::CustomImpl,
            (AbiEncodeSizeHint::PotentiallyInfinite, _) => AbiEncodeSizeHint::PotentiallyInfinite,
            (_, AbiEncodeSizeHint::PotentiallyInfinite) => AbiEncodeSizeHint::PotentiallyInfinite,
            (AbiEncodeSizeHint::Exact(l), AbiEncodeSizeHint::Exact(r)) => {
                AbiEncodeSizeHint::Exact(*l.max(r))
            }
            (AbiEncodeSizeHint::Exact(l), AbiEncodeSizeHint::Range(_, rmax)) => {
                AbiEncodeSizeHint::Exact(*l.max(rmax))
            }
            (AbiEncodeSizeHint::Range(_, lmax), AbiEncodeSizeHint::Exact(r)) => {
                AbiEncodeSizeHint::Exact(*r.max(lmax))
            }
            (AbiEncodeSizeHint::Range(_, lmax), AbiEncodeSizeHint::Range(_, rmax)) => {
                AbiEncodeSizeHint::Exact(*lmax.max(rmax))
            }
        }
    }
}

impl std::ops::Add<usize> for AbiEncodeSizeHint {
    type Output = AbiEncodeSizeHint;

    fn add(self, rhs: usize) -> Self::Output {
        match self {
            AbiEncodeSizeHint::CustomImpl => AbiEncodeSizeHint::CustomImpl,
            AbiEncodeSizeHint::PotentiallyInfinite => AbiEncodeSizeHint::PotentiallyInfinite,
            AbiEncodeSizeHint::Exact(current) => AbiEncodeSizeHint::Exact(current + rhs),
            AbiEncodeSizeHint::Range(min, max) => AbiEncodeSizeHint::range(min + rhs, max + rhs),
        }
    }
}

impl std::ops::Add<AbiEncodeSizeHint> for AbiEncodeSizeHint {
    type Output = AbiEncodeSizeHint;

    fn add(self, rhs: AbiEncodeSizeHint) -> Self::Output {
        match (self, &rhs) {
            (AbiEncodeSizeHint::CustomImpl, _) => AbiEncodeSizeHint::CustomImpl,
            (_, AbiEncodeSizeHint::CustomImpl) => AbiEncodeSizeHint::CustomImpl,
            (AbiEncodeSizeHint::PotentiallyInfinite, _) => AbiEncodeSizeHint::PotentiallyInfinite,
            (_, AbiEncodeSizeHint::PotentiallyInfinite) => AbiEncodeSizeHint::PotentiallyInfinite,
            (AbiEncodeSizeHint::Exact(l), AbiEncodeSizeHint::Exact(r)) => {
                AbiEncodeSizeHint::Exact(l + r)
            }
            (AbiEncodeSizeHint::Exact(l), AbiEncodeSizeHint::Range(rmin, rmax)) => {
                AbiEncodeSizeHint::range(rmin + l, rmax + l)
            }
            (AbiEncodeSizeHint::Range(lmin, lmax), AbiEncodeSizeHint::Exact(r)) => {
                AbiEncodeSizeHint::range(lmin + r, lmax + r)
            }
            (AbiEncodeSizeHint::Range(lmin, lmax), AbiEncodeSizeHint::Range(rmin, rmax)) => {
                AbiEncodeSizeHint::range(lmin + rmin, lmax + rmax)
            }
        }
    }
}

impl std::ops::Mul<usize> for AbiEncodeSizeHint {
    type Output = AbiEncodeSizeHint;

    fn mul(self, rhs: usize) -> Self::Output {
        match self {
            AbiEncodeSizeHint::CustomImpl => AbiEncodeSizeHint::CustomImpl,
            AbiEncodeSizeHint::PotentiallyInfinite => AbiEncodeSizeHint::PotentiallyInfinite,
            AbiEncodeSizeHint::Exact(current) => AbiEncodeSizeHint::Exact(current * rhs),
            AbiEncodeSizeHint::Range(min, max) => AbiEncodeSizeHint::range(min * rhs, max * rhs),
        }
    }
}

fn print_inner_types(
    engines: &Engines,
    name: &str,
    inner_types: impl Iterator<Item = TypeId>,
) -> String {
    let inner_types = inner_types
        .map(|x| engines.help_out(x).to_string())
        .collect::<Vec<_>>();
    format!(
        "{}{}",
        name,
        if inner_types.is_empty() {
            String::new()
        } else {
            format!("<{}>", inner_types.join(", "))
        }
    )
}

fn print_inner_types_debug(
    engines: &Engines,
    name: &str,
    inner_types: impl Iterator<Item = TypeId>,
) -> String {
    let inner_types = inner_types
        .map(|x| format!("{:?}", engines.help_out(x)))
        .collect::<Vec<_>>();
    format!(
        "{}{}",
        name,
        if inner_types.is_empty() {
            String::new()
        } else {
            format!("<{}>", inner_types.join(", "))
        }
    )
}

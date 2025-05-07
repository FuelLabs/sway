use crate::{
    concurrent_slab::{ConcurrentSlab, ListDisplay},
    decl_engine::*,
    engine_threading::*,
    language::{
        parsed::{EnumDeclaration, StructDeclaration},
        ty::{TyEnumDecl, TyExpression, TyStructDecl},
        QualifiedCallPath,
    },
    type_system::priv_prelude::*,
};
use core::fmt::Write;
use hashbrown::{hash_map::RawEntryMut, HashMap};
use parking_lot::RwLock;
use std::{
    hash::{BuildHasher, Hash, Hasher},
    sync::Arc,
    time::Instant,
};
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
    type_error::TypeError,
};
use sway_types::{
    integer_bits::IntegerBits, span::Span, Ident, Named, ProgramId, SourceId, Spanned,
};

use super::{
    ast_elements::{length::NumericLength, type_parameter::ConstGenericExpr},
    unify::unifier::UnifyKind,
};

/// To be able to garbage-collect [TypeInfo]s from the [TypeEngine]
/// we need to track which types need to be GCed when a particular
/// module, represented by its source id, is GCed. [TypeSourceInfo]
/// encapsulates this information.
///
/// For types that should never be GCed the `source_id` must be `None`.
///
/// The concrete logic of assigning `source_id`s to `type_info`s is
/// given in the [TypeEngine::get_type_fallback_source_id].
// TODO: This logic will be further improved when https://github.com/FuelLabs/sway/issues/6603
//       is implemented (Optimize `TypeEngine` for garbage collection).
#[derive(Debug, Default, Clone)]
struct TypeSourceInfo {
    type_info: Arc<TypeInfo>,
    source_id: Option<SourceId>,
}

impl TypeSourceInfo {
    /// Returns true if the `self` would be equal to another [TypeSourceInfo]
    /// created from `type_info` and `source_id`.
    ///
    /// This method allows us to test equality "upfront", without the need to
    /// create a new [TypeSourceInfo] which would require a heap allocation
    /// of a new [TypeInfo].
    pub(crate) fn equals(
        &self,
        type_info: &TypeInfo,
        source_id: &Option<SourceId>,
        ctx: &PartialEqWithEnginesContext,
    ) -> bool {
        &self.source_id == source_id && self.type_info.eq(type_info, ctx)
    }
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
        self.equals(&other.type_info, &other.source_id, ctx)
    }
}

/// Holds the singleton instances of [TypeSourceInfo]s of *replaceable types* that,
/// although being inserted anew into the [TypeEngine], all share the single definition.
/// This means that, e.g., all the different [TypeEngine::slab] entries representing
/// the, e.g., [TypeInfo::Unknown] will point to the same singleton instance
/// of the corresponding [TypeSourceInfo].
#[derive(Debug, Clone)]
struct SingletonTypeSourceInfos {
    /// The single instance of the [TypeSourceInfo]
    /// representing the [TypeInfo::Unknown] replaceable type.
    unknown: Arc<TypeSourceInfo>,
    /// The single instance of the [TypeSourceInfo]
    /// representing the [TypeInfo::Numeric] replaceable type.
    numeric: Arc<TypeSourceInfo>,
}

/// Holds the instances of [TypeInfo]s and allows exchanging them for [TypeId]s.
/// Supports LSP garbage collection of unused [TypeInfo]s assigned to a particular [SourceId].
///
/// ## Intended Usage
/// Inserting [TypeInfo]s into the type engine returns a [TypeId] that can later be used
/// to get the same [TypeInfo] by using the [TypeEngine::get] method.
///
/// Properly using the various inserting methods is crucial for the optimal work of the type engine.
///
/// These methods are grouped by the following convention and are intended to be used in the
/// order of precedence given below:
/// - `id_of_<type>`: methods that always return the same [TypeId] for a type.
///   These methods, when inlined, compile to constant [TypeId]s.
/// - `new_<type>`: methods that always return a new [TypeId] for a type.
/// - `insert_<type>[_<additional options>]`: methods that might insert a new type into the engine,
///   and return a new [TypeId], but also reuse an existing [TypeInfo] and return an existing [TypeId].
/// - `insert`: the fallback method that should be used only in cases when the type is not known
///   at the call site.
///
/// ## Internal Implementation
/// [TypeInfo]s are stored in a private [TypeSourceInfo] structure that binds them with a [SourceId]
/// of the module in which they are used. Those [TypeSourceInfo]s are referenced from the `slab`.
/// The actual [TypeId] of a [TypeInfo] is just an index in the `slab`.
///
/// The engine attempts to maximize the reuse of [TypeSourceInfo]s by holding _shareable types_
/// (see: [Self::is_type_shareable]) in the `shareable_types` hash map.
///
/// TODO: Note that the reuse currently happens on the level of [TypeSourceInfo]s, and not [TypeInfo]s.
///       This is not optimal and will be improved in https://github.com/FuelLabs/sway/issues/6603.
///       Also note that because of that, having [TypeInfo] stored in `Arc` within the [TypeSourceInfo]
///       does not bring any real benefits.
///
/// The implementation of the type engine is primarily directed with the goal of maximizing the
/// reuse of the [TypeSourceInfo]s while at the same time having the [TypeInfo]s bound to [SourceId]s
/// of their use site, so that they can be garbage collected.
///
/// TODO: Note that the assignment of [SourceId]s to [TypeInfo]s is currently not as optimal as it
///       can be. This will be improved in https://github.com/FuelLabs/sway/issues/6603.
#[derive(Debug)]
pub struct TypeEngine {
    slab: ConcurrentSlab<TypeSourceInfo>,
    /// Holds [TypeId]s of [TypeSourceInfo]s of shareable types (see: [Self::is_type_shareable]).
    /// [TypeSourceInfo]s of shareable types can be reused if the type is used more
    /// then once. In that case, for every usage, instead of inserting a new [TypeSourceInfo] instance
    /// into the [Self::slab], the [TypeId] of an existing instance is returned.
    shareable_types: RwLock<HashMap<Arc<TypeSourceInfo>, TypeId>>,
    singleton_types: RwLock<SingletonTypeSourceInfos>,
    unifications: ConcurrentSlab<Unification>,
    last_replace: RwLock<Instant>,
}

pub trait IsConcrete {
    fn is_concrete(&self, engines: &Engines) -> bool;
}

#[derive(Debug, Clone)]
pub(crate) struct Unification {
    pub received: TypeId,
    pub expected: TypeId,
    pub span: Span,
    pub help_text: String,
    pub unify_kind: UnifyKind,
}

impl Default for TypeEngine {
    fn default() -> Self {
        let singleton_types = SingletonTypeSourceInfos {
            unknown: TypeSourceInfo {
                type_info: TypeInfo::Unknown.into(),
                source_id: None,
            }
            .into(),
            numeric: TypeSourceInfo {
                type_info: TypeInfo::Numeric.into(),
                source_id: None,
            }
            .into(),
        };

        let mut te = TypeEngine {
            slab: Default::default(),
            shareable_types: Default::default(),
            singleton_types: RwLock::new(singleton_types),
            unifications: Default::default(),
            last_replace: RwLock::new(Instant::now()),
        };
        te.insert_shareable_built_in_types();
        te
    }
}

impl Clone for TypeEngine {
    fn clone(&self) -> Self {
        TypeEngine {
            slab: self.slab.clone(),
            shareable_types: RwLock::new(self.shareable_types.read().clone()),
            singleton_types: RwLock::new(self.singleton_types.read().clone()),
            unifications: self.unifications.clone(),
            last_replace: RwLock::new(*self.last_replace.read()),
        }
    }
}

/// Generates:
///  - `id_of_<type>` methods for every provided shareable built-in type.
///  - `insert_shareable_built_in_types` method for initial creation of built-in types within the [TypeEngine].
///  - `get_shareable_built_in_type_id` method for potential retrieval of built-in types in the [TypeEngine::insert] method.
///
/// Note that, when invoking the macro, the `unit` and the [TypeInfo::ErrorRecovery] types *must not be provided in the list*.
/// The shareable `unit` type requires a special treatment within the macro, because it is modeled
/// as [TypeInfo::Tuple] with zero elements and not as a separate [TypeInfo] variant.
/// The [TypeInfo::ErrorRecovery], although being an enum variant with the parameter (the [ErrorEmitted] proof), is
/// actually a single type because all the [ErrorEmitted] proofs are the same.
/// This special case is also handled within the macro.
///
/// Unfortunately, due to limitations of Rust's macro-by-example, the [TypeInfo] must be
/// provided twice during the macro invocation, once as an expression `expr` and once as a pattern `pat`.
///
/// The macro recursively creates the `id_of_<type>` methods in order to get the proper `usize` value
/// generated, which corresponds to the index of those types within the slab.
macro_rules! type_engine_shareable_built_in_types {
    // The base recursive case.
    (@step $_idx:expr,) => {};

    // The actual recursion step that generates the `id_of_<type>` functions.
    (@step $idx:expr, ($ty_name:ident, $ti:expr, $ti_pat:pat), $(($tail_ty_name:ident, $tail_ti:expr, $tail_ti_pat:pat),)*) => {
        paste::paste! {
            pub const fn [<id_of_ $ty_name>](&self) -> TypeId {
                TypeId::new($idx)
            }
        }

        type_engine_shareable_built_in_types!(@step $idx + 1, $(($tail_ty_name, $tail_ti, $tail_ti_pat),)*);
    };

    // The entry point. Invoking the macro matches this arm.
    ($(($ty_name:ident, $ti:expr, $ti_pat:pat),)*) => {
        // The `unit` type is a special case. It will be inserted in the slab as the first type.
        pub(crate) const fn id_of_unit(&self) -> TypeId {
            TypeId::new(0)
        }

        // The error recovery type is a special case. It will be inserted in the slab as the second type.
        // To preserve the semantics of the `TypeInfo::ErrorRecovery(ErrorEmitted)`, we still insist on
        // providing the proof of the error being emitted, although that proof is actually
        // not needed to obtain the type id, nor is used within this method at all.
        #[allow(unused_variables)]
        pub(crate) const fn id_of_error_recovery(&self, error_emitted: ErrorEmitted) -> TypeId {
            TypeId::new(1)
        }

        // Generate the remaining `id_of_<type>` methods. We start counting the indices from 2.
        type_engine_shareable_built_in_types!(@step 2, $(($ty_name, $ti, $ti_pat),)*);

        // Generate the method that initially inserts the built-in shareable types into the `slab` in the right order.
        //
        // Note that we are inserting the types **only into the `slab`, but not into the `shareable_types`**,
        // although they should, by definition, be in the `shareable_types` as well.
        //
        // What is the reason for not inserting them into the `shareable_types`?
        //
        // To insert them into the `shareable_types` we need `Engines` to be able to calculate
        // the hash and equality with engines, and method is supposed be called internally during the creation
        // of the `TypeEngine`. At that moment, we are creating a single, isolated engine, and do not
        // have all the engines available. The only way to have it called with all the engines, is
        // to do it when `Engines` are created. But this would mean that we cannot have a semantically
        // valid `TypeEngine` created in isolation, without the `Engines`, which would be a problematic
        // design that breaks cohesion and creates unexpected dependency.
        //
        // Note that having the built-in shareable types initially inserted only in the `slab` does
        // not cause any issues with type insertion and retrieval. The `id_of_<type>` methods return
        // indices that are compile-time constants and there is no need for `shareable_types` access.
        // Also, calling `insert` with built-in shareable types has an optimized path which will redirect
        // to `id_of_<type>` methods, again bypassing the `shareable_types`.
        //
        // The only negligible small "penalty" comes during replacements of replaceable types,
        // where a potential replacement with a built-in shareable type will create a separate instance
        // of that built-in type and add it to `shareable_types`.
        fn insert_shareable_built_in_types(&mut self) {
            use TypeInfo::*;

            let tsi = TypeSourceInfo {
                type_info: Tuple(vec![]).into(),
                source_id: None,
            };
            self.slab.insert(tsi);

            // For the `ErrorRecovery`, we need an `ErrorEmitted` instance.
            // All of its instances are the same, so we will use, or perhaps misuse,
            // the `Handler::cancel` method here to obtain an instance.
            let tsi = TypeSourceInfo {
                type_info: ErrorRecovery(crate::Handler::default().cancel()).into(),
                source_id: None,
            };
            self.slab.insert(tsi);

            $(
                let tsi = TypeSourceInfo {
                    type_info: $ti.into(),
                    source_id: None,
                };
                self.slab.insert(tsi);
            )*
        }

        /// Returns the [TypeId] of the `type_info` only if the type info is
        /// a shareable built-in type, otherwise `None`.
        ///
        /// For a particular shareable built-in type, the method guarantees to always
        /// return the same, existing [TypeId].
        fn get_shareable_built_in_type_id(&self, type_info: &TypeInfo) -> Option<TypeId> {
            paste::paste! {
                use TypeInfo::*;
                match type_info {
                    Tuple(v) if v.is_empty() => Some(self.id_of_unit()),
                    // Here we also "pass" the dummy value obtained from `Handler::cancel` which will be
                    // optimized away.
                    ErrorRecovery(_) => Some(self.id_of_error_recovery(crate::Handler::default().cancel())),
                    $(
                        $ti_pat => Some(self.[<id_of_ $ty_name>]()),
                    )*
                    _ => None
                }
            }
        }

        /// Returns true if the type represented by the `type_info`
        /// is a shareable built-in type.
        fn is_shareable_built_in_type(&self, type_info: &TypeInfo) -> bool {
            use TypeInfo::*;
            match type_info {
                Tuple(v) if v.is_empty() => true,
                // Here we also "pass" the dummy value obtained from `Handler::cancel` which will be
                // optimized away.
                ErrorRecovery(_) => true,
                $(
                    $ti_pat => true,
                )*
                _ => false
            }
        }
    }
}

impl TypeEngine {
    type_engine_shareable_built_in_types!(
        (never, Never, Never),
        (string_slice, StringSlice, StringSlice),
        (
            u8,
            UnsignedInteger(IntegerBits::Eight),
            UnsignedInteger(IntegerBits::Eight)
        ),
        (
            u16,
            UnsignedInteger(IntegerBits::Sixteen),
            UnsignedInteger(IntegerBits::Sixteen)
        ),
        (
            u32,
            UnsignedInteger(IntegerBits::ThirtyTwo),
            UnsignedInteger(IntegerBits::ThirtyTwo)
        ),
        (
            u64,
            UnsignedInteger(IntegerBits::SixtyFour),
            UnsignedInteger(IntegerBits::SixtyFour)
        ),
        (
            u256,
            UnsignedInteger(IntegerBits::V256),
            UnsignedInteger(IntegerBits::V256)
        ),
        (bool, Boolean, Boolean),
        (b256, B256, B256),
        (contract, Contract, Contract),
        (raw_ptr, RawUntypedPtr, RawUntypedPtr),
        (raw_slice, RawUntypedSlice, RawUntypedSlice),
    );

    /// Inserts a new [TypeInfo::Unknown] into the [TypeEngine] and returns its [TypeId].
    ///
    /// [TypeInfo::Unknown] is an always replaceable type and the method
    /// guarantees that a new (or unused) [TypeId] will be returned on every
    /// call.
    pub(crate) fn new_unknown(&self) -> TypeId {
        TypeId::new(
            self.slab
                .insert_arc(self.singleton_types.read().unknown.clone()),
        )
    }

    /// Inserts a new [TypeInfo::Numeric] into the [TypeEngine] and returns its [TypeId].
    ///
    /// [TypeInfo::Numeric] is an always replaceable type and the method
    /// guarantees that a new (or unused) [TypeId] will be returned on every
    /// call.
    pub(crate) fn new_numeric(&self) -> TypeId {
        TypeId::new(
            self.slab
                .insert_arc(self.singleton_types.read().numeric.clone()),
        )
    }

    /// Inserts a new [TypeInfo::TypeParam] into the [TypeEngine] and returns its [TypeId].
    ///
    /// [TypeInfo::TypeParam] is an always replaceable type and the method
    /// guarantees that a new (or unused) [TypeId] will be returned on every
    /// call.
    pub(crate) fn new_type_param(&self, type_parameter: TypeParameter) -> TypeId {
        self.new_type_param_impl(TypeInfo::TypeParam(type_parameter))
    }

    fn new_type_param_impl(&self, type_param: TypeInfo) -> TypeId {
        let source_id = self.get_type_parameter_fallback_source_id(&type_param);
        let tsi = TypeSourceInfo {
            type_info: type_param.into(),
            source_id,
        };
        TypeId::new(self.slab.insert(tsi))
    }

    /// Inserts a new [TypeInfo::Placeholder] into the [TypeEngine] and returns its [TypeId].
    ///
    /// [TypeInfo::Placeholder] is an always replaceable type and the method
    /// guarantees that a new (or unused) [TypeId] will be returned on every
    /// call.
    pub(crate) fn new_placeholder(&self, type_parameter: TypeParameter) -> TypeId {
        self.new_placeholder_impl(TypeInfo::Placeholder(type_parameter))
    }

    fn new_placeholder_impl(&self, placeholder: TypeInfo) -> TypeId {
        let source_id = self.get_placeholder_fallback_source_id(&placeholder);
        let tsi = TypeSourceInfo {
            type_info: placeholder.into(),
            source_id,
        };
        TypeId::new(self.slab.insert(tsi))
    }

    /// Inserts a new [TypeInfo::UnknownGeneric] into the [TypeEngine] and returns its [TypeId].
    ///
    /// [TypeInfo::UnknownGeneric] is an always replaceable type and the method
    /// guarantees that a new (or unused) [TypeId] will be returned on every
    /// call.
    pub(crate) fn new_unknown_generic(
        &self,
        name: Ident,
        trait_constraints: VecSet<TraitConstraint>,
        parent: Option<TypeId>,
        is_from_type_parameter: bool,
    ) -> TypeId {
        self.new_unknown_generic_impl(TypeInfo::UnknownGeneric {
            name,
            trait_constraints,
            parent,
            is_from_type_parameter,
        })
    }

    fn new_unknown_generic_impl(&self, unknown_generic: TypeInfo) -> TypeId {
        let source_id = Self::get_unknown_generic_fallback_source_id(&unknown_generic);
        let tsi = TypeSourceInfo {
            type_info: unknown_generic.into(),
            source_id,
        };
        TypeId::new(self.slab.insert(tsi))
    }

    /// Inserts a new [TypeInfo::UnknownGeneric] into the [TypeEngine]
    /// that represents a `Self` type and returns its [TypeId].
    /// The unknown generic `name` [Ident] will be set to "Self" with the provided `use_site_span`.
    ///
    /// Note that the span in general does not point to a reserved word "Self" in
    /// the source code, nor is related to it. The `Self` type represents the type
    /// in `impl`s and does not necessarily relate to the "Self" keyword in code.
    ///
    /// Therefore, *the span must always point to a location in the source file in which
    /// the particular `Self` type is, e.g., being declared or implemented*.
    ///
    /// Returns the [TypeId] and the [Ident] set to "Self" and the provided `use_site_span`.
    pub(crate) fn new_unknown_generic_self(
        &self,
        use_site_span: Span,
        is_from_type_parameter: bool,
    ) -> (TypeId, Ident) {
        let name = Ident::new_with_override("Self".into(), use_site_span);
        let type_id =
            self.new_unknown_generic(name.clone(), VecSet(vec![]), None, is_from_type_parameter);
        (type_id, name)
    }

    /// Inserts a new [TypeInfo::Enum] into the [TypeEngine] and returns
    /// its [TypeId], or returns a [TypeId] of an existing shareable enum type
    /// that corresponds to the enum given by the `decl_id`.
    pub(crate) fn insert_enum(&self, engines: &Engines, decl_id: DeclId<TyEnumDecl>) -> TypeId {
        let decl = engines.de().get_enum(&decl_id);
        let source_id = Self::get_enum_fallback_source_id(&decl);
        let is_shareable_type = self.is_shareable_enum(engines, &decl);
        let type_info = TypeInfo::Enum(decl_id);
        self.insert_or_replace_type_source_info(
            engines,
            type_info,
            source_id,
            is_shareable_type,
            None,
        )
    }

    /// Inserts a new [TypeInfo::Struct] into the [TypeEngine] and returns
    /// its [TypeId], or returns a [TypeId] of an existing shareable struct type
    /// that corresponds to the struct given by the `decl_id`.
    pub(crate) fn insert_struct(&self, engines: &Engines, decl_id: DeclId<TyStructDecl>) -> TypeId {
        let decl = engines.de().get_struct(&decl_id);
        let source_id = Self::get_struct_fallback_source_id(&decl);
        let is_shareable_type = self.is_shareable_struct(engines, &decl);
        let type_info = TypeInfo::Struct(decl_id);
        self.insert_or_replace_type_source_info(
            engines,
            type_info,
            source_id,
            is_shareable_type,
            None,
        )
    }

    /// Inserts a new [TypeInfo::Tuple] into the [TypeEngine] and returns
    /// its [TypeId], or returns a [TypeId] of an existing shareable tuple type
    /// that corresponds to the tuple given by the `elements`.
    pub(crate) fn insert_tuple(&self, engines: &Engines, elements: Vec<GenericArgument>) -> TypeId {
        let source_id = self.get_tuple_fallback_source_id(&elements);
        let is_shareable_type = self.is_shareable_tuple(engines, &elements);
        let type_info = TypeInfo::Tuple(elements);
        self.insert_or_replace_type_source_info(
            engines,
            type_info,
            source_id,
            is_shareable_type,
            None,
        )
    }

    /// Same as [Self::insert_tuple], but intended to be used mostly in the code generation,
    /// where the tuple elements are non-annotated [TypeArgument]s that contain
    /// only the [TypeId]s provided in the `elements`.
    pub(crate) fn insert_tuple_without_annotations(
        &self,
        engines: &Engines,
        elements: Vec<TypeId>,
    ) -> TypeId {
        self.insert_tuple(
            engines,
            elements.into_iter().map(|type_id| type_id.into()).collect(),
        )
    }

    /// Inserts a new [TypeInfo::Array] into the [TypeEngine] and returns
    /// its [TypeId], or returns a [TypeId] of an existing shareable array type
    /// that corresponds to the array given by the `elem_type` and the `length`.
    pub(crate) fn insert_array(
        &self,
        engines: &Engines,
        elem_type: GenericArgument,
        length: Length,
    ) -> TypeId {
        let source_id = self.get_array_fallback_source_id(&elem_type, &length);
        let is_shareable_type = self.is_shareable_array(engines, &elem_type, &length);
        let type_info = TypeInfo::Array(elem_type, length);
        self.insert_or_replace_type_source_info(
            engines,
            type_info,
            source_id,
            is_shareable_type,
            None,
        )
    }

    /// Same as [Self::insert_array], but intended to insert arrays without annotations.
    // TODO: Unlike `insert_array`, once the https://github.com/FuelLabs/sway/issues/6603 gets implemented,
    //       this method will get the additional `use_site_source_id` parameter.
    pub(crate) fn insert_array_without_annotations(
        &self,
        engines: &Engines,
        elem_type: TypeId,
        length: usize,
    ) -> TypeId {
        self.insert_array(
            engines,
            elem_type.into(),
            Length(ConstGenericExpr::literal(length, None)),
        )
    }

    /// Inserts a new [TypeInfo::StringArray] into the [TypeEngine] and returns
    /// its [TypeId], or returns a [TypeId] of an existing shareable string array type
    /// that corresponds to the string array given by the `length`.
    pub(crate) fn insert_string_array(&self, engines: &Engines, length: NumericLength) -> TypeId {
        let source_id = Self::get_string_array_fallback_source_id(&length);
        let is_shareable_type = self.is_shareable_string_array(&length);
        let type_info = TypeInfo::StringArray(length);
        self.insert_or_replace_type_source_info(
            engines,
            type_info,
            source_id,
            is_shareable_type,
            None,
        )
    }

    /// Same as [Self::insert_string_array], but intended to insert string arrays without annotations.
    // TODO: Unlike `insert_string_array`, once the https://github.com/FuelLabs/sway/issues/6603 gets implemented,
    //       this method will get the additional `use_site_source_id` parameter.
    pub(crate) fn insert_string_array_without_annotations(
        &self,
        engines: &Engines,
        length: usize,
    ) -> TypeId {
        self.insert_string_array(
            engines,
            NumericLength {
                val: length,
                span: Span::dummy(),
            },
        )
    }

    /// Inserts a new [TypeInfo::ContractCaller] into the [TypeEngine] and returns its [TypeId].
    ///
    /// [TypeInfo::ContractCaller] is not a shareable type and the method
    /// guarantees that a new (or unused) [TypeId] will be returned on every
    /// call.
    pub(crate) fn new_contract_caller(
        &self,
        engines: &Engines,
        abi_name: AbiName,
        address: Option<Box<TyExpression>>,
    ) -> TypeId {
        // The contract caller type shareability would be calculated as:
        //
        //   !(Self::is_replaceable_contract_caller(abi_name, address)
        //     ||
        //     Self::is_contract_caller_distinguishable_by_annotations(abi_name, address))
        //
        // If the contract caller is replaceable, either the `abi_name` id `Deferred` or the `address` is `None`.
        // On the other hand, if the `abi_name` is `Known` or the `address` is `Some`, it will be distinguishable by annotations.
        // Which means, it will be either replaceable or distinguishable by annotations, which makes the condition always
        // evaluating to false.
        //
        // The fact that we cannot share `ContractCaller`s is not an issue. In any real-life project, the number of contract callers
        // will be negligible, order of magnitude of ~10.
        let source_id = Self::get_contract_caller_fallback_source_id(&abi_name, &address);
        let type_info = TypeInfo::ContractCaller { abi_name, address };
        self.insert_or_replace_type_source_info(engines, type_info, source_id, false, None)
    }

    /// Inserts a new [TypeInfo::Alias] into the [TypeEngine] and returns its [TypeId].
    ///
    /// [TypeInfo::Alias] is not a shareable type and the method
    /// guarantees that a new (or unused) [TypeId] will be returned on every
    /// call.
    pub(crate) fn new_alias(&self, engines: &Engines, name: Ident, ty: GenericArgument) -> TypeId {
        // The alias type shareability would be calculated as `!(false || true) ==>> false`.
        let source_id = self.get_alias_fallback_source_id(&name, &ty);
        let type_info = TypeInfo::Alias { name, ty };
        self.insert_or_replace_type_source_info(engines, type_info, source_id, false, None)
    }

    /// Inserts a new [TypeInfo::Custom] into the [TypeEngine] and returns its [TypeId].
    ///
    /// [TypeInfo::Custom] is not a shareable type and the method
    /// guarantees that a new (or unused) [TypeId] will be returned on every
    /// call.
    pub(crate) fn new_custom(
        &self,
        engines: &Engines,
        qualified_call_path: QualifiedCallPath,
        type_arguments: Option<Vec<GenericArgument>>,
    ) -> TypeId {
        let source_id = self.get_custom_fallback_source_id(&qualified_call_path, &type_arguments);
        // The custom type shareability would be calculated as `!(true || true) ==>> false`.
        // TODO: Improve handling of `TypeInfo::Custom` and `TypeInfo::TraitType`` within the `TypeEngine`:
        //       https://github.com/FuelLabs/sway/issues/6601
        let is_shareable_type = false;
        let type_info = TypeInfo::Custom {
            qualified_call_path,
            type_arguments,
        };
        self.insert_or_replace_type_source_info(
            engines,
            type_info,
            source_id,
            is_shareable_type,
            None,
        )
    }

    /// Inserts a new [TypeInfo::Custom] into the [TypeEngine] and returns its [TypeId].
    /// The custom type is defined only by its `name`. In other words, it does not have
    /// the qualified call path or type arguments. This is a very common situation in
    /// the code that just uses the type name, like, e.g., when instantiating structs:
    ///
    /// ```ignore
    /// let _ = Struct { };
    /// ```
    ///
    /// [TypeInfo::Custom] is not a shareable type and the method
    /// guarantees that a new (or unused) [TypeId] will be returned on every
    /// call.
    pub(crate) fn new_custom_from_name(&self, engines: &Engines, name: Ident) -> TypeId {
        self.new_custom(engines, name.into(), None)
    }

    /// Creates a new [TypeInfo::Custom] that represents a Self type.
    ///
    /// The `span` must either be a [Span::dummy] or a span pointing
    /// to text "Self" or "self", otherwise the method panics.
    ///
    /// [TypeInfo::Custom] is not a shareable type and the method
    /// guarantees that a new (or unused) [TypeId] will be returned on every
    /// call.
    pub(crate) fn new_self_type(&self, engines: &Engines, span: Span) -> TypeId {
        let source_id = span.source_id().copied();
        // The custom type shareability would be calculated as `!(true || true) ==>> false`.
        // TODO: Improve handling of `TypeInfo::Custom` and `TypeInfo::TraitType`` within the `TypeEngine`:
        //       https://github.com/FuelLabs/sway/issues/6601
        let is_shareable_type = false;
        let type_info = TypeInfo::new_self_type(span);
        self.insert_or_replace_type_source_info(
            engines,
            type_info,
            source_id,
            is_shareable_type,
            None,
        )
    }

    /// Inserts a new [TypeInfo::Slice] into the [TypeEngine] and returns
    /// its [TypeId], or returns a [TypeId] of an existing shareable slice type
    /// that corresponds to the slice given by the `elem_type`.
    pub(crate) fn insert_slice(&self, engines: &Engines, elem_type: GenericArgument) -> TypeId {
        let source_id = self.get_slice_fallback_source_id(&elem_type);
        let is_shareable_type = self.is_shareable_slice(engines, &elem_type);
        let type_info = TypeInfo::Slice(elem_type);
        self.insert_or_replace_type_source_info(
            engines,
            type_info,
            source_id,
            is_shareable_type,
            None,
        )
    }

    /// Inserts a new [TypeInfo::Ptr] into the [TypeEngine] and returns
    /// its [TypeId], or returns a [TypeId] of an existing shareable pointer type
    /// that corresponds to the pointer given by the `pointee_type`.
    pub(crate) fn insert_ptr(&self, engines: &Engines, pointee_type: GenericArgument) -> TypeId {
        let source_id = self.get_ptr_fallback_source_id(&pointee_type);
        let is_shareable_type = self.is_shareable_ptr(engines, &pointee_type);
        let type_info = TypeInfo::Ptr(pointee_type);
        self.insert_or_replace_type_source_info(
            engines,
            type_info,
            source_id,
            is_shareable_type,
            None,
        )
    }

    /// Inserts a new [TypeInfo::Ref] into the [TypeEngine] and returns
    /// its [TypeId], or returns a [TypeId] of an existing shareable reference type
    /// that corresponds to the reference given by the `referenced_type` and `to_mutable_value`.
    pub(crate) fn insert_ref(
        &self,
        engines: &Engines,
        to_mutable_value: bool,
        referenced_type: GenericArgument,
    ) -> TypeId {
        let source_id = self.get_ref_fallback_source_id(&referenced_type);
        let is_shareable_type = self.is_shareable_ref(engines, &referenced_type);
        let type_info = TypeInfo::Ref {
            to_mutable_value,
            referenced_type,
        };
        self.insert_or_replace_type_source_info(
            engines,
            type_info,
            source_id,
            is_shareable_type,
            None,
        )
    }

    /// Inserts a new [TypeInfo::TraitType] into the [TypeEngine] and returns
    /// its [TypeId], or returns a [TypeId] of an existing shareable trait type type
    /// that corresponds to the trait type given by the `name` and `trait_type_id`.
    pub(crate) fn insert_trait_type(
        &self,
        engines: &Engines,
        name: Ident,
        trait_type_id: TypeId,
    ) -> TypeId {
        let source_id = self.get_trait_type_fallback_source_id(&name, &trait_type_id);
        // The trait type type shareability would be calculated as `!(false || false) ==>> true`.
        // TODO: Improve handling of `TypeInfo::Custom` and `TypeInfo::TraitType`` within the `TypeEngine`:
        //       https://github.com/FuelLabs/sway/issues/6601
        let is_shareable_type = true;
        let type_info = TypeInfo::TraitType {
            name,
            trait_type_id,
        };
        self.insert_or_replace_type_source_info(
            engines,
            type_info,
            source_id,
            is_shareable_type,
            None,
        )
    }

    /// Same as [Self::insert_ref], but intended to insert references without annotations.
    // TODO: Unlike `insert_ref`, once the https://github.com/FuelLabs/sway/issues/6603 gets implemented,
    //       this method will get the additional `use_site_source_id` parameter.
    pub(crate) fn insert_ref_without_annotations(
        &self,
        engines: &Engines,
        to_mutable_value: bool,
        referenced_type: TypeId,
    ) -> TypeId {
        self.insert_ref(engines, to_mutable_value, referenced_type.into())
    }

    /// Inserts a [TypeInfo] into the [TypeEngine] and returns a [TypeId]
    /// referring to that [TypeInfo].
    pub(crate) fn insert(
        &self,
        engines: &Engines,
        ty: TypeInfo,
        source_id: Option<&SourceId>,
    ) -> TypeId {
        // Avoid all of the heavy lifting of inserting and replacing logic, if `ty` is a shareable built-in type.
        //
        // Note that we are ignoring here the eventual `source_id` that could be provided by the caller.
        // Ideally, for shareable built-in types that should never be the case, but `insert` is called in
        // rare cases where the `ty` is not known and not inspected and is usually providing the use site span.
        //
        // The reason for ignoring the `source_id` is, because we want these types to be reused and "live forever"
        // and never be garbage-collected and thus we do not assign any source id to them.
        if let Some(type_id) = self.get_shareable_built_in_type_id(&ty) {
            return type_id;
        }

        // Same for the replaceable types, avoid heavy lifting.
        // Note that we don't want to pack this `match` into a method, because we want to avoid cloning
        // of `ty` in the case of it being a `Placeholder` or `UnknownGeneric`.
        //
        // TODO: Also, note that also here we are ignoring the `source_id` provided by the caller.
        //       This is only temporary until https://github.com/FuelLabs/sway/issues/6603 gets implemented.
        //       Until then, this shortcut corresponds to the current `TypeEngine` behavior:
        //       - `Unknown`s and `Numeric`s never have `source_id` assigned.
        //       - for `Placeholder`s and `UnknownGeneric`s, the `source_id` is extracted from the call site.
        match ty {
            TypeInfo::Unknown => return self.new_unknown(),
            TypeInfo::Numeric => return self.new_numeric(),
            TypeInfo::Placeholder(_) => return self.new_placeholder_impl(ty),
            TypeInfo::TypeParam(_) => return self.new_type_param_impl(ty),
            TypeInfo::UnknownGeneric { .. } => return self.new_unknown_generic_impl(ty),
            _ => (),
        }

        let is_shareable_type = self.is_type_shareable(engines, &ty);
        let source_id = source_id
            .copied()
            .or_else(|| self.get_type_fallback_source_id(engines, &ty));

        self.insert_or_replace_type_source_info(engines, ty, source_id, is_shareable_type, None)
    }

    /// This method performs two actions, depending on the `replace_at_type_id`.
    ///
    /// If the `replace_at_type_id` is `Some`, this indicates that we want to unconditionally replace the [TypeSourceInfo]
    /// currently located at `replace_at_type_id` with the one made of the `ty` + `source_id` pair.
    /// In the case of replacement the method always return the [TypeId] provided in `replace_at_type_id`.
    ///
    /// If the `replace_at_type_id` is `None`, this indicates that we want to insert the [TypeSourceInfo], made of the
    /// `ty` + `source_id` pair, into the `TypeEngine`. The insertion into the engine might require a new insert into
    /// the `slab` or just returning a [TypeId] of an existing shareable [TypeSourceInfo] that is equal to the one defined by
    /// the `ty` + `source_id` pair.
    ///
    /// If a new insertion is always made, or a reuse is possible, depends on the shareability of `ty` that is given by
    /// `is_shareable_type`.
    fn insert_or_replace_type_source_info(
        &self,
        engines: &Engines,
        ty: TypeInfo,
        source_id: Option<SourceId>,
        is_shareable_type: bool,
        replace_at_type_id: Option<TypeId>,
    ) -> TypeId {
        if !is_shareable_type {
            let tsi = TypeSourceInfo {
                type_info: ty.into(),
                source_id,
            };
            match replace_at_type_id {
                Some(existing_id) => {
                    self.slab.replace(existing_id.index(), tsi);
                    existing_id
                }
                None => TypeId::new(self.slab.insert(tsi)),
            }
        } else {
            let mut shareable_types = self.shareable_types.write();

            let hash_builder = shareable_types.hasher().clone();
            let ty_hash =
                self.compute_hash_without_heap_allocation(engines, &hash_builder, &ty, &source_id);

            let raw_entry = shareable_types.raw_entry_mut().from_hash(ty_hash, |x| {
                // Not that the equality with engines of the types contained in the
                // `shareable_types` is "strict" in the sense that only one element can equal.
                // This is because the types that have annotation fields, a.k.a. distinguishable by
                // annotations types, will never end up in the hash map because they are considered
                // not to be shareable.
                x.equals(&ty, &source_id, &PartialEqWithEnginesContext::new(engines))
            });
            match raw_entry {
                RawEntryMut::Occupied(o) => match replace_at_type_id {
                    Some(existing_id) => {
                        let existing_type_source_info = o.key();
                        self.slab
                            .replace_arc(existing_id.index(), existing_type_source_info.clone());
                        existing_id
                    }
                    None => *o.get(),
                },
                RawEntryMut::Vacant(v) => {
                    let tsi = TypeSourceInfo {
                        type_info: ty.into(),
                        source_id,
                    };
                    let tsi_arc = Arc::new(tsi);
                    let type_id = TypeId::new(self.slab.insert_arc(tsi_arc.clone()));
                    v.insert_with_hasher(
                        ty_hash,
                        tsi_arc.clone(),
                        type_id,
                        make_hasher(&hash_builder, engines),
                    );
                    match replace_at_type_id {
                        Some(existing_id) => {
                            self.slab.replace_arc(existing_id.index(), tsi_arc);
                            existing_id
                        }
                        None => type_id,
                    }
                }
            }
        }
    }

    /// Computes the same hash as the [Hasher] returned by [make_hasher] but without
    /// allocating a new [TypeInfo] on the heap.
    fn compute_hash_without_heap_allocation(
        &self,
        engines: &Engines,
        hash_builder: &impl BuildHasher,
        type_info: &TypeInfo,
        source_id: &Option<SourceId>,
    ) -> u64 {
        let mut state = hash_builder.build_hasher();
        type_info.hash(&mut state, engines);
        source_id.hash(&mut state);
        state.finish()
    }

    /// Returns true if the `ty` is a type that can be replaced by using
    /// the [Self::replace] method during the type unification.
    fn is_replaceable_type(ty: &TypeInfo) -> bool {
        match ty {
            TypeInfo::Unknown
            | TypeInfo::Numeric
            | TypeInfo::Placeholder(_)
            | TypeInfo::UnknownGeneric { .. }
            | TypeInfo::Array(.., Length(ConstGenericExpr::AmbiguousVariableExpression { .. }))
            | TypeInfo::Struct(_)
            | TypeInfo::Enum(_) => true,
            TypeInfo::ContractCaller { abi_name, address } => {
                Self::is_replaceable_contract_caller(abi_name, address)
            }
            _ => false,
        }
    }

    fn is_replaceable_contract_caller(
        abi_name: &AbiName,
        address: &Option<Box<TyExpression>>,
    ) -> bool {
        address.is_none() || matches!(abi_name, AbiName::Deferred)
    }

    /// Returns true if the `ty` is a shareable type.
    ///
    /// A shareable type instance can be reused by the engine and is put into the [Self::shareable_types].
    fn is_type_shareable(&self, engines: &Engines, ty: &TypeInfo) -> bool {
        !(self.is_type_changeable(engines, ty) || self.is_type_distinguishable_by_annotations(ty))
    }

    /// Returns true if the `ty` is a changeable type. A changeable type is either:
    /// - a type that can be replaced during the type unification (by calling [Self::replace]).
    ///   We call such types replaceable types. A typical example would be [TypeInfo::UnknownGeneric].
    /// - or a type that is recursively defined over one or more replaceable types. E.g., a
    ///   generic enum type `SomeEnum<T>` that is still not monomorphized is a changeable type.
    ///   Note that a monomorphized version of `SomeEnum`, like e.g., `SomeEnum<u64>` *is not
    ///   changeable*.
    ///
    /// Note that the changeability of a type is tightly related to the unification process
    /// and the process of handling the types within the [TypeEngine]. As such, it is not
    /// seen as a property of a type itself, but rather as an information on how the [TypeEngine]
    /// treats the type. That's why the definition of the changeability of a type resides
    /// inside of the [TypeEngine].
    pub(crate) fn is_type_changeable(&self, engines: &Engines, ty: &TypeInfo) -> bool {
        let decl_engine = engines.de();
        let parsed_decl_engine = engines.pe();
        match ty {
            // Shareable built-in types are unchangeable by definition.
            // These type have only one shared `TypeInfo` instance per type
            // (and one for each unsigned integer).
            TypeInfo::StringSlice
            | TypeInfo::UnsignedInteger(_)
            | TypeInfo::Boolean
            | TypeInfo::B256
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice
            | TypeInfo::ErrorRecovery(_)
            | TypeInfo::Contract
            | TypeInfo::Never => false,

            // `StringArray`s are not changeable. We will have one shared
            // `TypeInfo` instance for every string size. Note that in case
            // of explicitly defined string arrays, e.g. in the storage or type ascriptions
            // like `str[5]`, we can also have different instances for string
            // arrays of the same size, because the `Length` in that case contains
            // as well the span of the size (`5` in the example).
            TypeInfo::StringArray(_) => false,

            // Replaceable types are, by definition, changeable.
            TypeInfo::Unknown
            | TypeInfo::Numeric
            | TypeInfo::Placeholder(_)
            | TypeInfo::TypeParam(_)
            | TypeInfo::UnknownGeneric { .. } => true,

            // The `ContractCaller` can be replaceable, and thus, sometimes changeable.
            TypeInfo::ContractCaller { abi_name, address } => {
                Self::is_replaceable_contract_caller(abi_name, address)
            }

            // For the types are defined over other types, inspect recursively their constituting types.
            TypeInfo::Enum(decl_id) => {
                let decl = decl_engine.get_enum(decl_id);
                self.is_changeable_enum(engines, &decl)
            }
            TypeInfo::UntypedEnum(decl_id) => {
                let decl = parsed_decl_engine.get_enum(decl_id);
                self.is_changeable_untyped_enum(engines, &decl)
            }
            TypeInfo::Struct(decl_id) => {
                let decl = decl_engine.get_struct(decl_id);
                self.is_changeable_struct(engines, &decl)
            }
            TypeInfo::UntypedStruct(decl_id) => {
                let decl = parsed_decl_engine.get_struct(decl_id);
                self.is_changeable_untyped_struct(engines, &decl)
            }
            TypeInfo::Tuple(elements) => self.is_changeable_tuple(engines, elements),

            // Currently, we support only non-generic aliases. Which means the alias
            // will never be changeable.
            // TODO: (GENERIC-TYPE-ALIASES) If we ever introduce generic type aliases, update this accordingly.
            TypeInfo::Alias { name: _, ty: _ } => false,

            // The following types are changeable if their type argument is changeable.
            TypeInfo::Array(ta, _)
            | TypeInfo::Slice(ta)
            | TypeInfo::Ptr(ta)
            | TypeInfo::Ref {
                referenced_type: ta,
                ..
            } => self.is_changeable_type_argument(engines, ta),

            // TODO: Improve handling of `TypeInfo::Custom` and `TypeInfo::TraitType`` within the `TypeEngine`:
            //       https://github.com/FuelLabs/sway/issues/6601
            TypeInfo::Custom { .. } => true,
            TypeInfo::TraitType { .. } => false,
        }
    }

    /// Returns true if two [TypeInfo] instances that are equal (with engines) and have same hashes (with engines)
    /// should potentially still be treated, within the type engine, as different types.
    ///
    /// [TypeParameter]s, [TypeArgument]s, and [Length]s can be "annotated". This means that, aside from the information they
    /// provide, like, e.g., [TypeArgument::type_id] or [Length::val], they can also, optionally, provide additional information
    /// most notably various spans.
    ///
    /// Same is with [Ident]s. From the hashing and equality (with engines) perspective, only the string value matters,
    /// but from the strict equality point of view, [Ident]'s span is also relevant.
    ///
    /// Thus, from the unification and type equivalence perspective, two [TypeArgument]s with the same `type_id` represent
    /// the same type. But if those two type arguments differ in their annotations, the [TypeEngine] must be able to distinguish between
    /// the equal (from the unification perspective) types that use those two different type arguments.
    ///
    /// In this example:
    ///
    /// ```ignore
    ///   let a: [u64;3] = (0, 0, 0);
    ///   let b: [u64;3] = (0, 0, 0);
    /// ```
    ///
    /// `a` and `b` will have the same type, but the span annotations in their [TypeArgument]s and [Length]s will be different
    /// (different spans pointing to two "u64"s and two "3"s) and thus the [TypeEngine] must treat those two types as two
    /// different types and when inserting them, assign them two different [TypeId]s, although the types themselves are not
    /// changeable.
    ///
    /// To sum it up:
    /// - if the `ty` consists of [TypeArgument]s, [TypeParameter]s, or [Length]s, they myst be check for annotations.
    /// - if the `ty` contains, e.g., [Ident]s, it is considered to be distinguishable by annotations.
    fn is_type_distinguishable_by_annotations(&self, ty: &TypeInfo) -> bool {
        match ty {
            // Types that do not have any annotations.
            TypeInfo::StringSlice
            | TypeInfo::UnsignedInteger(_)
            | TypeInfo::Boolean
            | TypeInfo::B256
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice
            | TypeInfo::ErrorRecovery(_)
            | TypeInfo::Never
            | TypeInfo::Unknown
            | TypeInfo::Numeric
            | TypeInfo::Contract => false,

            // Types that are always distinguishable because they have the `name: Ident`.
            //
            // Let's explain this in more detail, taking the `TypeInfo::Alias` as an example.
            // `TypeInfo::Alias` consists of the `name: Ident` and the `ty: TypeArgument`.
            //
            // Consider that we have two aliases with the same name and aliasing the same type but defined in different modules.
            // Thus, they would be two _different_ alias types. But because the spans in the `name` and `ty` do not count
            // neither for the equality check nor for the hash calculation, those two types will always be equal (with engines)
            // and also have the same hashes (with engines).
            //
            // This means that the `TypeEngine` would see them as the same type which would be wrong.
            // The fact that they are always distinguishable by annotations (span in the `name` and spans in the `ty`)
            // is actually a fortunate fact here, because it will help the `TypeEngine` to distinguish them.
            //
            // The consequence of this fact is, that all `TypeInfo::Alias`es are _always distinguishable by annotations_.
            //
            // The downside is that repeated usages of an actually *same* alias type will create
            // unnecessary new instances in the `TypeEngine` for every usage :-(
            //
            // Luckily, `TraitType`s and `Alias`es are rarely used and the number of their instances
            // within the `TypeEngine` will always be negligible, so we don't need to worry about this downside.
            // (At the time of writing this comment, out of ~200,000 types in the `TypeEngine` in a
            // realistic real-world project only ~20 were type aliases and only ~5 were trait types.)
            // And the `UnknownGeneric` is anyhow a changeable type.
            TypeInfo::UnknownGeneric { .. }
            // | TypeInfo::TraitType { .. }
            | TypeInfo::Alias { .. } => true,

            TypeInfo::StringArray(l) => l.is_annotated(),

            // If the contract caller has the `abi_name` defined (AbiName::Know) the span information
            // that comes with the `Ident`s of the `CallPath` is not relevant for the equality
            // and hashing (with engines) but makes two same names distinguishable. The same thing is
            // with the `address` expression. It can be, e.g., the same literal, but it will have different
            // spans. Moreover, the same `abi_name`, depending on the context, can represent different
            // ABI declarations, like in the example below:
            //
            //   fn a() {
            //       use ::lib_a::Abi as Abi; // <<<--- `Abi` coming from `lib_**a**`.
            //       let _ = abi(Abi, 0x1111111111111111111111111111111111111111111111111111111111111111);
            //   }

            //   fn b() {
            //       use ::lib_b::Abi as Abi; // <<<--- `Abi` coming from `lib_**b**`.
            //       let _ = abi(Abi, 0x1111111111111111111111111111111111111111111111111111111111111111);
            //   }
            //
            // This all means, if a `ContractCaller` has either the `abi_name` or the `address` defined,
            // it is distinguishable by annotations.
            TypeInfo::ContractCaller { abi_name, address } => Self::is_contract_caller_distinguishable_by_annotations(abi_name, address),

            // Enums `decl`s are either coming from enum declarations,
            // or from their monomorphizations. If an enum declaration is generic,
            // its type parameters will always be annotated, having, e.g., spans of
            // generic parameter names, like, e.g., "T".
            // Also, all the enum variants have `TypeArguments` that are _always_ annotated
            // with the type span.
            //
            // In other words, all `TyEnumDecl`s are annotated.
            // The question is, if the monomorphization can produce two same `decl`s that
            // are differently annotated. E.g., when unifying the generic parameter "T" with "u64",
            // like in the below example, will the span of the type parameter "T" change
            // to "u64":
            //
            //   let _ = GenericEnum::<u64>::A(42u64);
            //   let _ = GenericEnum::<u64>::A(42u64);
            //
            // In that case, the two equal `decl`s obtained via two monomorphizations above,
            // would be differently annotated, and thus, distinguishable by annotations.
            //
            // The answer is *no*. The monomorphization changes only the `TypeId`s of the
            // `TypeParameter`s and `TypeArgument`s but leaves the original spans untouched.
            // Therefore, we can never end up in a situation that an annotation differs from
            // the one in the original `TyEnumDecl` coming from the enum declaration.
            //
            // Thus, any two equal `TyEnumDecl`s are never distinguishable by annotations.
            TypeInfo::Enum(_) => false,

            // The same argument as above applies to struct and `TyStructDecl`s.
            TypeInfo::Struct(_) => false,

            // TODO: (UNTYPED-TYPES) Reassess this once `UntypedEnum` and `UntypedStruct`
            //       start getting logic.
            TypeInfo::UntypedEnum(_) => false,

            // TODO: (UNTYPED-TYPES) Reassess this once `UntypedEnum` and `UntypedStruct`
            //       start getting logic.
            TypeInfo::UntypedStruct(_) => false,

            // Equal (with engines) tuple types can have different annotations and are in that case
            // distinguishable by those annotations. E.g., in the example below, the two
            // `(u64, u8)` tuples will have different spans for two "u64"s and two "u8"s.
            //
            //   let _: (u64, u8) = (64u64, 8u8);
            //   let _: (u64, u8) = (64u64, 8u8);
            //
            // Note that _all the tuples used in code will always be distinguishable by annotations_,
            // because they will always have spans either pointing to the values like in `(64u64, 8u8)`
            // or to types like in `(u64, u8)`.
            //
            // Only the tuples used in generated code will not be distinguishable by annotations,
            // as well as tuples representing unit types.
            TypeInfo::Tuple(elements) => self.is_tuple_distinguishable_by_annotations(elements),

            // The below types are those have `TypeArgument`s (`ta`s) in their definitions.
            // Note that we are checking only if those `ta`s are annotated, but not
            // recursively if the types they "reference" are annotated ;-)
            // We don't need to recursively check if the types behind
            // the `ta.type_id`s are distinguishable by annotations.
            // This is because two equal (with engines) parent types
            // containing `ta`s that pass the above check are also equal in
            // case when their full type argument content is compared,
            // because the type arguments will be equal in all their fields.

            TypeInfo::Slice(ta)
            | TypeInfo::Ptr(ta)
            | TypeInfo::Ref { referenced_type: ta, .. } => ta.is_annotated(),

            TypeInfo::Array(ta, l) => {
                ta.is_annotated() || l.expr().is_annotated()
            }

            // The above reasoning for `TypeArgument`s applies also for the `TypeParameter`s.
            // We only need to check if the `tp` is annotated.
            TypeInfo::TypeParam(tp) | TypeInfo::Placeholder(tp) => {
                let tp = tp.as_type_parameter().expect("only works with type parameters");
                tp.is_annotated()
            },

            // TODO: Improve handling of `TypeInfo::Custom` and `TypeInfo::TraitType`` within the `TypeEngine`:
            //       https://github.com/FuelLabs/sway/issues/6601
            TypeInfo::Custom { .. } => true,
            TypeInfo::TraitType { .. } => false,
        }
    }

    fn is_tuple_distinguishable_by_annotations(&self, elements: &[GenericArgument]) -> bool {
        if elements.is_empty() {
            false
        } else {
            elements.iter().any(|ta| ta.is_annotated())
        }
    }

    fn is_contract_caller_distinguishable_by_annotations(
        abi_name: &AbiName,
        address: &Option<Box<TyExpression>>,
    ) -> bool {
        address.is_some() || matches!(abi_name, AbiName::Known(_))
    }

    /// Returns true if the `type_id` represents a changeable type.
    /// For the type changeability see [Self::is_type_changeable].
    fn is_type_id_of_changeable_type(&self, engines: &Engines, type_id: TypeId) -> bool {
        self.is_type_changeable(engines, &self.slab.get(type_id.index()).type_info)
    }

    fn is_changeable_type_argument(&self, engines: &Engines, ta: &GenericArgument) -> bool {
        self.is_type_id_of_changeable_type(engines, ta.type_id())
    }

    fn is_changeable_enum(&self, engines: &Engines, decl: &TyEnumDecl) -> bool {
        self.are_changeable_type_parameters(engines, &decl.generic_parameters)
        // TODO: Remove once https://github.com/FuelLabs/sway/issues/6687 is fixed.
        ||
        self.module_might_outlive_type_parameters(engines, decl.span.source_id(), &decl.generic_parameters)
    }

    fn is_changeable_untyped_enum(&self, engines: &Engines, decl: &EnumDeclaration) -> bool {
        self.are_changeable_type_parameters(engines, &decl.type_parameters)
        // TODO: Remove once https://github.com/FuelLabs/sway/issues/6687 is fixed.
        ||
        self.module_might_outlive_type_parameters(engines, decl.span.source_id(), &decl.type_parameters)
    }

    fn is_changeable_struct(&self, engines: &Engines, decl: &TyStructDecl) -> bool {
        self.are_changeable_type_parameters(engines, &decl.generic_parameters)
        // TODO: Remove once https://github.com/FuelLabs/sway/issues/6687 is fixed.
        ||
        self.module_might_outlive_type_parameters(engines, decl.span.source_id(), &decl.generic_parameters)
    }

    fn is_changeable_untyped_struct(&self, engines: &Engines, decl: &StructDeclaration) -> bool {
        self.are_changeable_type_parameters(engines, &decl.type_parameters)
        // TODO: Remove once https://github.com/FuelLabs/sway/issues/6687 is fixed.
        ||
        self.module_might_outlive_type_parameters(engines, decl.span.source_id(), &decl.type_parameters)
    }

    fn is_changeable_tuple(&self, engines: &Engines, elements: &[GenericArgument]) -> bool {
        if elements.is_empty() {
            false
        } else {
            elements
                .iter()
                .any(|ta| self.is_type_id_of_changeable_type(engines, ta.type_id()))
        }
    }

    fn are_changeable_type_parameters(
        &self,
        engines: &Engines,
        type_parameters: &[TypeParameter],
    ) -> bool {
        if type_parameters.is_empty() {
            false
        } else {
            type_parameters.iter().any(|tp| match tp {
                TypeParameter::Type(p) => self.is_type_id_of_changeable_type(engines, p.type_id),
                TypeParameter::Const(_) => true,
            })
        }
    }

    // TODO: Remove this and all `module_might_outlive_xyz` methods once https://github.com/FuelLabs/sway/issues/6687 is fixed.
    //
    //       This method represents the best effort to partially mitigate the issue
    //       described in https://github.com/FuelLabs/sway/issues/6687, by doing changes only in the `TypeEngine`.
    //
    //       Enum and struct types use it to restrict their shareability and reduce the chance of accessing
    //       GCed types.
    //
    //       The method takes an **existing** `type_id` and the source id of a particular module (`module_source_id`)
    //       and checks if the module represented by the `module_source_id` **might** survive LSP garbage collection
    //       even if the module to which `type_id` is bound gets GCed.
    //
    //       E.g., if the `module_source_id` points to the `Option` declaration of a monomorphized `Option<MyStruct>`,
    //       this method will return true if the `type_id` represents `MyStruct`, because if the `MyStruct`'s module
    //       gets GCed, the `Option`'s module will "survive" and outlive it, thus pointing via its `TypeArgument` to a
    //       non-existing, GCed, `MyStruct` type.
    //
    //       E.g., if the `module_source_id` points to the `Option` declaration of a monomorphized `Option<u64>`,
    //       this method will return false if the `type_id` represents `u64`, because the `u64` is not bound to
    //       any module, and thus, can never be GCed. This means that the `Option`'s module can never outlive it.
    fn module_might_outlive_type(
        &self,
        engines: &Engines,
        module_source_id: Option<&SourceId>,
        type_id: TypeId,
    ) -> bool {
        fn module_might_outlive_type_source_id(
            module_source_id: Option<&SourceId>,
            type_source_id: Option<SourceId>,
        ) -> bool {
            // If the type represented by the `type_id` is not bound to a source id (`type_source_id.is_none()`)
            // it cannot be outlived by the module.
            // Otherwise, if `type_source_id.is_some()` but is the same as the `module_source_id`, it can be GCed only if
            // the `module_source_id` is GCed.
            // Otherwise, we cannot guarantee that the module will not outlive the type's module and we must
            // be pessimistic and return false.
            type_source_id.is_some() && type_source_id != module_source_id.copied()
        }

        let tsi = self.slab.get(type_id.index());
        let type_info = &*tsi.type_info;
        let type_source_id = tsi.source_id;

        let decl_engine = engines.de();
        let parsed_decl_engine = engines.pe();

        // We always must check the `type_id` itself, like, e.h., `MyStruct` in `Option<MyStruct>`, ...
        module_might_outlive_type_source_id(module_source_id, type_source_id)
        ||
        // ... and also all types it transitively depends on, like, e.g., in `Option<Option<MyStruct>>`.
        match type_info {
            // If a type does not transitively depends on other types, just return `false`.
            TypeInfo::StringSlice
            | TypeInfo::UnsignedInteger(_)
            | TypeInfo::Boolean
            | TypeInfo::B256
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice
            | TypeInfo::ErrorRecovery(_)
            | TypeInfo::Contract
            | TypeInfo::Never => false,

            // Note that `TypeParam` is currently not used at all.
            TypeInfo::TypeParam(_) => false,

            TypeInfo::StringArray(_) => false,

            TypeInfo::Unknown
            | TypeInfo::Numeric => false,

            TypeInfo::Placeholder(tp) => self.module_might_outlive_type_parameter(engines, module_source_id, tp),
            TypeInfo::UnknownGeneric { trait_constraints, parent, .. } => {
                parent.is_some_and(|parent_type_id| self.module_might_outlive_type(engines, module_source_id, parent_type_id))
                ||
                self.module_might_outlive_trait_constraints(engines, module_source_id, trait_constraints)
            },

            TypeInfo::ContractCaller { .. } => false,

            TypeInfo::Enum(decl_id) => {
                let decl = decl_engine.get_enum(decl_id);
                self.module_might_outlive_type_parameters(engines, module_source_id, &decl.generic_parameters)
            }
            TypeInfo::UntypedEnum(decl_id) => {
                let decl = parsed_decl_engine.get_enum(decl_id);
                self.module_might_outlive_type_parameters(engines, module_source_id, &decl.type_parameters)
            }
            TypeInfo::Struct(decl_id) => {
                let decl = decl_engine.get_struct(decl_id);
                self.module_might_outlive_type_parameters(engines, module_source_id, &decl.generic_parameters)
            }
            TypeInfo::UntypedStruct(decl_id) => {
                let decl = parsed_decl_engine.get_struct(decl_id);
                self.module_might_outlive_type_parameters(engines, module_source_id, &decl.type_parameters)
            }
            TypeInfo::Tuple(elements) => self.module_might_outlive_type_arguments(engines, module_source_id, elements),

            TypeInfo::Alias { ty, .. } => self.module_might_outlive_type_argument(engines, module_source_id, ty),

            TypeInfo::Array(ta, _)
            | TypeInfo::Slice(ta)
            | TypeInfo::Ptr(ta)
            | TypeInfo::Ref {
                referenced_type: ta,
                ..
            } => self.module_might_outlive_type_argument(engines, module_source_id, ta),

            TypeInfo::Custom { type_arguments, .. } =>
                type_arguments.as_ref().is_some_and(|type_arguments|
                    self.module_might_outlive_type_arguments(engines, module_source_id, type_arguments)),
            TypeInfo::TraitType { trait_type_id, .. } => self.module_might_outlive_type(engines, module_source_id, *trait_type_id)
        }
    }

    fn module_might_outlive_type_parameter(
        &self,
        engines: &Engines,
        module_source_id: Option<&SourceId>,
        type_parameter: &TypeParameter,
    ) -> bool {
        let type_parameter = type_parameter
            .as_type_parameter()
            .expect("only works with type parameters");
        self.module_might_outlive_type(engines, module_source_id, type_parameter.type_id)
            || self.module_might_outlive_type(
                engines,
                module_source_id,
                type_parameter.initial_type_id,
            )
            || self.module_might_outlive_trait_constraints(
                engines,
                module_source_id,
                &type_parameter.trait_constraints,
            )
    }

    fn module_might_outlive_type_parameters(
        &self,
        engines: &Engines,
        module_source_id: Option<&SourceId>,
        type_parameters: &[TypeParameter],
    ) -> bool {
        if type_parameters.is_empty() {
            false
        } else {
            type_parameters
                .iter()
                .filter(|x| x.as_type_parameter().is_some())
                .any(|tp| self.module_might_outlive_type_parameter(engines, module_source_id, tp))
        }
    }

    fn module_might_outlive_type_argument(
        &self,
        engines: &Engines,
        module_source_id: Option<&SourceId>,
        type_argument: &GenericArgument,
    ) -> bool {
        self.module_might_outlive_type(engines, module_source_id, type_argument.type_id())
            || self.module_might_outlive_type(
                engines,
                module_source_id,
                type_argument.initial_type_id(),
            )
    }

    fn module_might_outlive_type_arguments(
        &self,
        engines: &Engines,
        module_source_id: Option<&SourceId>,
        type_arguments: &[GenericArgument],
    ) -> bool {
        if type_arguments.is_empty() {
            false
        } else {
            type_arguments
                .iter()
                .any(|ta| self.module_might_outlive_type_argument(engines, module_source_id, ta))
        }
    }

    fn module_might_outlive_trait_constraint(
        &self,
        _engines: &Engines,
        _module_source_id: Option<&SourceId>,
        trait_constraint: &TraitConstraint,
    ) -> bool {
        // `TraitConstraint`s can contain `TypeArgument`s that can cause endless recursion,
        // unless we track already visited types. This happens in cases of recursive generic
        // traits like, e.g., `T: Trait<T>`.
        //
        // We deliberately decide not to track visited types because:
        //  - `module_might_outlive_type` represents a best effort to mitigate the issue of modules outliving their types.
        //    It is already not exact.
        //  - trait constraints with type arguments are rather rare.
        //  - tracking already visited types is expensive and `module_might_outlive_type` already adds an overhead.
        //
        // Instead, if the `trait_constraint` contains type arguments, we will bail out and
        // conclude that the module might outlive the types in the trait constraint.
        !trait_constraint.type_arguments.is_empty()
    }

    fn module_might_outlive_trait_constraints(
        &self,
        engines: &Engines,
        module_source_id: Option<&SourceId>,
        trait_constraint: &[TraitConstraint],
    ) -> bool {
        trait_constraint
            .iter()
            .any(|tc| self.module_might_outlive_trait_constraint(engines, module_source_id, tc))
    }

    // In the `is_shareable_<type>` methods we reuse the logic from the
    // `is_type_changeable` and `is_type_distinguishable_by_annotations`.
    // This is a very slight and minimal copy of logic, that improves performance
    // (Inlining, no additional fetching from the `DeclEngine`, no intensive function
    // calls, etc. Don't forget that this is all on a *very* hot path.)

    fn is_shareable_enum(&self, engines: &Engines, decl: &TyEnumDecl) -> bool {
        // !(self.is_changeable_enum(decl_engine, decl) || false)
        !self.is_changeable_enum(engines, decl)
    }

    fn is_shareable_struct(&self, engines: &Engines, decl: &TyStructDecl) -> bool {
        // !(self.is_changeable_struct(decl_engine, decl) || false)
        !self.is_changeable_struct(engines, decl)
    }

    fn is_shareable_tuple(&self, engines: &Engines, elements: &[GenericArgument]) -> bool {
        !(self.is_changeable_tuple(engines, elements)
            || self.is_tuple_distinguishable_by_annotations(elements))
    }

    fn is_shareable_array(
        &self,
        engines: &Engines,
        elem_type: &GenericArgument,
        length: &Length,
    ) -> bool {
        !(self.is_changeable_type_argument(engines, elem_type)
            || elem_type.is_annotated()
            || length.expr().is_annotated())
    }

    fn is_shareable_string_array(&self, length: &NumericLength) -> bool {
        // !(false || length.is_annotated())
        !length.is_annotated()
    }

    fn is_shareable_slice(&self, engines: &Engines, elem_type: &GenericArgument) -> bool {
        !(self.is_changeable_type_argument(engines, elem_type) || elem_type.is_annotated())
    }

    fn is_shareable_ptr(&self, engines: &Engines, pointee_type: &GenericArgument) -> bool {
        !(self.is_changeable_type_argument(engines, pointee_type) || pointee_type.is_annotated())
    }

    fn is_shareable_ref(&self, engines: &Engines, referenced_type: &GenericArgument) -> bool {
        !(self.is_changeable_type_argument(engines, referenced_type)
            || referenced_type.is_annotated())
    }

    // TODO: This and other, type-specific, methods that calculate `source_id` have `fallback` in their name.
    //       They will be called if the `source_id` is not provided in `new` or `insert` methods,
    //       thus, fallbacks.
    //       However, some of them actually do calculate the appropriate `source_id` which will for certain
    //       types and situations always be extracted from the `TypeInfo` and not allowed to be provided
    //       in `new` or `insert` methods.
    //       Until https://github.com/FuelLabs/sway/issues/6603 gets implemented, we will call them all
    //       fallbacks, which corresponds to the current usage, and eventually rename them accordingly
    //       once we optimize the `TypeEngine` for garbage collection (#6603).

    fn get_type_fallback_source_id(&self, engines: &Engines, ty: &TypeInfo) -> Option<SourceId> {
        let decl_engine = engines.de();
        let parsed_decl_engine = engines.pe();
        match ty {
            TypeInfo::Unknown
            | TypeInfo::Never
            | TypeInfo::TypeParam(_)
            | TypeInfo::UnsignedInteger(_)
            | TypeInfo::Boolean
            | TypeInfo::B256
            | TypeInfo::Numeric
            | TypeInfo::ErrorRecovery(_)
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice
            | TypeInfo::Contract
            | TypeInfo::StringSlice => None,

            TypeInfo::UnknownGeneric { .. } => Self::get_unknown_generic_fallback_source_id(ty),
            TypeInfo::Placeholder(_) => self.get_placeholder_fallback_source_id(ty),
            TypeInfo::StringArray(length) => Self::get_source_id_from_spanned(length),
            TypeInfo::Enum(decl_id) => {
                let decl = decl_engine.get_enum(decl_id);
                Self::get_enum_fallback_source_id(&decl)
            }
            TypeInfo::UntypedEnum(decl_id) => {
                let decl = parsed_decl_engine.get_enum(decl_id);
                Self::get_untyped_enum_fallback_source_id(&decl)
            }
            TypeInfo::Struct(decl_id) => {
                let decl = decl_engine.get_struct(decl_id);
                Self::get_struct_fallback_source_id(&decl)
            }
            TypeInfo::UntypedStruct(decl_id) => {
                let decl = parsed_decl_engine.get_struct(decl_id);
                Self::get_untyped_struct_fallback_source_id(&decl)
            }
            TypeInfo::Tuple(elements) => self.get_tuple_fallback_source_id(elements),
            TypeInfo::Array(elem_type, length) => {
                self.get_array_fallback_source_id(elem_type, length)
            }

            TypeInfo::ContractCaller { abi_name, address } => {
                Self::get_contract_caller_fallback_source_id(abi_name, address)
            }

            TypeInfo::Alias { name, ty } => self.get_alias_fallback_source_id(name, ty),

            TypeInfo::Ptr(ta)
            | TypeInfo::Slice(ta)
            | TypeInfo::Ref {
                referenced_type: ta,
                ..
            } => self.get_source_id_from_type_argument(ta),

            TypeInfo::Custom {
                qualified_call_path,
                type_arguments,
            } => self.get_custom_fallback_source_id(qualified_call_path, type_arguments),

            TypeInfo::TraitType {
                name,
                trait_type_id,
            } => self.get_trait_type_fallback_source_id(name, trait_type_id),
        }
    }

    fn get_source_id_from_spanned(item: &impl Spanned) -> Option<SourceId> {
        item.span().source_id().copied()
    }

    fn get_source_id_from_type_argument(&self, ta: &GenericArgument) -> Option<SourceId> {
        // If the `ta` is span-annotated, take the source id from its `span`,
        // otherwise, take the source id of the type it represents.
        ta.span()
            .source_id()
            .copied()
            .or_else(|| self.get_type_source_id(ta.type_id()))
    }

    fn get_source_id_from_type_arguments(
        &self,
        type_arguments: &[GenericArgument],
    ) -> Option<SourceId> {
        // For type arguments, if they are annotated, we take the use site source file.
        // In semantically valid usages, in a vector of `TypeArgument`s, the use site source file
        // will be the same for all the elements, so we are taking it from the
        // first one that is annotated (which will be the first one or none).
        // E.g., in a `TypeInfo::Tuple`, all the `TypeArgument`s will either not be annotated, or will
        // all be annotated and situated within the same file.
        //
        // If the type arguments are not annotated, we are taking the source file of the first type
        // pointed by a type argument, that has a source id.
        if type_arguments.is_empty() {
            None
        } else {
            type_arguments
                .iter()
                .find_map(|ta| ta.span().source_id().copied())
                .or_else(|| {
                    type_arguments
                        .iter()
                        .find_map(|ta| self.get_type_source_id(ta.type_id()))
                })
        }
    }

    fn get_source_id_from_type_parameter(&self, tp: &TypeParameter) -> Option<SourceId> {
        // If the `tp` is span-annotated, take the source id from its `span`,
        // otherwise, take the source id of the type it represents.
        tp.name().span().source_id().copied().or_else(|| match tp {
            TypeParameter::Type(p) => self.get_type_source_id(p.type_id),
            TypeParameter::Const(_) => None,
        })
    }

    fn get_placeholder_fallback_source_id(&self, placeholder: &TypeInfo) -> Option<SourceId> {
        // `TypeInfo::Placeholder` is an always replaceable type and we know we will
        // get a new instance of it in the engine for every "_" occurrence. This means
        // that it can never happen that instances from different source files point
        // to the same `TypeSourceInfo`. Therefore, we can safely remove an instance
        // of a `Placeholder` from the engine if its source file is garbage collected.
        //
        // The source file itself is always the one in which the `name`, means "_",
        // is situated.
        let TypeInfo::Placeholder(tp) = &placeholder else {
            unreachable!("The `placeholder` is checked to be of variant `TypeInfo::Placeholder`.");
        };

        self.get_source_id_from_type_parameter(tp)
    }

    fn get_type_parameter_fallback_source_id(&self, type_param: &TypeInfo) -> Option<SourceId> {
        // `TypeInfo::TypeParam` is an always replaceable type and we know we will
        // get a new instance of it in the engine for every trait type parameter occurrence. This means
        // that it can never happen that instances from different source files point
        // to the same `TypeSourceInfo`. Therefore, we can safely remove an instance
        // of a `TypeParam` from the engine if its source file is garbage collected.
        //
        // The source file itself is always the one in which the `name` is situated.
        let TypeInfo::TypeParam(tp) = &type_param else {
            unreachable!("The `placeholder` is checked to be of variant `TypeInfo::TypeParam`.");
        };

        self.get_source_id_from_type_parameter(tp)
    }

    fn get_unknown_generic_fallback_source_id(unknown_generic: &TypeInfo) -> Option<SourceId> {
        // `TypeInfo::UnknownGeneric` is an always replaceable type and we know we will
        // get a new instance of it in the engine for every, e.g., "<T1>", "<T2>", etc. occurrence.
        // This means that it can never happen that instances from different source files point
        // to the same `TypeSourceInfo`. Therefore, we can safely remove an instance
        // of an `UnknownGeneric` from the engine if its source file is garbage collected.
        //
        // The source file itself is always the one in which the `name`, means, e.g. "T1", "T2", etc.
        // is situated.
        let TypeInfo::UnknownGeneric { name, .. } = &unknown_generic else {
            unreachable!(
                "The `unknown_generic` is checked to be of variant `TypeInfo::UnknownGeneric`."
            );
        };
        name.span().source_id().copied()
    }

    fn get_enum_fallback_source_id(decl: &TyEnumDecl) -> Option<SourceId> {
        // For `TypeInfo::Enum`, we are taking the source file in which the enum is declared.
        decl.span.source_id().copied()
    }

    fn get_untyped_enum_fallback_source_id(decl: &EnumDeclaration) -> Option<SourceId> {
        // For `TypeInfo::UntypedEnum`, we are taking the source file in which the enum is declared.
        decl.span.source_id().copied()
    }

    fn get_struct_fallback_source_id(decl: &TyStructDecl) -> Option<SourceId> {
        // For `TypeInfo::Struct`, we are taking the source file in which the struct is declared.
        decl.span.source_id().copied()
    }

    fn get_untyped_struct_fallback_source_id(decl: &StructDeclaration) -> Option<SourceId> {
        // For `TypeInfo::UntypedStruct`, we are taking the source file in which the struct is declared.
        decl.span.source_id().copied()
    }

    fn get_tuple_fallback_source_id(&self, elements: &[GenericArgument]) -> Option<SourceId> {
        self.get_source_id_from_type_arguments(elements)
    }

    fn get_array_fallback_source_id(
        &self,
        elem_type: &GenericArgument,
        _length: &Length,
    ) -> Option<SourceId> {
        self.get_source_id_from_type_argument(elem_type)
    }

    fn get_string_array_fallback_source_id(length: &NumericLength) -> Option<SourceId> {
        // For `TypeInfo::StringArray`, if it is annotated, we take the use site source file found in the `length`.
        Self::get_source_id_from_spanned(length)
    }

    fn get_contract_caller_fallback_source_id(
        abi_name: &AbiName,
        address: &Option<Box<TyExpression>>,
    ) -> Option<SourceId> {
        // For `TypeInfo::ContractCaller`, if it has an `address`, we take the use site source file found in the it.
        // Otherwise, if it has an `abi_name`, we take the source file of the ABI definition.
        match address {
            Some(addr_expr) => addr_expr.span.source_id().copied(),
            None => {
                if let AbiName::Known(name) = abi_name {
                    name.span().source_id().copied()
                } else {
                    None
                }
            }
        }
    }

    fn get_alias_fallback_source_id(&self, name: &Ident, ty: &GenericArgument) -> Option<SourceId> {
        // For `TypeInfo::Alias`, we take the source file in which the alias is declared, if it exists.
        // Otherwise, we take the source file of the aliased type `ty`.
        name.span()
            .source_id()
            .copied()
            .or_else(|| self.get_source_id_from_type_argument(ty))
    }

    fn get_slice_fallback_source_id(&self, elem_type: &GenericArgument) -> Option<SourceId> {
        self.get_source_id_from_type_argument(elem_type)
    }

    fn get_ptr_fallback_source_id(&self, pointee_type: &GenericArgument) -> Option<SourceId> {
        self.get_source_id_from_type_argument(pointee_type)
    }

    fn get_ref_fallback_source_id(&self, referenced_type: &GenericArgument) -> Option<SourceId> {
        self.get_source_id_from_type_argument(referenced_type)
    }

    fn get_custom_fallback_source_id(
        &self,
        qualified_call_path: &QualifiedCallPath,
        type_arguments: &Option<Vec<GenericArgument>>,
    ) -> Option<SourceId> {
        // For `TypeInfo::Custom`, we take the source file in which the custom type is used, extracted from the `qualified_call_path`.
        // For non-generated source code, this will always exists.
        // For potential situation of having a `qualified_call_path` without spans in generated code, we do a fallback and
        // extract the source id from the remaining parameters.
        qualified_call_path
            .call_path
            .suffix
            .span()
            .source_id()
            .copied()
            .or_else(|| {
                type_arguments
                    .as_ref()
                    .and_then(|tas| self.get_source_id_from_type_arguments(tas))
            })
    }

    fn get_trait_type_fallback_source_id(
        &self,
        name: &Ident,
        trait_type_id: &TypeId,
    ) -> Option<SourceId> {
        // For `TypeInfo::TraitType`, we take the source file in which the trait type is declared, extracted from the `name`.
        // For non-generated source code, this will always exists.
        // For potential situation of having a `name` without spans in generated code, we do a fallback and
        // extract the source id from the `trait_type_id`.
        name.span()
            .source_id()
            .copied()
            .or_else(|| self.get_type_source_id(*trait_type_id))
    }

    /// Returns the known [SourceId] of a type that already exists
    /// in the [TypeEngine]. The type is given by its `type_id`.
    fn get_type_source_id(&self, type_id: TypeId) -> Option<SourceId> {
        self.slab.get(type_id.index()).source_id
    }

    fn clear_items<F>(&mut self, keep: F)
    where
        F: Fn(&SourceId) -> bool,
    {
        self.slab
            .retain(|_, tsi| tsi.source_id.as_ref().is_none_or(&keep));
        self.shareable_types
            .write()
            .retain(|tsi, _| tsi.source_id.as_ref().is_none_or(&keep));
    }

    /// Removes all data associated with `program_id` from the type engine.
    pub fn clear_program(&mut self, program_id: &ProgramId) {
        self.clear_items(|id| id.program_id() != *program_id);
    }

    /// Removes all data associated with `source_id` from the type engine.
    pub fn clear_module(&mut self, source_id: &SourceId) {
        self.clear_items(|id| id != source_id);
    }

    /// Replaces the replaceable type behind the `type_id` with the `new_value`.
    /// The existing source id will be preserved.
    ///
    /// Note that, if the `new_value` represents a shareable built-in type,
    /// the existing source id will be removed (replaced by `None`).
    ///
    /// Panics if the type behind the `type_id` is not a replaceable type.
    pub fn replace(&self, engines: &Engines, type_id: TypeId, new_value: TypeInfo) {
        // We keep the existing source id. `replace_with_new_source_id` is treated just as a common implementation.
        let source_id = self.slab.get(type_id.index()).source_id;
        self.replace_with_new_source_id(engines, type_id, new_value, source_id);
    }

    /// Replaces the replaceable type behind the `type_id` with the `new_value`.
    /// The existing source id will also be replaced with the `new_source_id`.
    ///
    /// Note that, if the `new_value` represents a shareable built-in type,
    /// the existing source id will be removed (replaced by `None`).
    ///
    /// Panics if the type behind the `type_id` is not a replaceable type.
    // TODO: Once https://github.com/FuelLabs/sway/issues/6603 gets implemented and we further optimize
    //       the `TypeEngine` for garbage collection, this variant of `replace` will actually not be
    //       needed any more. The version of `replace` that uses the initially provided `source_id` will
    //       be sufficient.
    pub fn replace_with_new_source_id(
        &self,
        engines: &Engines,
        id: TypeId,
        new_value: TypeInfo,
        new_source_id: Option<SourceId>,
    ) {
        let type_source_info = self.slab.get(id.index());
        assert!(
            Self::is_replaceable_type(&type_source_info.type_info),
            "The type requested to be replaced is not a replaceable type. The type was: {:#?}.",
            &type_source_info.type_info
        );

        if !type_source_info.equals(
            &new_value,
            &new_source_id,
            &PartialEqWithEnginesContext::new(engines),
        ) {
            self.touch_last_replace();
        }
        let is_shareable_type = self.is_type_shareable(engines, &new_value);
        // Shareable built-in types like, e.g., `u64`, should "live forever" and never be
        // garbage-collected. When replacing types, like e.g., unknown generics, that
        // might be bound to a source id, we will still remove that source id, if the
        // replaced type is replaced by a shareable built-in type. This maximizes the
        // reuse of sharable built-in types, and also ensures that they are never GCed.
        let source_id = if self.is_shareable_built_in_type(&new_value) {
            None
        } else {
            new_source_id
        };
        self.insert_or_replace_type_source_info(
            engines,
            new_value,
            source_id,
            is_shareable_type,
            Some(id),
        );
    }

    /// Performs a lookup of `id` into the [TypeEngine].
    pub fn get(&self, id: TypeId) -> Arc<TypeInfo> {
        self.slab.get(id.index()).type_info.clone()
    }

    pub fn map<R>(&self, id: TypeId, f: impl FnOnce(&TypeInfo) -> R) -> R {
        self.slab.map(id.index(), |x| f(x.type_info.as_ref()))
    }

    /// Performs a lookup of `id` into the [TypeEngine] recursing when finding a
    /// [TypeInfo::Alias].
    pub fn get_unaliased(&self, id: TypeId) -> Arc<TypeInfo> {
        // A slight infinite loop concern if we somehow have self-referential aliases, but that
        // shouldn't be possible.
        let tsi = self.slab.get(id.index());
        match &*tsi.type_info {
            TypeInfo::Alias { ty, .. } => self.get_unaliased(ty.type_id()),
            _ => tsi.type_info.clone(),
        }
    }

    /// Performs a lookup of `id` into the [TypeEngine] recursing when finding a
    /// [TypeInfo::Alias].
    pub fn get_unaliased_type_id(&self, id: TypeId) -> TypeId {
        // A slight infinite loop concern if we somehow have self-referential aliases, but that
        // shouldn't be possible.
        let tsi = self.slab.get(id.index());
        match &*tsi.type_info {
            TypeInfo::Alias { ty, .. } => self.get_unaliased_type_id(ty.type_id()),
            _ => id,
        }
    }

    /// Make the types of `received` and `expected` equivalent (or produce an
    /// error if there is a conflict between them).
    ///
    /// More specifically, this function tries to make `received` equivalent to
    /// `expected`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn unify(
        &self,
        handler: &Handler,
        engines: &Engines,
        received: TypeId,
        expected: TypeId,
        span: &Span,
        help_text: &str,
        err_override: Option<CompileError>,
    ) {
        Self::unify_helper(
            handler,
            engines,
            received,
            expected,
            span,
            help_text,
            err_override,
            UnifyKind::Default,
            true,
        );
    }

    /// Make the types of `received` and `expected` equivalent (or produce an
    /// error if there is a conflict between them).
    ///
    /// More specifically, this function tries to make `received` equivalent to
    /// `expected`, except in cases where `received` has more type information
    /// than `expected` (e.g. when `expected` is a self type and `received`
    /// is not).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn unify_with_self(
        &self,
        handler: &Handler,
        engines: &Engines,
        received: TypeId,
        expected: TypeId,
        span: &Span,
        help_text: &str,
        err_override: Option<CompileError>,
    ) {
        Self::unify_helper(
            handler,
            engines,
            received,
            expected,
            span,
            help_text,
            err_override,
            UnifyKind::WithSelf,
            true,
        );
    }

    /// Make the types of `received` and `expected` equivalent (or produce an
    /// error if there is a conflict between them).
    ///
    /// More specifically, this function tries to make `received` equivalent to
    /// `expected`, except in cases where `received` has more type information
    /// than `expected` (e.g. when `expected` is a generic type and `received`
    /// is not).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn unify_with_generic(
        &self,
        handler: &Handler,
        engines: &Engines,
        received: TypeId,
        expected: TypeId,
        span: &Span,
        help_text: &str,
        err_override: Option<CompileError>,
    ) {
        Self::unify_helper(
            handler,
            engines,
            received,
            expected,
            span,
            help_text,
            err_override,
            UnifyKind::WithGeneric,
            true,
        );
    }

    fn touch_last_replace(&self) {
        // eprintln!(
        //     "    touch_last_replace {}",
        //     std::backtrace::Backtrace::force_capture()
        // );
        let mut write_last_change = self.last_replace.write();
        *write_last_change = Instant::now();
    }

    #[allow(clippy::too_many_arguments)]
    fn unify_helper(
        handler: &Handler,
        engines: &Engines,
        received: TypeId,
        expected: TypeId,
        span: &Span,
        help_text: &str,
        err_override: Option<CompileError>,
        unify_kind: UnifyKind,
        push_unification: bool,
    ) {
        if !UnifyCheck::coercion(engines).check(received, expected) {
            // create a "mismatched type" error unless the `err_override`
            // argument has been provided
            match err_override {
                Some(err_override) => {
                    handler.emit_err(err_override);
                }
                None => {
                    handler.emit_err(CompileError::TypeError(TypeError::MismatchedType {
                        expected: engines.help_out(expected).to_string(),
                        received: engines.help_out(received).to_string(),
                        help_text: help_text.to_string(),
                        span: span.clone(),
                    }));
                }
            }
            return;
        }

        let h = Handler::default();
        let unifier = Unifier::new(engines, help_text, unify_kind);
        unifier.unify(handler, received, expected, span, push_unification);

        match err_override {
            Some(err_override) if h.has_errors() => {
                handler.emit_err(err_override);
            }
            _ => {
                handler.append(h);
            }
        }
    }

    pub(crate) fn push_unification(&self, unification: Unification) {
        self.unifications.insert(unification);
    }

    pub(crate) fn clear_unifications(&self) {
        self.unifications.clear();
    }

    pub(crate) fn reapply_unifications(&self, engines: &Engines, depth: usize) {
        if depth > 2000 {
            panic!("Possible infinite recursion");
        }

        let current_last_replace = *self.last_replace.read();
        for unification in self.unifications.values() {
            // eprintln!(
            //     "{depth}: {:?} -> {:?}",
            //     engines.help_out(unification.received),
            //     engines.help_out(unification.expected)
            // );
            Self::unify_helper(
                &Handler::default(),
                engines,
                unification.received,
                unification.expected,
                &unification.span,
                &unification.help_text,
                None,
                unification.unify_kind.clone(),
                false,
            )
        }
        if *self.last_replace.read() > current_last_replace {
            self.reapply_unifications(engines, depth + 1);
        }
    }

    pub(crate) fn to_typeinfo(&self, id: TypeId, error_span: &Span) -> Result<TypeInfo, TypeError> {
        match &*self.get(id) {
            TypeInfo::Unknown => Err(TypeError::UnknownType {
                span: error_span.clone(),
            }),
            ty => Ok(ty.clone()),
        }
    }

    /// Return whether a given type still contains undecayed references to [TypeInfo::Numeric]
    pub(crate) fn contains_numeric(&self, engines: &Engines, type_id: TypeId) -> bool {
        let decl_engine = engines.de();
        match &&*self.get(type_id) {
            TypeInfo::UntypedEnum(decl_id) => {
                engines
                    .pe()
                    .get_enum(decl_id)
                    .variants
                    .iter()
                    .all(|variant_type| {
                        self.contains_numeric(engines, variant_type.type_argument.type_id())
                    })
            }
            TypeInfo::UntypedStruct(decl_id) => engines
                .pe()
                .get_struct(decl_id)
                .fields
                .iter()
                .any(|field| self.contains_numeric(engines, field.type_argument.type_id())),
            TypeInfo::Enum(decl_ref) => {
                decl_engine
                    .get_enum(decl_ref)
                    .variants
                    .iter()
                    .all(|variant_type| {
                        self.contains_numeric(engines, variant_type.type_argument.type_id())
                    })
            }
            TypeInfo::Struct(decl_ref) => decl_engine
                .get_struct(decl_ref)
                .fields
                .iter()
                .any(|field| self.contains_numeric(engines, field.type_argument.type_id())),
            TypeInfo::Tuple(fields) => fields
                .iter()
                .any(|field_type| self.contains_numeric(engines, field_type.type_id())),
            TypeInfo::Array(elem_ty, _length) => self.contains_numeric(engines, elem_ty.type_id()),
            TypeInfo::Ptr(targ) => self.contains_numeric(engines, targ.type_id()),
            TypeInfo::Slice(targ) => self.contains_numeric(engines, targ.type_id()),
            TypeInfo::Ref {
                referenced_type, ..
            } => self.contains_numeric(engines, referenced_type.type_id()),
            TypeInfo::Unknown
            | TypeInfo::Never
            | TypeInfo::UnknownGeneric { .. }
            | TypeInfo::Placeholder(..)
            | TypeInfo::TypeParam(..)
            | TypeInfo::StringArray(..)
            | TypeInfo::StringSlice
            | TypeInfo::UnsignedInteger(..)
            | TypeInfo::Boolean
            | TypeInfo::ContractCaller { .. }
            | TypeInfo::Custom { .. }
            | TypeInfo::B256
            | TypeInfo::Contract
            | TypeInfo::ErrorRecovery(..)
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice
            | TypeInfo::Alias { .. }
            | TypeInfo::TraitType { .. } => false,
            TypeInfo::Numeric => true,
        }
    }

    /// Resolve all inner types that still are a [TypeInfo::Numeric] to a concrete `u64`
    pub(crate) fn decay_numeric(
        &self,
        handler: &Handler,
        engines: &Engines,
        type_id: TypeId,
        span: &Span,
    ) -> Result<(), ErrorEmitted> {
        let decl_engine = engines.de();

        match &&*self.get(type_id) {
            TypeInfo::UntypedEnum(decl_id) => {
                for variant_type in &engines.pe().get_enum(decl_id).variants {
                    self.decay_numeric(
                        handler,
                        engines,
                        variant_type.type_argument.type_id(),
                        span,
                    )?;
                }
            }
            TypeInfo::UntypedStruct(decl_id) => {
                for field in &engines.pe().get_struct(decl_id).fields {
                    self.decay_numeric(handler, engines, field.type_argument.type_id(), span)?;
                }
            }
            TypeInfo::Enum(decl_ref) => {
                for variant_type in &decl_engine.get_enum(decl_ref).variants {
                    self.decay_numeric(
                        handler,
                        engines,
                        variant_type.type_argument.type_id(),
                        span,
                    )?;
                }
            }
            TypeInfo::Struct(decl_ref) => {
                for field in &decl_engine.get_struct(decl_ref).fields {
                    self.decay_numeric(handler, engines, field.type_argument.type_id(), span)?;
                }
            }
            TypeInfo::Tuple(fields) => {
                for field_type in fields {
                    self.decay_numeric(handler, engines, field_type.type_id(), span)?;
                }
            }
            TypeInfo::Array(elem_ty, _length) => {
                self.decay_numeric(handler, engines, elem_ty.type_id(), span)?;
            }
            TypeInfo::Ptr(targ) => self.decay_numeric(handler, engines, targ.type_id(), span)?,
            TypeInfo::Slice(targ) => self.decay_numeric(handler, engines, targ.type_id(), span)?,
            TypeInfo::Ref {
                referenced_type, ..
            } => self.decay_numeric(handler, engines, referenced_type.type_id(), span)?,
            TypeInfo::Unknown
            | TypeInfo::Never
            | TypeInfo::UnknownGeneric { .. }
            | TypeInfo::Placeholder(..)
            | TypeInfo::TypeParam(..)
            | TypeInfo::StringSlice
            | TypeInfo::StringArray(..)
            | TypeInfo::UnsignedInteger(..)
            | TypeInfo::Boolean
            | TypeInfo::ContractCaller { .. }
            | TypeInfo::Custom { .. }
            | TypeInfo::B256
            | TypeInfo::Contract
            | TypeInfo::ErrorRecovery(..)
            | TypeInfo::RawUntypedPtr
            | TypeInfo::RawUntypedSlice
            | TypeInfo::Alias { .. }
            | TypeInfo::TraitType { .. } => {}
            TypeInfo::Numeric => {
                self.unify(handler, engines, type_id, self.id_of_u64(), span, "", None);
            }
        }
        Ok(())
    }

    /// Pretty print method for printing the [TypeEngine]. This method is
    /// manually implemented to avoid implementation overhead regarding using
    /// [DisplayWithEngines].
    pub fn pretty_print(&self, _decl_engine: &DeclEngine, engines: &Engines) -> String {
        let mut builder = String::new();
        let mut list = vec![];
        for tsi in self.slab.values() {
            list.push(format!("{:?}", engines.help_out(&*tsi.type_info)));
        }
        let list = ListDisplay { list };
        write!(builder, "TypeEngine {{\n{list}\n}}").unwrap();
        builder
    }
}

//! An analysis to compute symbols that escape out from a function.
//! This could be into another function, or via `ptr_to_int` etc.
//! Any transformations involving such symbols are unsafe.

use indexmap::IndexSet;
use rustc_hash::FxHashSet;
use sway_types::{FxIndexMap, FxIndexSet};

use crate::{
    AnalysisResult, AnalysisResultT, AnalysisResults, BlockArgument, Context, FuelVmInstruction,
    Function, InitAggr, InstOp, Instruction, IrError, LocalVar, Pass, PassMutability, ScopedPass,
    Type, Value, ValueDatum,
};

pub const ESCAPED_SYMBOLS_NAME: &str = "escaped-symbols";

/// A leaf, non-aggregate, element of a type's GEP shape (see [gep_shape]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LayoutLeaf {
    /// A single byte: `bool` or `u8`.
    Byte,
    /// An 8-byte word: `u16`/`u32`/`u64` or a (typed or untyped) pointer.
    ///
    /// A pointer and an integer word are deliberately folded into the same leaf kind.
    /// We intentionally do not distinguish the two here; doing
    /// so would only reject genuinely interchangeable reinterpretations without
    /// buying any additional safety.
    ///
    /// This is part of what lets a `slice` be reinterpreted as `{ ptr, u64 }`. A side
    /// effect is that e.g., `{ u64, u64 }` and `slice` are also GEP-equivalent.
    /// That is still true and safe for every consumer, which only relies on offsets
    /// and sizes lining up, never on a slot being "a pointer" vs. "an integer".
    Word,
    /// A 32-byte scalar: `b256` or `u256`.
    Wide,
    /// A string array of the given byte length.
    StringArray(u64),
}

/// The GEP-addressable shape of a type: either a non-aggregate leaf, or
/// an indexable aggregate broken into its ordered children, each tagged with its
/// byte offset relative to the aggregate's base.
///
/// It preserves the tree structure, so that **a `get_elem_ptr` index
/// chain valid in one type designates the same element in an equivalent type**.
///
/// This is what [types_are_gep_equivalent] compares.
enum GepShape {
    Leaf(LayoutLeaf),
    /// Ordered `(offset, child_type)` pairs of an indexable aggregate.
    Aggregate(Vec<(u64, Type)>),
}

/// Returns `ty`'s [GepShape], or `None` for a [Type] whose layout we deliberately
/// refuse to reason about (currently unions and unused integer widths).
///
/// The offset arithmetic mirrors [Type::get_indexed_offset] and [Type::size]:
/// struct fields are word-aligned, array elements are tightly packed, and a
/// `slice` (fat pointer) is modelled as `{ ptr, u64 }`.
fn gep_shape(context: &Context, ty: Type) -> Option<GepShape> {
    // TODO-MEMLAY: Warning! Here we make an assumption about the memory layout of structs and arrays.
    //              The memory layout of structs and arrays can be changed in the future.
    use crate::TypeContent::*;
    Some(match ty.get_content(context) {
        // Zero-sized types have no indexable children.
        Never | Unit => GepShape::Aggregate(vec![]),
        Bool | Uint(8) => GepShape::Leaf(LayoutLeaf::Byte),
        Uint(16) | Uint(32) | Uint(64) | Pointer | TypedPointer(_) => {
            GepShape::Leaf(LayoutLeaf::Word)
        }
        Uint(256) | B256 => GepShape::Leaf(LayoutLeaf::Wide),
        // Any other integer width is unexpected.
        Uint(_) => return None,
        StringArray(n) => GepShape::Leaf(LayoutLeaf::StringArray(*n)),
        // A slice / string slice is a fat pointer, GEP-equivalent to `{ ptr, u64 }`.
        Slice | TypedSlice(_) | StringSlice => {
            GepShape::Aggregate(vec![(0, Type::get_ptr(context)), (8, Type::get_uint64(context))])
        }
        Array(elem_ty, count) => {
            // Array elements are tightly packed.
            let elem_size = elem_ty.size(context).in_bytes();
            GepShape::Aggregate((0..*count).map(|i| (i * elem_size, *elem_ty)).collect())
        }
        Struct(fields) => {
            // Struct fields are aligned to word boundaries.
            let mut offset = 0;
            let mut children = Vec::with_capacity(fields.len());
            for field_ty in fields {
                children.push((offset, *field_ty));
                offset += field_ty.size(context).in_bytes_aligned();
            }
            GepShape::Aggregate(children)
        }
        // We deliberately do not support unions.
        Union(_) => return None,
    })
}

/// Returns `true` if `a` and `b` are GEP-equivalent: the same chain of
/// `get_elem_ptr` indices is valid in both and lands on the same byte offset on
/// a GEP-equivalent element. Equivalently, they have identical GEP trees down to
/// [LayoutLeaf]s.
///
/// E.g.: `[u64; 4]` and `[[u64; 2]; 2]` flatten to the same four [LayoutLeaf::Word]s
/// but are not GEP-equivalent, because, e.g, the index `[1]` selects a `u64` in the
/// first and a `[u64; 2]` sub-array in the second. In contrast `[[u64; 2]; 2]` and
/// `{ { u64, u64 }, { u64, u64 } }` are GEP-equivalent.
///
/// Its primary purpose is to recognize that a `slice` and a `{ ptr, u64 }` struct
/// are interchangeable, so that memory analyses can safely see through a
/// layout-preserving `cast_ptr` between them without ever attributing an index in
/// one type's basis to an incompatible element of the other.
///
/// The predicate is intentionally conservative: whenever it cannot prove `a` and `b`
/// are GEP-equivalent (e.g., for unions) it returns `false`.
pub fn types_are_gep_equivalent(context: &Context, a: Type, b: Type) -> bool {
    // Fast path for the common (and cheap) case of the *same* interned type.
    //
    // We deliberately use identity equality (`==` on the interned key) here and
    // not `Type::eq`: `Type::eq` considers a union equal to any of its
    // variants (a type-compatibility notion), which is not what we want here.
    if a == b {
        return true;
    }

    // Differently-sized types can never be GEP-equivalent.
    if a.size(context).in_bytes() != b.size(context).in_bytes() {
        return false;
    }

    match (gep_shape(context, a), gep_shape(context, b)) {
        (Some(GepShape::Leaf(la)), Some(GepShape::Leaf(lb))) => la == lb,
        (Some(GepShape::Aggregate(ca)), Some(GepShape::Aggregate(cb))) => {
            ca.len() == cb.len()
                && ca
                    .iter()
                    .zip(cb.iter())
                    .all(|(&(off_a, ty_a), &(off_b, ty_b))| {
                        off_a == off_b && types_are_gep_equivalent(context, ty_a, ty_b)
                    })
        }
        // A leaf vs. an aggregate (e.g. `b256` vs. `[u64; 4]`), or an opaque type
        // (a union or an unexpected integer width) are not GEP-equivalent.
        _ => false,
    }
}

/// Returns `true` if a `cast_ptr` from a value of type `from_ptr_ty` to type
/// `to_ptr_ty` is layout-preserving: both are pointers and their pointee
/// types are GEP-equivalent (see [types_are_gep_equivalent]).
///
/// When a `cast_ptr` is layout-preserving, any GEP/load/store/memcpy access
/// performed through the cast pointer touches exactly the same bytes, and via
/// the same index chain, the same element that it would through the original
/// pointer, so memory analyses can safely track symbols straight through the
/// cast. For any other cast the function returns `false` and the `cast_ptr` acts
/// as an opaque barrier to memory analysis.
pub fn cast_ptr_preserves_layout(context: &Context, from_ptr_ty: Type, to_ptr_ty: Type) -> bool {
    match (
        from_ptr_ty.get_pointee_type(context),
        to_ptr_ty.get_pointee_type(context),
    ) {
        (Some(from), Some(to)) => types_are_gep_equivalent(context, from, to),
        _ => false,
    }
}

/// Returns `true` if the `ptr_to_cast` is a pointer whose cast to `to_ty` pointer
/// is layout-preserving (see [cast_ptr_preserves_layout]).
fn is_layout_preserving_cast_ptr(context: &Context, ptr_to_cast: Value, to_ty: Type) -> bool {
    match ptr_to_cast.get_type(context) {
        Some(from_ty) => cast_ptr_preserves_layout(context, from_ty, to_ty),
        None => false,
    }
}

pub fn create_escaped_symbols_pass() -> Pass {
    Pass {
        name: ESCAPED_SYMBOLS_NAME,
        descr: "Symbols that escape or cannot be analyzed",
        deps: vec![],
        runner: ScopedPass::FunctionPass(PassMutability::Analysis(compute_escaped_symbols_pass)),
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum Symbol {
    Local(LocalVar),
    Arg(BlockArgument),
}

impl Symbol {
    pub fn get_type(&self, context: &Context) -> Type {
        match self {
            Symbol::Local(l) => l.get_type(context),
            Symbol::Arg(ba) => ba.ty,
        }
    }

    pub fn _get_name(&self, context: &Context, function: Function) -> String {
        match self {
            Symbol::Local(l) => function.lookup_local_name(context, l).unwrap().clone(),
            Symbol::Arg(ba) => format!("{}[{}]", ba.block.get_label(context), ba.idx),
        }
    }
}

/// Get [Symbol]s, both [Symbol::Local]s and [Symbol::Arg]s, reachable
/// from the `val` via chain of [InstOp::GetElemPtr] (GEP) instructions.
/// A `val` can, via GEP instructions, refer indirectly to none, or one
/// or more symbols.
///
/// If the `val` is not a pointer, an empty set is returned.
///
/// Note that this function does not return [Symbol]s potentially reachable
/// via referencing (`&`), dereferencing (`*`), and raw pointers (`__addr_of`)
/// and is thus suitable for all IR analysis and manipulation that deals
/// strictly with GEP access.
///
/// To acquire all [Symbol]s reachable from the `val`, use [get_referred_symbols] instead.
pub fn get_gep_referred_symbols(context: &Context, val: Value) -> FxIndexSet<Symbol> {
    match get_symbols(context, val, true) {
        ReferredSymbols::Complete(symbols) => symbols,
        _ => unreachable!(
            "In the case of GEP access, the set of returned symbols is always complete."
        ),
    }
}

/// Provides [Symbol]s, both [Symbol::Local]s and [Symbol::Arg]s, reachable
/// from a certain [Value] via chain of [InstOp::GetElemPtr] (GEP) instructions
/// or via [InstOp::IntToPtr] and [InstOp::PtrToInt] instruction patterns
/// specific to references, both referencing (`&`) and dereferencing (`*`),
/// and raw pointers, via `__addr_of`.
pub enum ReferredSymbols {
    /// Guarantees that all [Symbol]s reachable from the particular [Value]
    /// are collected, thus, that there are no escapes or pointer accesses
    /// in the scope that _might_ result in symbols indirectly related to
    /// the [Value] but not reachable only via GEP, or references, or
    /// raw pointers only.
    Complete(FxIndexSet<Symbol>),
    /// Denotes that there _might_ be [Symbol]s out of returned ones that
    /// are related to the particular [Value], but not reachable only via GEP,
    /// or references, or raw pointers.
    Incomplete(FxIndexSet<Symbol>),
}

impl ReferredSymbols {
    pub fn new(is_complete: bool, symbols: FxIndexSet<Symbol>) -> Self {
        if is_complete {
            Self::Complete(symbols)
        } else {
            Self::Incomplete(symbols)
        }
    }

    /// Returns the referred [Symbol]s and the information if they are
    /// complete (true) or incomplete (false).
    pub fn consume(self) -> (bool, FxIndexSet<Symbol>) {
        let is_complete = matches!(self, ReferredSymbols::Complete(_));
        let syms = match self {
            ReferredSymbols::Complete(syms) | ReferredSymbols::Incomplete(syms) => syms,
        };

        (is_complete, syms)
    }
}

/// Get [Symbol]s, both [Symbol::Local]s and [Symbol::Arg]s, reachable
/// from the `val` via chain of [InstOp::GetElemPtr] (GEP) instructions
/// or via [InstOp::IntToPtr] and [InstOp::PtrToInt] instruction patterns
/// specific to references, both referencing (`&`) and dereferencing (`*`),
/// and raw pointers, via `__addr_of`.
/// A `val` can, via these instructions, refer indirectly to none, or one
/// or more symbols.
///
/// Note that *this function does not perform any escape analysis*. E.g., if a
/// local symbol gets passed by `raw_ptr` or `&T` to a function and returned
/// back from the function via the same `raw_ptr` or `&T` the value returned
/// from the function will not be tracked back to the original symbol and the
/// symbol will not be collected as referred.
///
/// This means that, even if the result contains [Symbol]s, it _might_ be that
/// there are still other [Symbol]s in scope related to the `val`. E.g., in case
/// of branching, where the first branch directly returns `& local_var_a`
/// and the second branch, indirectly over a function call as explained above,
/// `& local_var_b`, only the `local_var_a` will be returned as a result.
///
/// Therefore, the function returns the [ReferredSymbols] enum to denote
/// if the returned set of symbols is guaranteed to be complete, or if it is
/// incomplete.
///
/// If the `val` is not a pointer, an empty set is returned and marked as
/// [ReferredSymbols::Complete].
pub fn get_referred_symbols(context: &Context, val: Value) -> ReferredSymbols {
    get_symbols(context, val, false)
}

/// Get [Symbol]s, both [Symbol::Local]s and [Symbol::Arg]s, reachable
/// from the `val`.
///
/// If `gep_only` is `true` only the [Symbol]s reachable via GEP instructions
/// are returned. Otherwise, the result also contains [Symbol]s reachable
/// via referencing (`&`) and dereferencing (`*`).
///
/// If the `val` is not a pointer, an empty set is returned and marked as
/// [ReferredSymbols::Complete].
fn get_symbols(context: &Context, val: Value, gep_only: bool) -> ReferredSymbols {
    // The input to this recursive function is always a pointer.
    // The function tracks backwards where the pointer is coming from.
    fn get_symbols_rec(
        context: &Context,
        symbols: &mut FxIndexSet<Symbol>,
        visited: &mut FxHashSet<Value>,
        ptr: Value,
        gep_only: bool,
        is_complete: &mut bool,
    ) {
        fn get_argument_symbols(
            context: &Context,
            symbols: &mut FxIndexSet<Symbol>,
            visited: &mut FxHashSet<Value>,
            arg: BlockArgument,
            gep_only: bool,
            is_complete: &mut bool,
        ) {
            if arg.block.get_label(context) == "entry" {
                symbols.insert(Symbol::Arg(arg));
            } else {
                arg.block
                    .pred_iter(context)
                    .map(|pred| arg.get_val_coming_from(context, pred).unwrap())
                    .for_each(|v| {
                        get_symbols_rec(context, symbols, visited, v, gep_only, is_complete)
                    })
            }
        }

        fn get_symbols_from_u64_address_argument(
            context: &Context,
            symbols: &mut FxIndexSet<Symbol>,
            visited: &mut FxHashSet<Value>,
            u64_address_arg: BlockArgument,
            is_complete: &mut bool,
        ) {
            if u64_address_arg.block.get_label(context) == "entry" {
                // The u64 address is coming from a function argument.
                // Same as in the case of a pointer coming from a function argument,
                // we collect it.
                symbols.insert(Symbol::Arg(u64_address_arg));
            } else {
                u64_address_arg
                    .block
                    .pred_iter(context)
                    .map(|pred| u64_address_arg.get_val_coming_from(context, pred).unwrap())
                    .for_each(|v| {
                        get_symbols_from_u64_address_rec(context, symbols, visited, v, is_complete)
                    })
            }
        }

        // The input to this recursive function is always a `u64` holding an address.
        // The below chain of instructions are specific to patterns where pointers
        // are obtained from `u64` addresses and vice versa. This includes:
        //  - referencing and dereferencing
        //  - raw pointers (`__addr_of`)
        //  - GTF intrinsic
        fn get_symbols_from_u64_address_rec(
            context: &Context,
            symbols: &mut FxIndexSet<Symbol>,
            visited: &mut FxHashSet<Value>,
            u64_address: Value,
            is_complete: &mut bool,
        ) {
            match context.values[u64_address.0].value {
                // Follow the sources of the address, and for every source address,
                // recursively come back to this function.
                ValueDatum::Argument(arg) => get_symbols_from_u64_address_argument(
                    context,
                    symbols,
                    visited,
                    arg,
                    is_complete,
                ),
                // 1. Patterns related to references and raw pointers.
                ValueDatum::Instruction(Instruction {
                    // The address is coming from a `raw_pointer` or `&T` variable.
                    op: InstOp::Load(_loaded_from),
                    ..
                }) => {
                    // TODO: https://github.com/FuelLabs/sway/issues/6065
                    //       We want to track sources of loaded addresses.
                    //       Currently we don't and simply mark the result as incomplete.
                    *is_complete = false;
                }
                ValueDatum::Instruction(Instruction {
                    op: InstOp::PtrToInt(ptr_value, _),
                    ..
                }) => get_symbols_rec(context, symbols, visited, ptr_value, false, is_complete),
                // 2. The address is coming from a GTF instruction.
                ValueDatum::Instruction(Instruction {
                    // There cannot be a symbol behind it, and so the returned set is complete.
                    op: InstOp::FuelVm(FuelVmInstruction::Gtf { .. }),
                    ..
                }) => (),
                // In other cases, e.g., getting the integer address from an unsafe pointer
                // arithmetic, or as a function result, etc. we bail out and mark the
                // collection as not being guaranteed to be a complete set of all referred symbols.
                _ => {
                    *is_complete = false;
                }
            }
        }

        if visited.contains(&ptr) {
            return;
        }
        visited.insert(ptr);
        match context.values[ptr.0].value {
            ValueDatum::Instruction(Instruction {
                op: InstOp::GetLocal(local),
                ..
            }) => {
                symbols.insert(Symbol::Local(local));
            }
            ValueDatum::Instruction(Instruction {
                op: InstOp::GetElemPtr { base, .. },
                ..
            }) => get_symbols_rec(context, symbols, visited, base, gep_only, is_complete),
            ValueDatum::Instruction(Instruction {
                op: InstOp::IntToPtr(u64_address, _),
                ..
            }) if !gep_only => get_symbols_from_u64_address_rec(
                context,
                symbols,
                visited,
                u64_address,
                is_complete,
            ),
            // We've reached a configurable at the top of the chain.
            // There cannot be a symbol behind it, and so the returned set is complete.
            ValueDatum::Instruction(Instruction {
                op: InstOp::GetConfig(_),
                ..
            }) if !gep_only => (),
            // We've reached a global at the top of the chain.
            // There cannot be a symbol behind it, and so the returned set is complete.
            ValueDatum::Instruction(Instruction {
                op: InstOp::GetGlobal(_),
                ..
            }) if !gep_only => (),
            // We've reached a storage key at the top of the chain.
            // There cannot be a symbol behind it, and so the returned set is complete.
            ValueDatum::Instruction(Instruction {
                op: InstOp::GetStorageKey(_),
                ..
            }) if !gep_only => (),
            // Note that in this case, the pointer itself is coming from a `Load`,
            // and not an address. So, we just continue following the pointer.
            ValueDatum::Instruction(Instruction {
                op: InstOp::Load(loaded_from),
                ..
            }) if !gep_only => get_symbols_rec(
                context,
                symbols,
                visited,
                loaded_from,
                gep_only,
                is_complete,
            ),
            ValueDatum::Instruction(Instruction {
                op: InstOp::CastPtr(ptr_to_cast, to_ty),
                ..
            }) if !gep_only || is_layout_preserving_cast_ptr(context, ptr_to_cast, to_ty) => {
                // For non-GEP tracking we always follow a `cast_ptr`. For GEP-only
                // tracking we may follow it too, but only when it is
                // layout-preserving: then a GEP through the cast addresses the same
                // bytes of the same symbol as a GEP through the original pointer.
                get_symbols_rec(
                    context,
                    symbols,
                    visited,
                    ptr_to_cast,
                    gep_only,
                    is_complete,
                )
            }
            ValueDatum::Argument(arg) => {
                get_argument_symbols(context, symbols, visited, arg, gep_only, is_complete)
            }
            // We've reached a constant at the top of the chain.
            // There cannot be a symbol behind it, and so the returned set is complete.
            ValueDatum::Constant(_) if !gep_only => (),
            _ if !gep_only => {
                // In other cases, e.g., getting the pointer from an ASM block,
                // or as a function result, etc., we cannot track the value up the chain
                // and cannot guarantee that the value is not coming from some of the symbols.
                // So, we bail out and mark the collection as not being guaranteed to be
                // a complete set of all referred symbols.
                *is_complete = false;
            }
            // In the case of GEP only access, the returned set is always complete.
            _ => (),
        }
    }

    if !val.get_type(context).is_some_and(|t| t.is_ptr(context)) {
        return ReferredSymbols::new(true, IndexSet::default());
    }

    let mut visited = FxHashSet::default();
    let mut symbols = IndexSet::default();
    let mut is_complete = true;

    get_symbols_rec(
        context,
        &mut symbols,
        &mut visited,
        val,
        gep_only,
        &mut is_complete,
    );

    ReferredSymbols::new(is_complete, symbols)
}

pub fn get_gep_symbol(context: &Context, val: Value) -> Option<Symbol> {
    let syms = get_gep_referred_symbols(context, val);
    (syms.len() == 1)
        .then(|| syms.iter().next().cloned())
        .flatten()
}

/// Return [Symbol] referred by `val` if there is _exactly one_ symbol referred,
/// or `None` if there are no [Symbol]s referred or if there is more than one
/// referred.
pub fn get_referred_symbol(context: &Context, val: Value) -> Option<Symbol> {
    let syms = get_referred_symbols(context, val);
    match syms {
        ReferredSymbols::Complete(syms) => (syms.len() == 1)
            .then(|| syms.iter().next().cloned())
            .flatten(),
        // It might be that we have more than one referred symbol here.
        ReferredSymbols::Incomplete(_) => None,
    }
}

pub enum EscapedSymbols {
    /// Guarantees that all escaping [Symbol]s are collected.
    Complete(FxHashSet<Symbol>),
    /// Denotes that there _might_ be additional escaping [Symbol]s
    /// out of the collected ones.
    Incomplete(FxHashSet<Symbol>),
}

impl AnalysisResultT for EscapedSymbols {}

pub fn compute_escaped_symbols_pass(
    context: &Context,
    _analyses: &AnalysisResults,
    function: Function,
) -> Result<AnalysisResult, IrError> {
    Ok(Box::new(compute_escaped_symbols(context, &function)))
}

fn compute_escaped_symbols(context: &Context, function: &Function) -> EscapedSymbols {
    let add_from_val = |result: &mut FxHashSet<Symbol>, val: &Value, is_complete: &mut bool| {
        let (complete, syms) = get_referred_symbols(context, *val).consume();

        *is_complete &= complete;

        syms.iter().for_each(|s| {
            result.insert(*s);
        });
    };

    let mut result = FxHashSet::default();
    let mut is_complete = true;

    for (_block, inst) in function.instruction_iter(context) {
        match &inst.get_instruction(context).unwrap().op {
            InstOp::AsmBlock(_, args) => {
                for arg_init in args.iter().filter_map(|arg| arg.initializer) {
                    add_from_val(&mut result, &arg_init, &mut is_complete)
                }
            }
            InstOp::UnaryOp { .. } => (),
            InstOp::BinaryOp { .. } => (),
            InstOp::BitCast(_, _) => (),
            InstOp::Branch(_) => (),
            InstOp::Call(callee, args) => args
                .iter()
                .enumerate()
                .filter(|(arg_idx, _arg)| {
                    // Immutable arguments are not considered as escaping symbols.
                    !callee.is_arg_immutable(context, *arg_idx)
                })
                .for_each(|(_, v)| add_from_val(&mut result, v, &mut is_complete)),
            InstOp::CastPtr(ptr, to_ty) => {
                // A layout-preserving `cast_ptr` (e.g. `slice` <-> `{ ptr, u64 }`)
                // merely reinterprets the pointer; it does not, by itself, let the
                // pointee escape. The symbols behind `ptr` remain fully trackable
                // through the cast (see `get_symbols`), and any genuinely escaping
                // later use of the cast result is still caught when that use is
                // visited. For any other cast we can no longer reason about the
                // accesses performed through it, so we conservatively treat the
                // pointee as escaped.
                if !is_layout_preserving_cast_ptr(context, *ptr, *to_ty) {
                    add_from_val(&mut result, ptr, &mut is_complete)
                }
            }
            InstOp::Cmp(_, _, _) => (),
            InstOp::ConditionalBranch { .. } => (),
            InstOp::ContractCall { params, .. } => {
                add_from_val(&mut result, params, &mut is_complete)
            }
            InstOp::FuelVm(_) => (),
            InstOp::GetLocal(_) => (),
            InstOp::GetGlobal(_) => (),
            InstOp::GetConfig(_) => (),
            InstOp::GetStorageKey(_) => (),
            InstOp::GetElemPtr { .. } => (),
            InstOp::IntToPtr(_, _) => (),
            InstOp::Load(_) => (),
            InstOp::MemCopyBytes { .. } => (),
            InstOp::MemCopyVal { .. } => (),
            InstOp::MemClearVal { .. } => (),
            InstOp::Nop => (),
            InstOp::PtrToInt(v, _) => add_from_val(&mut result, v, &mut is_complete),
            InstOp::Ret(_, _) => (),
            InstOp::Store { stored_val, .. } => {
                add_from_val(&mut result, stored_val, &mut is_complete)
            }
            InstOp::Alloc { .. } => (),
            InstOp::InitAggr(init_aggr) => {
                // Conceptually, we can think of `InitAggr` as a series of stores into the aggregate.
                // If any of the initializer values refer to symbols, those symbols escape.
                // This happens when the aggregate contains pointers, e.g., a Sway struct with fields
                // that are references.
                for init in init_aggr.initializers.iter() {
                    add_from_val(&mut result, init, &mut is_complete);
                }
            }
        }
    }

    if is_complete {
        EscapedSymbols::Complete(result)
    } else {
        EscapedSymbols::Incomplete(result)
    }
}

/// Pointers that may possibly be loaded from the instruction `inst`.
pub fn get_loaded_ptr_values(context: &Context, inst: Value) -> Vec<Value> {
    match &inst.get_instruction(context).unwrap().op {
        InstOp::UnaryOp { .. }
        | InstOp::BinaryOp { .. }
        | InstOp::BitCast(_, _)
        | InstOp::Branch(_)
        | InstOp::ConditionalBranch { .. }
        | InstOp::Cmp(_, _, _)
        | InstOp::Nop
        | InstOp::CastPtr(_, _)
        | InstOp::GetLocal(_)
        | InstOp::GetGlobal(_)
        | InstOp::GetConfig(_)
        | InstOp::GetStorageKey(_)
        | InstOp::GetElemPtr { .. }
        | InstOp::IntToPtr(_, _) => vec![],
        InstOp::PtrToInt(src_val_ptr, _) => vec![*src_val_ptr],
        InstOp::ContractCall {
            params,
            coins,
            asset_id,
            ..
        } => vec![*params, *coins, *asset_id],
        InstOp::Call(_, args) => args.clone(),
        InstOp::AsmBlock(_, args) => args.iter().filter_map(|val| val.initializer).collect(),
        InstOp::MemClearVal { .. } => vec![],
        InstOp::MemCopyBytes { src_val_ptr, .. }
        | InstOp::MemCopyVal { src_val_ptr, .. }
        | InstOp::Ret(src_val_ptr, _)
        | InstOp::Load(src_val_ptr)
        | InstOp::FuelVm(FuelVmInstruction::Log {
            log_val: src_val_ptr,
            ..
        })
        | InstOp::FuelVm(FuelVmInstruction::StateLoadWord {
            key: src_val_ptr, ..
        })
        | InstOp::FuelVm(FuelVmInstruction::StateStoreWord {
            key: src_val_ptr, ..
        })
        | InstOp::FuelVm(FuelVmInstruction::StateLoadQuadWord {
            key: src_val_ptr, ..
        })
        | InstOp::FuelVm(FuelVmInstruction::StateReadSlot {
            key: src_val_ptr, ..
        })
        | InstOp::FuelVm(FuelVmInstruction::StateClear {
            key: src_val_ptr, ..
        })
        | InstOp::FuelVm(FuelVmInstruction::StateClearSlots {
            key: src_val_ptr, ..
        })
        | InstOp::FuelVm(FuelVmInstruction::StatePreload {
            key: src_val_ptr, ..
        }) => vec![*src_val_ptr],
        InstOp::FuelVm(FuelVmInstruction::StateStoreQuadWord {
            stored_val: memopd1,
            key: memopd2,
            ..
        })
        | InstOp::FuelVm(FuelVmInstruction::StateWriteSlot {
            stored_val: memopd1,
            key: memopd2,
            ..
        })
        | InstOp::FuelVm(FuelVmInstruction::StateUpdateSlot {
            stored_val: memopd1,
            key: memopd2,
            ..
        })
        | InstOp::FuelVm(FuelVmInstruction::Smo {
            recipient: memopd1,
            message: memopd2,
            ..
        }) => vec![*memopd1, *memopd2],
        InstOp::Store { dst_val_ptr: _, .. } => vec![],
        InstOp::Alloc { .. } => vec![],
        InstOp::FuelVm(FuelVmInstruction::Gtf { .. })
        | InstOp::FuelVm(FuelVmInstruction::ReadRegister(_))
        | InstOp::FuelVm(FuelVmInstruction::Revert(_) | FuelVmInstruction::JmpMem) => vec![],
        InstOp::FuelVm(FuelVmInstruction::WideUnaryOp { arg, .. }) => vec![*arg],
        InstOp::FuelVm(FuelVmInstruction::WideBinaryOp { arg1, arg2, .. })
        | InstOp::FuelVm(FuelVmInstruction::WideCmpOp { arg1, arg2, .. }) => {
            vec![*arg1, *arg2]
        }
        InstOp::FuelVm(FuelVmInstruction::WideModularOp {
            arg1, arg2, arg3, ..
        }) => vec![*arg1, *arg2, *arg3],
        InstOp::FuelVm(FuelVmInstruction::Retd { ptr, .. }) => vec![*ptr],
        // Similar like `Store` that never loads a pointer that might be its value,
        // `InitAggr` does not load any pointers it might have in the initializers.
        InstOp::InitAggr(_) => vec![],
    }
}

/// [Symbol]s that may possibly, directly or indirectly, be loaded from the instruction `inst`.
pub fn get_loaded_symbols(context: &Context, inst: Value) -> ReferredSymbols {
    let mut res = IndexSet::default();
    let mut is_complete = true;
    for val in get_loaded_ptr_values(context, inst) {
        let (complete, syms) = get_referred_symbols(context, val).consume();

        is_complete &= complete;

        for sym in syms {
            res.insert(sym);
        }
    }

    ReferredSymbols::new(is_complete, res)
}

/// Pointers that may possibly be stored to the instruction `inst`.
pub fn get_stored_ptr_values(context: &Context, inst: Value) -> Vec<Value> {
    match &inst.get_instruction(context).unwrap().op {
        InstOp::UnaryOp { .. }
        | InstOp::BinaryOp { .. }
        | InstOp::BitCast(_, _)
        | InstOp::Branch(_)
        | InstOp::ConditionalBranch { .. }
        | InstOp::Cmp(_, _, _)
        | InstOp::Nop
        | InstOp::PtrToInt(_, _)
        | InstOp::Ret(_, _)
        | InstOp::CastPtr(_, _)
        | InstOp::GetLocal(_)
        | InstOp::GetGlobal(_)
        | InstOp::GetConfig(_)
        | InstOp::GetStorageKey(_)
        | InstOp::GetElemPtr { .. }
        | InstOp::IntToPtr(_, _) => vec![],
        InstOp::ContractCall { params, .. } => vec![*params],
        InstOp::Call(_, args) => args.clone(),
        InstOp::AsmBlock(_, args) => args.iter().filter_map(|val| val.initializer).collect(),
        InstOp::MemCopyBytes { dst_val_ptr, .. }
        | InstOp::MemCopyVal { dst_val_ptr, .. }
        | InstOp::MemClearVal { dst_val_ptr }
        | InstOp::Store { dst_val_ptr, .. } => vec![*dst_val_ptr],
        InstOp::InitAggr(InitAggr {
            aggr_ptr,
            initializers: _,
        }) => vec![*aggr_ptr],
        InstOp::Load(_) => vec![],
        InstOp::Alloc { .. } => vec![],
        InstOp::FuelVm(vmop) => match vmop {
            FuelVmInstruction::Gtf { .. }
            | FuelVmInstruction::Log { .. }
            | FuelVmInstruction::ReadRegister(_)
            | FuelVmInstruction::Revert(_)
            | FuelVmInstruction::JmpMem
            | FuelVmInstruction::Smo { .. }
            | FuelVmInstruction::Retd { .. }
            | FuelVmInstruction::StateClear { .. }
            | FuelVmInstruction::StateClearSlots { .. }
            | FuelVmInstruction::StateLoadWord { .. }
            | FuelVmInstruction::StateStoreWord { .. }
            | FuelVmInstruction::StateStoreQuadWord { .. }
            | FuelVmInstruction::StateWriteSlot { .. }
            | FuelVmInstruction::StateUpdateSlot { .. }
            | FuelVmInstruction::StatePreload { .. } => vec![],
            FuelVmInstruction::StateLoadQuadWord { load_val, .. }
            | FuelVmInstruction::StateReadSlot { load_val, .. } => vec![*load_val],
            FuelVmInstruction::WideUnaryOp { result, .. }
            | FuelVmInstruction::WideBinaryOp { result, .. }
            | FuelVmInstruction::WideModularOp { result, .. } => vec![*result],
            FuelVmInstruction::WideCmpOp { .. } => vec![],
        },
    }
}

/// [Symbol]s that may possibly, directly or indirectly, be stored to the instruction `inst`.
pub fn get_stored_symbols(context: &Context, inst: Value) -> ReferredSymbols {
    let mut res = IndexSet::default();
    let mut is_complete = true;
    for val in get_stored_ptr_values(context, inst) {
        let (complete, syms) = get_referred_symbols(context, val).consume();

        is_complete &= complete;

        for sym in syms {
            res.insert(sym);
        }
    }

    ReferredSymbols::new(is_complete, res)
}

/// Combine a series of GEPs into one.
pub fn combine_indices(context: &Context, val: Value) -> Option<Vec<Value>> {
    match &context.values[val.0].value {
        // Locals are base symbols, always accessed at offset zero, so they
        // don't contribute additional indices.
        ValueDatum::Instruction(Instruction {
            op: InstOp::GetLocal(_),
            ..
        }) => Some(vec![]),
        ValueDatum::Instruction(Instruction {
            op:
                InstOp::GetElemPtr {
                    base,
                    elem_ptr_ty: _,
                    indices,
                },
            ..
        }) => {
            let mut base_indices = combine_indices(context, *base)?;
            base_indices.append(&mut indices.clone());
            Some(base_indices)
        }
        ValueDatum::Argument(arg) => {
            if arg.block.get_label(context) == "entry" {
                // Entry block arguments are function parameters, and same as locals base
                // symbols, accessed at offset zero, and contribute no indices.
                Some(vec![])
            } else {
                // All other block arguments are control-flow joins: their values arrive
                // along multiple predecessor edges and may be pointers at arbitrary
                // offsets into a symbol, depending on a predecessor.
                // There is no single GEP index path to combine in this case,
                // so the offset is unknown.
                None
            }
        }
        _ => None,
    }
}

/// Given a memory pointer instruction, compute the offset of indexed element,
/// for each symbol that it may alias to.
/// If for any symbol we can't compute it, return None.
pub fn get_memory_offsets(context: &Context, val: Value) -> Option<FxIndexMap<Symbol, u64>> {
    let syms = get_gep_referred_symbols(context, val);

    let mut res: FxIndexMap<Symbol, u64> = FxIndexMap::default();
    for sym in syms {
        let offset = sym
            .get_type(context)
            .get_pointee_type(context)?
            .get_value_indexed_offset(context, &combine_indices(context, val)?)?;
        res.insert(sym, offset);
    }
    Some(res)
}

/// Can memory ranges [val1, val1+len1] and [val2, val2+len2] overlap?
/// Conservatively returns true if cannot statically determine.
pub fn may_alias(context: &Context, val1: Value, len1: u64, val2: Value, len2: u64) -> bool {
    let (Some(mem_offsets_1), Some(mem_offsets_2)) = (
        get_memory_offsets(context, val1),
        get_memory_offsets(context, val2),
    ) else {
        return true;
    };

    for (sym1, off1) in mem_offsets_1 {
        if let Some(off2) = mem_offsets_2.get(&sym1) {
            // does off1 + len1 overlap with off2 + len2?
            if (off1 <= *off2 && (off1 + len1 > *off2)) || (*off2 <= off1 && (*off2 + len2 > off1))
            {
                return true;
            }
        }
    }
    false
}

/// Are memory ranges [val1, val1+len1] and [val2, val2+len2] exactly the same?
/// Conservatively returns false if cannot statically determine.
pub fn must_alias(context: &Context, val1: Value, len1: u64, val2: Value, len2: u64) -> bool {
    let (Some(mem_offsets_1), Some(mem_offsets_2)) = (
        get_memory_offsets(context, val1),
        get_memory_offsets(context, val2),
    ) else {
        return false;
    };

    if mem_offsets_1.len() != 1 || mem_offsets_2.len() != 1 {
        return false;
    }

    let (sym1, off1) = mem_offsets_1.iter().next().unwrap();
    let (sym2, off2) = mem_offsets_2.iter().next().unwrap();

    // does off1 + len1 overlap with off2 + len2?
    sym1 == sym2 && off1 == off2 && len1 == len2
}

/// For a pointer argument `ptr_val`, what's the size of its pointee.
pub fn pointee_size(context: &Context, ptr_val: Value) -> u64 {
    ptr_val
        .get_type(context)
        .unwrap()
        .get_pointee_type(context)
        .expect("Expected arg to be a pointer")
        .size(context)
        .in_bytes()
}

#[cfg(test)]
mod tests {
    use once_cell::sync::Lazy;
    use sway_features::ExperimentalFeatures;
    use sway_types::SourceEngine;

    use super::{cast_ptr_preserves_layout, types_are_gep_equivalent};
    use crate::{Backtrace, Context, Type};

    static SOURCE_ENGINE: Lazy<SourceEngine> = Lazy::new(SourceEngine::default);

    fn create_context() -> Context<'static> {
        Context::new(
            &SOURCE_ENGINE,
            ExperimentalFeatures::default(),
            Backtrace::default(),
        )
    }

    #[test]
    /// A `slice` is a fat pointer laid out as `{ ptr, u64 }`, so the two are
    /// layout-compatible even though they are different types.
    fn slice_and_ptr_u64_struct_have_same_layout() {
        let mut context = create_context();

        let ptr = Type::get_ptr(&context);
        let u64_ty = Type::get_uint64(&context);
        let slice = Type::get_slice(&context);
        let ptr_u64 = Type::new_struct(&mut context, vec![ptr, u64_ty]);

        assert!(types_are_gep_equivalent(&context, slice, ptr_u64));
        assert!(types_are_gep_equivalent(&context, ptr_u64, slice));
        // Reflexivity.
        assert!(types_are_gep_equivalent(&context, slice, slice));
    }

    #[test]
    /// Two words are two words, regardless of whether they are spelled as a
    /// struct, a two-element array, or a slice.
    fn two_word_aggregates_have_same_layout() {
        let mut context = create_context();

        let u64_ty = Type::get_uint64(&context);
        let ptr = Type::get_ptr(&context);
        let struct_u64_u64 = Type::new_struct(&mut context, vec![u64_ty, u64_ty]);
        let array_u64_2 = Type::new_array(&mut context, u64_ty, 2);
        let struct_ptr_u64 = Type::new_struct(&mut context, vec![ptr, u64_ty]);

        assert!(types_are_gep_equivalent(
            &context,
            struct_u64_u64,
            array_u64_2
        ));
        assert!(types_are_gep_equivalent(
            &context,
            struct_u64_u64,
            struct_ptr_u64
        ));
    }

    #[test]
    /// A 32-byte scalar (`b256`) is not layout-compatible with a 32-byte
    /// aggregate of four words.
    fn scalar_and_aggregate_of_same_size_differ() {
        let mut context = create_context();

        let u64_ty = Type::get_uint64(&context);
        let b256 = Type::get_b256(&context);
        let array_u64_4 = Type::new_array(&mut context, u64_ty, 4);
        let struct_u64_4 = Type::new_struct(&mut context, vec![u64_ty, u64_ty, u64_ty, u64_ty]);

        assert_eq!(b256.size(&context).in_bytes(), 32);
        assert_eq!(array_u64_4.size(&context).in_bytes(), 32);

        assert!(!types_are_gep_equivalent(&context, b256, array_u64_4));
        assert!(!types_are_gep_equivalent(&context, b256, struct_u64_4));
    }

    #[test]
    /// Differently-sized types are never layout-compatible.
    fn different_sizes_differ() {
        let mut context = create_context();

        let ptr = Type::get_ptr(&context);
        let u64_ty = Type::get_uint64(&context);
        let slice = Type::get_slice(&context); // 16 bytes
        let struct_ptr_u64_u64 = Type::new_struct(&mut context, vec![ptr, u64_ty, u64_ty]); // 24 bytes

        assert!(!types_are_gep_equivalent(&context, slice, struct_ptr_u64_u64));
    }

    #[test]
    /// The word granularity must line up: a struct whose first field is a byte
    /// is not compatible with one whose first field is a word, even at equal
    /// total size (padding differs).
    fn byte_vs_word_layout_differs() {
        let mut context = create_context();

        let u8_ty = Type::get_uint8(&context);
        let u64_ty = Type::get_uint64(&context);
        // { u8, u64 }: u8 padded to a word, then u64 => [Byte@0, Word@8], 16 bytes.
        let struct_u8_u64 = Type::new_struct(&mut context, vec![u8_ty, u64_ty]);
        // { u64, u64 }: [Word@0, Word@8], 16 bytes.
        let struct_u64_u64 = Type::new_struct(&mut context, vec![u64_ty, u64_ty]);

        assert_eq!(struct_u8_u64.size(&context).in_bytes(), 16);
        assert_eq!(struct_u64_u64.size(&context).in_bytes(), 16);
        assert!(!types_are_gep_equivalent(
            &context,
            struct_u8_u64,
            struct_u64_u64
        ));
    }

    #[test]
    /// A union (and an enum, which is `{ tag, union }`) must never be considered
    /// layout-compatible with one of its variants, even though `Type::eq` treats
    /// them as equal.
    fn union_and_variant_differ() {
        let mut context = create_context();

        let u64_ty = Type::get_uint64(&context);
        let t1 = Type::new_struct(&mut context, vec![u64_ty]); // { u64 }
        let t2 = Type::new_struct(&mut context, vec![u64_ty, u64_ty]); // { u64, u64 }
        let variants = Type::new_union(&mut context, vec![u64_ty, t1, t2]);

        // The union is as large as its biggest variant (16 bytes), the small
        // variant is 8 bytes: clearly different layouts.
        assert!(!types_are_gep_equivalent(&context, variants, u64_ty));
        assert!(!types_are_gep_equivalent(&context, variants, t1));
        // Even against the largest variant, we conservatively refuse (we do
        // not reason about union layouts at all).
        assert!(!types_are_gep_equivalent(&context, variants, t2));

        // The exact enum-payload shape from the failing test: `{ u64, <union> }`
        // vs. `{ u64, u64 }`. `Type::eq` would call these equal; we must not.
        let enum_ty = Type::new_struct(&mut context, vec![u64_ty, variants]);
        assert!(enum_ty.eq(&context, &t2)); // sanity: `Type::eq` is permissive here
        assert!(!types_are_gep_equivalent(&context, enum_ty, t2));
    }

    #[test]
    /// `cast_ptr_preserves_layout` compares the pointee types of two pointer
    /// types.
    fn cast_ptr_predicate_compares_pointees() {
        let mut context = create_context();

        let ptr = Type::get_ptr(&context);
        let u64_ty = Type::get_uint64(&context);
        let slice = Type::get_slice(&context);
        let ptr_u64 = Type::new_struct(&mut context, vec![ptr, u64_ty]);
        let b256 = Type::get_b256(&context);
        let array_u64_4 = Type::new_array(&mut context, u64_ty, 4);

        let slice_ptr = Type::new_typed_pointer(&mut context, slice);
        let ptr_u64_ptr = Type::new_typed_pointer(&mut context, ptr_u64);
        let b256_ptr = Type::new_typed_pointer(&mut context, b256);
        let array_ptr = Type::new_typed_pointer(&mut context, array_u64_4);

        // Layout-preserving: `slice*` <-> `{ ptr, u64 }*`.
        assert!(cast_ptr_preserves_layout(&context, slice_ptr, ptr_u64_ptr));
        // Not layout-preserving: `b256*` <-> `[u64; 4]*`.
        assert!(!cast_ptr_preserves_layout(&context, b256_ptr, array_ptr));
    }

    // GEP-equivalence (structural) tests.
    //
    // Two pointee types are only interchangeable when they are GEP-equivalent:
    // the same chain of GEP indices must be valid in both and must land on the
    // same byte offset or same leaf element.

    #[test]
    /// `[u64; 4]` and `[[u64; 2]; 2]` flatten to the same four words, but the GEP
    /// index chain `[i]` lands on a `u64` in the first and on a `[u64; 2]` sub-array
    /// in the second. They are not GEP-equivalent.
    fn flat_and_nested_arrays_are_not_gep_equivalent() {
        let mut context = create_context();

        let u64_ty = Type::get_uint64(&context);
        let arr4 = Type::new_array(&mut context, u64_ty, 4); // [u64; 4]
        let inner = Type::new_array(&mut context, u64_ty, 2); // [u64; 2]
        let nested = Type::new_array(&mut context, inner, 2); // [[u64; 2]; 2]

        assert_eq!(arr4.size(&context).in_bytes(), nested.size(&context).in_bytes());
        assert!(!types_are_gep_equivalent(&context, arr4, nested));
    }

    #[test]
    /// `{ u64, u64, u64, u64 }` and `{ { u64, u64 }, { u64, u64 } }` share the same
    /// leaves but expose different GEP trees (four flat fields vs. two sub-structs).
    fn flat_and_nested_structs_are_not_gep_equivalent() {
        let mut context = create_context();

        let u64_ty = Type::get_uint64(&context);
        let flat = Type::new_struct(&mut context, vec![u64_ty, u64_ty, u64_ty, u64_ty]);
        let pair = Type::new_struct(&mut context, vec![u64_ty, u64_ty]);
        let nested = Type::new_struct(&mut context, vec![pair, pair]);

        assert!(!types_are_gep_equivalent(&context, flat, nested));
    }

    #[test]
    /// A flat `[u64; 4]` and a nested `{ { u64, u64 }, { u64, u64 } }`. The chain `[i]`
    /// selects a leaf word in the array and a sub-struct in the struct. Not equivalent.
    fn flat_array_and_nested_struct_are_not_gep_equivalent() {
        let mut context = create_context();

        let u64_ty = Type::get_uint64(&context);
        let arr4 = Type::new_array(&mut context, u64_ty, 4);
        let pair = Type::new_struct(&mut context, vec![u64_ty, u64_ty]);
        let nested = Type::new_struct(&mut context, vec![pair, pair]);

        assert!(!types_are_gep_equivalent(&context, arr4, nested));
    }

    #[test]
    /// `[[u64; 2]; 2]` and `{ { u64, u64 }, { u64, u64 } }` expose the same 2x2 GEP
    /// tree at the same offsets: `[i, j]` designates the same word in both. An array
    /// node and a struct node of the same shape are GEP-equivalent.
    fn nested_array_and_nested_struct_are_gep_equivalent() {
        let mut context = create_context();

        let u64_ty = Type::get_uint64(&context);
        let inner_arr = Type::new_array(&mut context, u64_ty, 2); // [u64; 2]
        let nested_arr = Type::new_array(&mut context, inner_arr, 2); // [[u64; 2]; 2]
        let pair = Type::new_struct(&mut context, vec![u64_ty, u64_ty]); // { u64, u64 }
        let nested_struct = Type::new_struct(&mut context, vec![pair, pair]);

        assert!(types_are_gep_equivalent(&context, nested_arr, nested_struct));
    }

    #[test]
    /// A flat `[u64; 4]` and a flat `{ u64, u64, u64, u64 }` both expose four
    /// word-sized children at 0/8/16/24, so `[i]` matches. GEP-equivalent.
    fn flat_array_and_flat_struct_are_gep_equivalent() {
        let mut context = create_context();

        let u64_ty = Type::get_uint64(&context);
        let arr4 = Type::new_array(&mut context, u64_ty, 4);
        let flat = Type::new_struct(&mut context, vec![u64_ty, u64_ty, u64_ty, u64_ty]);

        assert!(types_are_gep_equivalent(&context, arr4, flat));
    }
}

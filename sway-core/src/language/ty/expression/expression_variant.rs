use crate::{
    decl_engine::*,
    engine_threading::*,
    has_changes,
    language::{ty::*, *},
    semantic_analysis::{
        TyNodeDepGraphEdge, TyNodeDepGraphEdgeInfo, TypeCheckAnalysis, TypeCheckAnalysisContext,
        TypeCheckContext, TypeCheckFinalization, TypeCheckFinalizationContext,
    },
    type_system::*,
};
use ast_elements::type_parameter::GenericTypeParameter;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Write},
    hash::{Hash, Hasher},
};
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Ident, Named, Span, Spanned};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TyExpressionVariant {
    Literal(Literal),
    FunctionApplication {
        call_path: CallPath,
        arguments: Vec<(Ident, TyExpression)>,
        fn_ref: DeclRefFunction,
        selector: Option<ContractCallParams>,
        /// Optional binding information for the LSP.
        type_binding: Option<TypeBinding<()>>,
        /// In case of a method call, a [TypeId] of the method target (self).
        /// E.g., `method_target.some_method()`.
        method_target: Option<TypeId>,
        contract_call_params: IndexMap<String, TyExpression>,
        contract_caller: Option<Box<TyExpression>>,
    },
    LazyOperator {
        op: LazyOp,
        lhs: Box<TyExpression>,
        rhs: Box<TyExpression>,
    },
    ConstantExpression {
        span: Span,
        decl: Box<TyConstantDecl>,
        call_path: Option<CallPath>,
    },
    ConfigurableExpression {
        span: Span,
        decl: Box<TyConfigurableDecl>,
        call_path: Option<CallPath>,
    },
    ConstGenericExpression {
        span: Span,
        decl: Box<TyConstGenericDecl>,
        call_path: CallPath,
    },
    VariableExpression {
        name: Ident,
        span: Span,
        mutability: VariableMutability,
        call_path: Option<CallPath>,
    },
    Tuple {
        fields: Vec<TyExpression>,
    },
    ArrayExplicit {
        elem_type: TypeId,
        contents: Vec<TyExpression>,
    },
    ArrayRepeat {
        elem_type: TypeId,
        value: Box<TyExpression>,
        length: Box<TyExpression>,
    },
    ArrayIndex {
        prefix: Box<TyExpression>,
        index: Box<TyExpression>,
    },
    StructExpression {
        struct_id: DeclId<TyStructDecl>,
        fields: Vec<TyStructExpressionField>,
        instantiation_span: Span,
        call_path_binding: TypeBinding<CallPath>,
    },
    CodeBlock(TyCodeBlock),
    // a flag that this value will later be provided as a parameter, but is currently unknown
    FunctionParameter,
    MatchExp {
        desugared: Box<TyExpression>,
        scrutinees: Vec<TyScrutinee>,
    },
    IfExp {
        condition: Box<TyExpression>,
        then: Box<TyExpression>,
        r#else: Option<Box<TyExpression>>,
    },
    AsmExpression {
        registers: Vec<TyAsmRegisterDeclaration>,
        body: Vec<AsmOp>,
        returns: Option<(AsmRegister, Span)>,
        whole_block_span: Span,
    },
    // like a variable expression but it has multiple parts,
    // like looking up a field in a struct
    StructFieldAccess {
        prefix: Box<TyExpression>,
        field_to_access: TyStructField,
        field_instantiation_span: Span,
        /// Final resolved type of the `prefix` part
        /// of the expression. This will always be
        /// a [TypeId] of a struct, never an alias
        /// or a reference to a struct.
        /// The original parent might be an alias
        /// or a direct or indirect reference to a
        /// struct.
        resolved_type_of_parent: TypeId,
    },
    TupleElemAccess {
        prefix: Box<TyExpression>,
        elem_to_access_num: usize,
        /// Final resolved type of the `prefix` part
        /// of the expression. This will always be
        /// a [TypeId] of a tuple, never an alias
        /// or a reference to a tuple.
        /// The original parent might be an alias
        /// or a direct or indirect reference to a
        /// tuple.
        resolved_type_of_parent: TypeId,
        elem_to_access_span: Span,
    },
    EnumInstantiation {
        enum_ref: DeclRef<DeclId<TyEnumDecl>>,
        /// for printing
        variant_name: Ident,
        tag: usize,
        contents: Option<Box<TyExpression>>,
        /// If there is an error regarding this instantiation of the enum,
        /// use these spans as it points to the call site and not the declaration.
        /// They are also used in the language server.
        variant_instantiation_span: Span,
        call_path_binding: TypeBinding<CallPath>,
        /// The enum type, can be a type alias.
        call_path_decl: ty::TyDecl,
    },
    AbiCast {
        abi_name: CallPath,
        address: Box<TyExpression>,
        #[allow(dead_code)]
        // this span may be used for errors in the future, although it is not right now.
        span: Span,
    },
    StorageAccess(TyStorageAccess),
    IntrinsicFunction(TyIntrinsicFunctionKind),
    /// a zero-sized type-system-only compile-time thing that is used for constructing ABI casts.
    AbiName(AbiName),
    /// grabs the enum tag from the particular enum and variant of the `exp`
    EnumTag {
        exp: Box<TyExpression>,
    },
    /// performs an unsafe cast from the `exp` to the type of the given enum `variant`
    UnsafeDowncast {
        exp: Box<TyExpression>,
        variant: TyEnumVariant,
        /// Should contain a TyDecl to either an enum or a type alias.
        call_path_decl: ty::TyDecl,
    },
    WhileLoop {
        condition: Box<TyExpression>,
        body: TyCodeBlock,
    },
    ForLoop {
        desugared: Box<TyExpression>,
    },
    Break,
    Continue,
    Reassignment(Box<TyReassignment>),
    ImplicitReturn(Box<TyExpression>),
    Return(Box<TyExpression>),
    Panic(Box<TyExpression>),
    Ref(Box<TyExpression>),
    Deref(Box<TyExpression>),
}

impl TyExpressionVariant {
    pub fn as_literal(&self) -> Option<&Literal> {
        match self {
            TyExpressionVariant::Literal(v) => Some(v),
            _ => None,
        }
    }
}

impl EqWithEngines for TyExpressionVariant {}
impl PartialEqWithEngines for TyExpressionVariant {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        let type_engine = ctx.engines().te();
        match (self, other) {
            (Self::Literal(l0), Self::Literal(r0)) => l0 == r0,
            (
                Self::FunctionApplication {
                    call_path: l_name,
                    arguments: l_arguments,
                    fn_ref: l_fn_ref,
                    ..
                },
                Self::FunctionApplication {
                    call_path: r_name,
                    arguments: r_arguments,
                    fn_ref: r_fn_ref,
                    ..
                },
            ) => {
                l_name == r_name
                    && l_arguments.len() == r_arguments.len()
                    && l_arguments
                        .iter()
                        .zip(r_arguments.iter())
                        .all(|((xa, xb), (ya, yb))| xa == ya && xb.eq(yb, ctx))
                    && l_fn_ref.eq(r_fn_ref, ctx)
            }
            (
                Self::LazyOperator {
                    op: l_op,
                    lhs: l_lhs,
                    rhs: l_rhs,
                },
                Self::LazyOperator {
                    op: r_op,
                    lhs: r_lhs,
                    rhs: r_rhs,
                },
            ) => l_op == r_op && (**l_lhs).eq(&(**r_lhs), ctx) && (**l_rhs).eq(&(**r_rhs), ctx),
            (
                Self::ConstantExpression {
                    call_path: l_call_path,
                    span: l_span,
                    decl: _,
                },
                Self::ConstantExpression {
                    call_path: r_call_path,
                    span: r_span,
                    decl: _,
                },
            ) => l_call_path == r_call_path && l_span == r_span,
            (
                Self::VariableExpression {
                    name: l_name,
                    span: l_span,
                    mutability: l_mutability,
                    call_path: _,
                },
                Self::VariableExpression {
                    name: r_name,
                    span: r_span,
                    mutability: r_mutability,
                    call_path: _,
                },
            ) => l_name == r_name && l_span == r_span && l_mutability == r_mutability,
            (Self::Tuple { fields: l_fields }, Self::Tuple { fields: r_fields }) => {
                l_fields.eq(r_fields, ctx)
            }
            (
                Self::ArrayExplicit {
                    contents: l_contents,
                    ..
                },
                Self::ArrayExplicit {
                    contents: r_contents,
                    ..
                },
            ) => l_contents.eq(r_contents, ctx),
            (
                Self::ArrayIndex {
                    prefix: l_prefix,
                    index: l_index,
                },
                Self::ArrayIndex {
                    prefix: r_prefix,
                    index: r_index,
                },
            ) => (**l_prefix).eq(&**r_prefix, ctx) && (**l_index).eq(&**r_index, ctx),
            (
                Self::StructExpression {
                    struct_id: l_struct_id,
                    fields: l_fields,
                    instantiation_span: l_span,
                    call_path_binding: _,
                },
                Self::StructExpression {
                    struct_id: r_struct_id,
                    fields: r_fields,
                    instantiation_span: r_span,
                    call_path_binding: _,
                },
            ) => {
                PartialEqWithEngines::eq(&l_struct_id, &r_struct_id, ctx)
                    && l_fields.eq(r_fields, ctx)
                    && l_span == r_span
            }
            (Self::CodeBlock(l0), Self::CodeBlock(r0)) => l0.eq(r0, ctx),
            (
                Self::IfExp {
                    condition: l_condition,
                    then: l_then,
                    r#else: l_r,
                },
                Self::IfExp {
                    condition: r_condition,
                    then: r_then,
                    r#else: r_r,
                },
            ) => {
                (**l_condition).eq(&**r_condition, ctx)
                    && (**l_then).eq(&**r_then, ctx)
                    && if let (Some(l), Some(r)) = (l_r, r_r) {
                        (**l).eq(&**r, ctx)
                    } else {
                        true
                    }
            }
            (
                Self::AsmExpression {
                    registers: l_registers,
                    body: l_body,
                    returns: l_returns,
                    ..
                },
                Self::AsmExpression {
                    registers: r_registers,
                    body: r_body,
                    returns: r_returns,
                    ..
                },
            ) => {
                l_registers.eq(r_registers, ctx)
                    && l_body.clone() == r_body.clone()
                    && l_returns == r_returns
            }
            (
                Self::StructFieldAccess {
                    prefix: l_prefix,
                    field_to_access: l_field_to_access,
                    resolved_type_of_parent: l_resolved_type_of_parent,
                    ..
                },
                Self::StructFieldAccess {
                    prefix: r_prefix,
                    field_to_access: r_field_to_access,
                    resolved_type_of_parent: r_resolved_type_of_parent,
                    ..
                },
            ) => {
                (**l_prefix).eq(&**r_prefix, ctx)
                    && l_field_to_access.eq(r_field_to_access, ctx)
                    && type_engine
                        .get(*l_resolved_type_of_parent)
                        .eq(&type_engine.get(*r_resolved_type_of_parent), ctx)
            }
            (
                Self::TupleElemAccess {
                    prefix: l_prefix,
                    elem_to_access_num: l_elem_to_access_num,
                    resolved_type_of_parent: l_resolved_type_of_parent,
                    ..
                },
                Self::TupleElemAccess {
                    prefix: r_prefix,
                    elem_to_access_num: r_elem_to_access_num,
                    resolved_type_of_parent: r_resolved_type_of_parent,
                    ..
                },
            ) => {
                (**l_prefix).eq(&**r_prefix, ctx)
                    && l_elem_to_access_num == r_elem_to_access_num
                    && type_engine
                        .get(*l_resolved_type_of_parent)
                        .eq(&type_engine.get(*r_resolved_type_of_parent), ctx)
            }
            (
                Self::EnumInstantiation {
                    enum_ref: l_enum_ref,
                    variant_name: l_variant_name,
                    tag: l_tag,
                    contents: l_contents,
                    ..
                },
                Self::EnumInstantiation {
                    enum_ref: r_enum_ref,
                    variant_name: r_variant_name,
                    tag: r_tag,
                    contents: r_contents,
                    ..
                },
            ) => {
                l_enum_ref.eq(r_enum_ref, ctx)
                    && l_variant_name == r_variant_name
                    && l_tag == r_tag
                    && if let (Some(l_contents), Some(r_contents)) = (l_contents, r_contents) {
                        (**l_contents).eq(&**r_contents, ctx)
                    } else {
                        true
                    }
            }
            (
                Self::AbiCast {
                    abi_name: l_abi_name,
                    address: l_address,
                    ..
                },
                Self::AbiCast {
                    abi_name: r_abi_name,
                    address: r_address,
                    ..
                },
            ) => l_abi_name == r_abi_name && (**l_address).eq(&**r_address, ctx),
            (Self::IntrinsicFunction(l_kind), Self::IntrinsicFunction(r_kind)) => {
                l_kind.eq(r_kind, ctx)
            }
            (
                Self::UnsafeDowncast {
                    exp: l_exp,
                    variant: l_variant,
                    call_path_decl: _,
                },
                Self::UnsafeDowncast {
                    exp: r_exp,
                    variant: r_variant,
                    call_path_decl: _,
                },
            ) => l_exp.eq(r_exp, ctx) && l_variant.eq(r_variant, ctx),
            (Self::EnumTag { exp: l_exp }, Self::EnumTag { exp: r_exp }) => l_exp.eq(r_exp, ctx),
            (Self::StorageAccess(l_exp), Self::StorageAccess(r_exp)) => l_exp.eq(r_exp, ctx),
            (
                Self::WhileLoop {
                    body: l_body,
                    condition: l_condition,
                },
                Self::WhileLoop {
                    body: r_body,
                    condition: r_condition,
                },
            ) => l_body.eq(r_body, ctx) && l_condition.eq(r_condition, ctx),
            (l, r) => std::mem::discriminant(l) == std::mem::discriminant(r),
        }
    }
}

impl HashWithEngines for TyExpressionVariant {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let type_engine = engines.te();
        std::mem::discriminant(self).hash(state);
        match self {
            Self::Literal(lit) => {
                lit.hash(state);
            }
            Self::FunctionApplication {
                call_path,
                arguments,
                fn_ref,
                // these fields are not hashed because they aren't relevant/a
                // reliable source of obj v. obj distinction
                contract_call_params: _,
                selector: _,
                type_binding: _,
                method_target: _,
                ..
            } => {
                call_path.hash(state);
                fn_ref.hash(state, engines);
                arguments.iter().for_each(|(name, arg)| {
                    name.hash(state);
                    arg.hash(state, engines);
                });
            }
            Self::LazyOperator { op, lhs, rhs } => {
                op.hash(state);
                lhs.hash(state, engines);
                rhs.hash(state, engines);
            }
            Self::ConstantExpression {
                decl: const_decl,
                span: _,
                call_path: _,
            } => {
                const_decl.hash(state, engines);
            }
            Self::ConfigurableExpression {
                decl: const_decl,
                span: _,
                call_path: _,
            } => {
                const_decl.hash(state, engines);
            }
            Self::ConstGenericExpression {
                decl: const_generic_decl,
                span: _,
                call_path: _,
            } => {
                const_generic_decl.name().hash(state);
            }
            Self::VariableExpression {
                name,
                mutability,
                // these fields are not hashed because they aren't relevant/a
                // reliable source of obj v. obj distinction
                call_path: _,
                span: _,
            } => {
                name.hash(state);
                mutability.hash(state);
            }
            Self::Tuple { fields } => {
                fields.hash(state, engines);
            }
            Self::ArrayExplicit {
                contents,
                elem_type: _,
            } => {
                contents.hash(state, engines);
            }
            Self::ArrayRepeat {
                value,
                length,
                elem_type: _,
            } => {
                value.hash(state, engines);
                length.hash(state, engines);
            }
            Self::ArrayIndex { prefix, index } => {
                prefix.hash(state, engines);
                index.hash(state, engines);
            }
            Self::StructExpression {
                struct_id,
                fields,
                // these fields are not hashed because they aren't relevant/a
                // reliable source of obj v. obj distinction
                instantiation_span: _,
                call_path_binding: _,
            } => {
                HashWithEngines::hash(&struct_id, state, engines);
                fields.hash(state, engines);
            }
            Self::CodeBlock(contents) => {
                contents.hash(state, engines);
            }
            Self::MatchExp {
                desugared,
                // these fields are not hashed because they aren't relevant/a
                // reliable source of obj v. obj distinction
                scrutinees: _,
            } => {
                desugared.hash(state, engines);
            }
            Self::IfExp {
                condition,
                then,
                r#else,
            } => {
                condition.hash(state, engines);
                then.hash(state, engines);
                if let Some(x) = r#else.as_ref() {
                    x.hash(state, engines)
                }
            }
            Self::AsmExpression {
                registers,
                body,
                returns,
                // these fields are not hashed because they aren't relevant/a
                // reliable source of obj v. obj distinction
                whole_block_span: _,
            } => {
                registers.hash(state, engines);
                body.hash(state);
                returns.hash(state);
            }
            Self::StructFieldAccess {
                prefix,
                field_to_access,
                resolved_type_of_parent,
                // these fields are not hashed because they aren't relevant/a
                // reliable source of obj v. obj distinction
                field_instantiation_span: _,
            } => {
                prefix.hash(state, engines);
                field_to_access.hash(state, engines);
                type_engine
                    .get(*resolved_type_of_parent)
                    .hash(state, engines);
            }
            Self::TupleElemAccess {
                prefix,
                elem_to_access_num,
                resolved_type_of_parent,
                // these fields are not hashed because they aren't relevant/a
                // reliable source of obj v. obj distinction
                elem_to_access_span: _,
            } => {
                prefix.hash(state, engines);
                elem_to_access_num.hash(state);
                type_engine
                    .get(*resolved_type_of_parent)
                    .hash(state, engines);
            }
            Self::EnumInstantiation {
                enum_ref,
                variant_name,
                tag,
                contents,
                // these fields are not hashed because they aren't relevant/a
                // reliable source of obj v. obj distinction
                variant_instantiation_span: _,
                call_path_binding: _,
                call_path_decl: _,
            } => {
                enum_ref.hash(state, engines);
                variant_name.hash(state);
                tag.hash(state);
                if let Some(x) = contents.as_ref() {
                    x.hash(state, engines)
                }
            }
            Self::AbiCast {
                abi_name,
                address,
                // these fields are not hashed because they aren't relevant/a
                // reliable source of obj v. obj distinction
                span: _,
            } => {
                abi_name.hash(state);
                address.hash(state, engines);
            }
            Self::StorageAccess(exp) => {
                exp.hash(state, engines);
            }
            Self::IntrinsicFunction(exp) => {
                exp.hash(state, engines);
            }
            Self::AbiName(name) => {
                name.hash(state);
            }
            Self::EnumTag { exp } => {
                exp.hash(state, engines);
            }
            Self::UnsafeDowncast {
                exp,
                variant,
                call_path_decl: _,
            } => {
                exp.hash(state, engines);
                variant.hash(state, engines);
            }
            Self::WhileLoop { condition, body } => {
                condition.hash(state, engines);
                body.hash(state, engines);
            }
            Self::ForLoop { desugared } => {
                desugared.hash(state, engines);
            }
            Self::Break | Self::Continue | Self::FunctionParameter => {}
            Self::Reassignment(exp) => {
                exp.hash(state, engines);
            }
            Self::ImplicitReturn(exp) | Self::Return(exp) => {
                exp.hash(state, engines);
            }
            Self::Panic(exp) => {
                exp.hash(state, engines);
            }
            Self::Ref(exp) | Self::Deref(exp) => {
                exp.hash(state, engines);
            }
        }
    }
}

impl SubstTypes for TyExpressionVariant {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        use TyExpressionVariant::*;
        match self {
            Literal(..) => HasChanges::No,
            FunctionApplication {
                arguments,
                ref mut fn_ref,
                ref mut method_target,
                ..
            } => has_changes! {
                arguments.subst(ctx);
                if let Some(new_decl_ref) = fn_ref
                    .clone()
                    .subst_types_and_insert_new_with_parent(ctx)
                {
                    fn_ref.replace_id(*new_decl_ref.id());
                    HasChanges::Yes
                } else {
                    HasChanges::No
                };
                method_target.subst(ctx);
            },
            LazyOperator { lhs, rhs, .. } => has_changes! {
                lhs.subst(ctx);
                rhs.subst(ctx);
            },
            ConstantExpression { decl, .. } => decl.subst(ctx),
            ConfigurableExpression { decl, .. } => decl.subst(ctx),
            ConstGenericExpression { decl, .. } => decl.subst(ctx),
            VariableExpression { .. } => HasChanges::No,
            Tuple { fields } => fields.subst(ctx),
            ArrayExplicit {
                ref mut elem_type,
                contents,
            } => has_changes! {
                elem_type.subst(ctx);
                contents.subst(ctx);
            },
            ArrayRepeat {
                ref mut elem_type,
                value,
                length,
            } => has_changes! {
                elem_type.subst(ctx);
                value.subst(ctx);
                length.subst(ctx);
            },
            ArrayIndex { prefix, index } => has_changes! {
                prefix.subst(ctx);
                index.subst(ctx);
            },
            StructExpression {
                struct_id,
                fields,
                instantiation_span: _,
                call_path_binding: _,
            } => has_changes! {
                if let Some(new_struct_ref) = struct_id
                    .clone()
                    .subst_types_and_insert_new(ctx) {
                    struct_id.replace_id(*new_struct_ref.id());
                    HasChanges::Yes
                } else {
                    HasChanges::No
                };
                fields.subst(ctx);
            },
            CodeBlock(block) => block.subst(ctx),
            FunctionParameter => HasChanges::No,
            MatchExp { desugared, .. } => desugared.subst(ctx),
            IfExp {
                condition,
                then,
                r#else,
            } => has_changes! {
                condition.subst(ctx);
                then.subst(ctx);
                r#else.subst(ctx);
            },
            AsmExpression {
                registers, //: Vec<TyAsmRegisterDeclaration>,
                ..
            } => registers.subst(ctx),
            // like a variable expression but it has multiple parts,
            // like looking up a field in a struct
            StructFieldAccess {
                prefix,
                field_to_access,
                ref mut resolved_type_of_parent,
                ..
            } => has_changes! {
                resolved_type_of_parent.subst(ctx);
                field_to_access.subst(ctx);
                prefix.subst(ctx);
            },
            TupleElemAccess {
                prefix,
                ref mut resolved_type_of_parent,
                ..
            } => has_changes! {
                resolved_type_of_parent.subst(ctx);
                prefix.subst(ctx);
            },
            EnumInstantiation {
                enum_ref, contents, ..
            } => has_changes! {
                if let Some(new_enum_ref) = enum_ref
                    .clone()
                    .subst_types_and_insert_new(ctx)
                {
                    enum_ref.replace_id(*new_enum_ref.id());
                    HasChanges::Yes
                } else {
                    HasChanges::No
                };
                contents.subst(ctx);
            },
            AbiCast { address, .. } => address.subst(ctx),
            // storage is never generic and cannot be monomorphized
            StorageAccess { .. } => HasChanges::No,
            IntrinsicFunction(kind) => kind.subst(ctx),
            EnumTag { exp } => exp.subst(ctx),
            UnsafeDowncast {
                exp,
                variant,
                call_path_decl: _,
            } => has_changes! {
                exp.subst(ctx);
                variant.subst(ctx);
            },
            AbiName(_) => HasChanges::No,
            WhileLoop {
                ref mut condition,
                ref mut body,
            } => {
                condition.subst(ctx);
                body.subst(ctx)
            }
            ForLoop { ref mut desugared } => desugared.subst(ctx),
            Break => HasChanges::No,
            Continue => HasChanges::No,
            Reassignment(reassignment) => reassignment.subst(ctx),
            ImplicitReturn(expr) | Return(expr) => expr.subst(ctx),
            Panic(expr) => expr.subst(ctx),
            Ref(exp) | Deref(exp) => exp.subst(ctx),
        }
    }
}

impl ReplaceDecls for TyExpressionVariant {
    fn replace_decls_inner(
        &mut self,
        decl_mapping: &DeclMapping,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
    ) -> Result<bool, ErrorEmitted> {
        handler.scope(|handler| {
            use TyExpressionVariant::*;
            match self {
                Literal(..) => Ok(false),
                FunctionApplication {
                    ref mut fn_ref,
                    ref mut arguments,
                    call_path,
                    ..
                } => {
                    let mut has_changes = false;

                    has_changes |= fn_ref.replace_decls(decl_mapping, handler, ctx)?;

                    for (_, arg) in arguments.iter_mut() {
                        if let Ok(r) = arg.replace_decls(decl_mapping, handler, ctx) {
                            has_changes |= r;
                        }
                    }

                    let decl_engine = ctx.engines().de();
                    let mut method = (*decl_engine.get(fn_ref)).clone();

                    // Finds method implementation for method dummy and replaces it.
                    // This is required because dummy methods don't have type parameters from impl traits.
                    // Thus we use the implemented method that already contains all the required type parameters,
                    // including those from the impl trait.
                    if method.is_trait_method_dummy {
                        if let Some(implementing_for) = method.implementing_for {
                            let arguments_types = arguments
                                .iter()
                                .map(|a| a.1.return_type)
                                .collect::<Vec<_>>();

                            // find method and improve error
                            let find_handler = Handler::default();
                            let r = ctx.find_method_for_type(
                                &find_handler,
                                implementing_for,
                                &[ctx.namespace().current_package_name().clone()],
                                &call_path.suffix,
                                method.return_type.type_id(),
                                &arguments_types,
                                None,
                            );
                            let _ =
                                handler.map_and_emit_errors_from(find_handler, |err| match err {
                                    CompileError::MultipleApplicableItemsInScope {
                                        span,
                                        item_name,
                                        item_kind,
                                        as_traits,
                                    } => {
                                        if let Some(ty) = call_path.prefixes.get(1) {
                                            Some(CompileError::MultipleApplicableItemsInScope {
                                                span,
                                                item_name,
                                                item_kind,
                                                as_traits: as_traits
                                                    .into_iter()
                                                    .map(|(tt, _)| (tt, ty.as_str().to_string()))
                                                    .collect(),
                                            })
                                        } else {
                                            Some(CompileError::MultipleApplicableItemsInScope {
                                                span,
                                                item_name,
                                                item_kind,
                                                as_traits: vec![],
                                            })
                                        }
                                    }
                                    _ => None,
                                });
                            let implementing_type_method_ref = r?;
                            method = (*decl_engine.get(&implementing_type_method_ref)).clone();
                        }
                    }

                    // Handle the trait constraints. This includes checking to see if the trait
                    // constraints are satisfied and replacing old decl ids based on the
                    let mut inner_decl_mapping =
                        GenericTypeParameter::gather_decl_mapping_from_trait_constraints(
                            handler,
                            ctx.by_ref(),
                            &method.type_parameters,
                            method.name.as_str(),
                            &method.name.span(),
                        )?;

                    inner_decl_mapping.extend(decl_mapping);

                    if method.replace_decls(&inner_decl_mapping, handler, ctx)? {
                        decl_engine.replace(*fn_ref.id(), method);
                        has_changes = true;
                    }

                    Ok(has_changes)
                }
                LazyOperator { lhs, rhs, .. } => {
                    let mut has_changes = (*lhs).replace_decls(decl_mapping, handler, ctx)?;
                    has_changes |= (*rhs).replace_decls(decl_mapping, handler, ctx)?;
                    Ok(has_changes)
                }
                ConstantExpression { decl, .. } => decl.replace_decls(decl_mapping, handler, ctx),
                ConfigurableExpression { decl, .. } => {
                    decl.replace_decls(decl_mapping, handler, ctx)
                }
                ConstGenericExpression { .. } => Ok(false),
                VariableExpression { .. } => Ok(false),
                Tuple { fields } => {
                    let mut has_changes = false;
                    for item in fields.iter_mut() {
                        if let Ok(r) = item.replace_decls(decl_mapping, handler, ctx) {
                            has_changes |= r;
                        }
                    }
                    Ok(has_changes)
                }
                ArrayExplicit {
                    elem_type: _,
                    contents,
                } => {
                    let mut has_changes = false;
                    for expr in contents.iter_mut() {
                        if let Ok(r) = expr.replace_decls(decl_mapping, handler, ctx) {
                            has_changes |= r;
                        }
                    }
                    Ok(has_changes)
                }
                ArrayRepeat {
                    elem_type: _,
                    value,
                    length,
                } => {
                    let mut has_changes = (*value).replace_decls(decl_mapping, handler, ctx)?;
                    has_changes |= (*length).replace_decls(decl_mapping, handler, ctx)?;
                    Ok(has_changes)
                }
                ArrayIndex { prefix, index } => {
                    let mut has_changes = false;
                    if let Ok(r) = (*prefix).replace_decls(decl_mapping, handler, ctx) {
                        has_changes |= r;
                    }
                    if let Ok(r) = (*index).replace_decls(decl_mapping, handler, ctx) {
                        has_changes |= r;
                    }
                    Ok(has_changes)
                }
                StructExpression {
                    struct_id: _,
                    fields,
                    instantiation_span: _,
                    call_path_binding: _,
                } => {
                    let mut has_changes = false;
                    for field in fields.iter_mut() {
                        if let Ok(r) = field.replace_decls(decl_mapping, handler, ctx) {
                            has_changes |= r;
                        }
                    }
                    Ok(has_changes)
                }
                CodeBlock(block) => block.replace_decls(decl_mapping, handler, ctx),
                FunctionParameter => Ok(false),
                MatchExp { desugared, .. } => desugared.replace_decls(decl_mapping, handler, ctx),
                IfExp {
                    condition,
                    then,
                    r#else,
                } => {
                    let mut has_changes = false;
                    if let Ok(r) = condition.replace_decls(decl_mapping, handler, ctx) {
                        has_changes |= r;
                    }
                    if let Ok(r) = then.replace_decls(decl_mapping, handler, ctx) {
                        has_changes |= r;
                    }
                    if let Some(r) = r#else
                        .as_mut()
                        .and_then(|expr| expr.replace_decls(decl_mapping, handler, ctx).ok())
                    {
                        has_changes |= r;
                    }
                    Ok(has_changes)
                }
                AsmExpression { .. } => Ok(false),
                StructFieldAccess { prefix, .. } => {
                    prefix.replace_decls(decl_mapping, handler, ctx)
                }
                TupleElemAccess { prefix, .. } => prefix.replace_decls(decl_mapping, handler, ctx),
                EnumInstantiation {
                    enum_ref: _,
                    contents,
                    ..
                } => {
                    // TODO: replace enum decl
                    //enum_decl.replace_decls(decl_mapping);
                    if let Some(ref mut contents) = contents {
                        contents.replace_decls(decl_mapping, handler, ctx)
                    } else {
                        Ok(false)
                    }
                }
                AbiCast { address, .. } => address.replace_decls(decl_mapping, handler, ctx),
                StorageAccess { .. } => Ok(false),
                IntrinsicFunction(TyIntrinsicFunctionKind { arguments, .. }) => {
                    let mut has_changes = false;
                    for expr in arguments.iter_mut() {
                        if let Ok(r) = expr.replace_decls(decl_mapping, handler, ctx) {
                            has_changes |= r;
                        }
                    }
                    Ok(has_changes)
                }
                EnumTag { exp } => exp.replace_decls(decl_mapping, handler, ctx),
                UnsafeDowncast { exp, .. } => exp.replace_decls(decl_mapping, handler, ctx),
                AbiName(_) => Ok(false),
                WhileLoop {
                    ref mut condition,
                    ref mut body,
                } => {
                    let mut has_changes = false;
                    if let Ok(r) = condition.replace_decls(decl_mapping, handler, ctx) {
                        has_changes |= r;
                    }
                    if let Ok(r) = body.replace_decls(decl_mapping, handler, ctx) {
                        has_changes |= r;
                    }
                    Ok(has_changes)
                }
                ForLoop { ref mut desugared } => {
                    desugared.replace_decls(decl_mapping, handler, ctx)
                }
                Break => Ok(false),
                Continue => Ok(false),
                Reassignment(reassignment) => {
                    reassignment.replace_decls(decl_mapping, handler, ctx)
                }
                ImplicitReturn(expr) | Return(expr) => {
                    expr.replace_decls(decl_mapping, handler, ctx)
                }
                Panic(expr) => expr.replace_decls(decl_mapping, handler, ctx),
                Ref(exp) | Deref(exp) => exp.replace_decls(decl_mapping, handler, ctx),
            }
        })
    }
}

impl TypeCheckAnalysis for TyExpressionVariant {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        match self {
            TyExpressionVariant::Literal(_) => {}
            TyExpressionVariant::FunctionApplication {
                fn_ref, arguments, ..
            } => {
                let fn_decl_id = ctx.get_normalized_fn_node_id(fn_ref.id());

                let fn_node = ctx.get_node_for_fn_decl(&fn_decl_id);
                if let Some(fn_node) = fn_node {
                    ctx.add_edge_from_current(
                        fn_node,
                        TyNodeDepGraphEdge(TyNodeDepGraphEdgeInfo::FnApp),
                    );

                    if !ctx.node_stack.contains(&fn_node) {
                        let _ = fn_decl_id.type_check_analyze(handler, ctx);
                    }
                }

                // Unify arguments that are still not concrete
                let decl = ctx.engines.de().get(fn_ref.id());

                use crate::type_system::unify::unifier::*;
                let unifier = Unifier::new(ctx.engines, "", UnifyKind::Default);

                for (decl_param, arg) in decl.parameters.iter().zip(arguments.iter()) {
                    unifier.unify(
                        handler,
                        arg.1.return_type,
                        decl_param.type_argument.type_id(),
                        &Span::dummy(),
                        false,
                    );
                }
            }
            TyExpressionVariant::LazyOperator { lhs, rhs, .. } => {
                lhs.type_check_analyze(handler, ctx)?;
                rhs.type_check_analyze(handler, ctx)?
            }
            TyExpressionVariant::ConstantExpression { decl, .. } => {
                decl.type_check_analyze(handler, ctx)?
            }
            TyExpressionVariant::ConfigurableExpression { decl, .. } => {
                decl.type_check_analyze(handler, ctx)?
            }
            TyExpressionVariant::ConstGenericExpression { decl, .. } => {
                decl.type_check_analyze(handler, ctx)?
            }
            TyExpressionVariant::VariableExpression { .. } => {}
            TyExpressionVariant::Tuple { fields } => {
                for field in fields.iter() {
                    field.type_check_analyze(handler, ctx)?
                }
            }
            TyExpressionVariant::ArrayExplicit { contents, .. } => {
                for elem in contents.iter() {
                    elem.type_check_analyze(handler, ctx)?
                }
            }
            TyExpressionVariant::ArrayRepeat { value, length, .. } => {
                value.type_check_analyze(handler, ctx)?;
                length.type_check_analyze(handler, ctx)?;
            }
            TyExpressionVariant::ArrayIndex { prefix, index } => {
                prefix.type_check_analyze(handler, ctx)?;
                index.type_check_analyze(handler, ctx)?;
            }
            TyExpressionVariant::StructExpression { fields: _, .. } => {}
            TyExpressionVariant::CodeBlock(block) => {
                block.type_check_analyze(handler, ctx)?;
            }
            TyExpressionVariant::FunctionParameter => {}
            TyExpressionVariant::MatchExp {
                desugared,
                scrutinees: _,
            } => {
                desugared.type_check_analyze(handler, ctx)?;
            }
            TyExpressionVariant::IfExp {
                condition,
                then,
                r#else,
            } => {
                condition.type_check_analyze(handler, ctx)?;
                then.type_check_analyze(handler, ctx)?;
                if let Some(r#else) = r#else {
                    r#else.type_check_analyze(handler, ctx)?;
                }
            }
            TyExpressionVariant::AsmExpression { .. } => {}
            TyExpressionVariant::StructFieldAccess { prefix, .. } => {
                prefix.type_check_analyze(handler, ctx)?;
            }
            TyExpressionVariant::TupleElemAccess { prefix, .. } => {
                prefix.type_check_analyze(handler, ctx)?;
            }
            TyExpressionVariant::EnumInstantiation { contents, .. } => {
                for expr in contents.iter() {
                    expr.type_check_analyze(handler, ctx)?
                }
            }
            TyExpressionVariant::AbiCast { address, .. } => {
                address.type_check_analyze(handler, ctx)?;
            }
            TyExpressionVariant::StorageAccess(_node) => {}
            TyExpressionVariant::IntrinsicFunction(node) => {
                for arg in node.arguments.iter() {
                    arg.type_check_analyze(handler, ctx)?
                }
            }
            TyExpressionVariant::AbiName(_node) => {}
            TyExpressionVariant::EnumTag { exp } => {
                exp.type_check_analyze(handler, ctx)?;
            }
            TyExpressionVariant::UnsafeDowncast { exp, .. } => {
                exp.type_check_analyze(handler, ctx)?;
            }
            TyExpressionVariant::WhileLoop { condition, body } => {
                condition.type_check_analyze(handler, ctx)?;
                body.type_check_analyze(handler, ctx)?;
            }
            TyExpressionVariant::ForLoop { desugared } => {
                desugared.type_check_analyze(handler, ctx)?;
            }
            TyExpressionVariant::Break => {}
            TyExpressionVariant::Continue => {}
            TyExpressionVariant::Reassignment(node) => {
                node.type_check_analyze(handler, ctx)?;
            }
            TyExpressionVariant::ImplicitReturn(exp) | TyExpressionVariant::Return(exp) => {
                exp.type_check_analyze(handler, ctx)?;
            }
            TyExpressionVariant::Panic(exp) => {
                exp.type_check_analyze(handler, ctx)?;
            }
            TyExpressionVariant::Ref(exp) | TyExpressionVariant::Deref(exp) => {
                exp.type_check_analyze(handler, ctx)?;
            }
        }
        Ok(())
    }
}

impl TypeCheckFinalization for TyExpressionVariant {
    fn type_check_finalize(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        handler.scope(|handler| {
            match self {
                TyExpressionVariant::ConstGenericExpression { .. } => {}
                TyExpressionVariant::Literal(_) => {}
                TyExpressionVariant::FunctionApplication { arguments, .. } => {
                    for (_, arg) in arguments.iter_mut() {
                        let _ = arg.type_check_finalize(handler, ctx);
                    }
                }
                TyExpressionVariant::LazyOperator { lhs, rhs, .. } => {
                    lhs.type_check_finalize(handler, ctx)?;
                    rhs.type_check_finalize(handler, ctx)?
                }
                TyExpressionVariant::ConstantExpression { decl, .. } => {
                    decl.type_check_finalize(handler, ctx)?
                }
                TyExpressionVariant::ConfigurableExpression { decl, .. } => {
                    decl.type_check_finalize(handler, ctx)?
                }
                TyExpressionVariant::VariableExpression { .. } => {}
                TyExpressionVariant::Tuple { fields } => {
                    for field in fields.iter_mut() {
                        field.type_check_finalize(handler, ctx)?
                    }
                }
                TyExpressionVariant::ArrayExplicit { contents, .. } => {
                    for elem in contents.iter_mut() {
                        elem.type_check_finalize(handler, ctx)?
                    }
                }
                TyExpressionVariant::ArrayRepeat { value, length, .. } => {
                    value.type_check_finalize(handler, ctx)?;
                    length.type_check_finalize(handler, ctx)?;
                }
                TyExpressionVariant::ArrayIndex { prefix, index } => {
                    prefix.type_check_finalize(handler, ctx)?;
                    index.type_check_finalize(handler, ctx)?;
                }
                TyExpressionVariant::StructExpression { fields, .. } => {
                    for field in fields.iter_mut() {
                        field.type_check_finalize(handler, ctx)?;
                    }
                }
                TyExpressionVariant::CodeBlock(block) => {
                    block.type_check_finalize(handler, ctx)?;
                }
                TyExpressionVariant::FunctionParameter => {}
                TyExpressionVariant::MatchExp {
                    desugared,
                    scrutinees,
                } => {
                    desugared.type_check_finalize(handler, ctx)?;
                    for scrutinee in scrutinees.iter_mut() {
                        scrutinee.type_check_finalize(handler, ctx)?
                    }
                }
                TyExpressionVariant::IfExp {
                    condition,
                    then,
                    r#else,
                } => {
                    condition.type_check_finalize(handler, ctx)?;
                    then.type_check_finalize(handler, ctx)?;
                    if let Some(ref mut r#else) = r#else {
                        r#else.type_check_finalize(handler, ctx)?;
                    }
                }
                TyExpressionVariant::AsmExpression { .. } => {}
                TyExpressionVariant::StructFieldAccess { prefix, .. } => {
                    prefix.type_check_finalize(handler, ctx)?;
                }
                TyExpressionVariant::TupleElemAccess { prefix, .. } => {
                    prefix.type_check_finalize(handler, ctx)?;
                }
                TyExpressionVariant::EnumInstantiation { contents, .. } => {
                    for expr in contents.iter_mut() {
                        expr.type_check_finalize(handler, ctx)?
                    }
                }
                TyExpressionVariant::AbiCast { address, .. } => {
                    address.type_check_finalize(handler, ctx)?;
                }
                TyExpressionVariant::StorageAccess(_) => {
                    todo!("")
                }
                TyExpressionVariant::IntrinsicFunction(kind) => {
                    for expr in kind.arguments.iter_mut() {
                        expr.type_check_finalize(handler, ctx)?;
                    }
                }
                TyExpressionVariant::AbiName(_) => {
                    todo!("")
                }
                TyExpressionVariant::EnumTag { exp } => {
                    exp.type_check_finalize(handler, ctx)?;
                }
                TyExpressionVariant::UnsafeDowncast { exp, .. } => {
                    exp.type_check_finalize(handler, ctx)?;
                }
                TyExpressionVariant::WhileLoop { condition, body } => {
                    condition.type_check_finalize(handler, ctx)?;
                    body.type_check_finalize(handler, ctx)?;
                }
                TyExpressionVariant::ForLoop { desugared } => {
                    desugared.type_check_finalize(handler, ctx)?;
                }
                TyExpressionVariant::Break => {}
                TyExpressionVariant::Continue => {}
                TyExpressionVariant::Reassignment(node) => {
                    node.type_check_finalize(handler, ctx)?;
                }
                TyExpressionVariant::ImplicitReturn(exp) | TyExpressionVariant::Return(exp) => {
                    exp.type_check_finalize(handler, ctx)?;
                }
                TyExpressionVariant::Panic(exp) => {
                    exp.type_check_finalize(handler, ctx)?;
                }
                TyExpressionVariant::Ref(exp) | TyExpressionVariant::Deref(exp) => {
                    exp.type_check_finalize(handler, ctx)?;
                }
            }
            Ok(())
        })
    }
}

impl UpdateConstantExpression for TyExpressionVariant {
    fn update_constant_expression(&mut self, engines: &Engines, implementing_type: &TyDecl) {
        use TyExpressionVariant::*;
        match self {
            Literal(..) => (),
            FunctionApplication { .. } => (),
            LazyOperator { lhs, rhs, .. } => {
                (*lhs).update_constant_expression(engines, implementing_type);
                (*rhs).update_constant_expression(engines, implementing_type);
            }
            ConstantExpression { ref mut decl, .. } => {
                if let Some(impl_const) =
                    find_const_decl_from_impl(implementing_type, engines.de(), decl)
                {
                    *decl = Box::new(impl_const);
                }
            }
            ConfigurableExpression { .. } => {
                unreachable!()
            }
            ConstGenericExpression { .. } => {}
            VariableExpression { .. } => (),
            Tuple { fields } => fields
                .iter_mut()
                .for_each(|x| x.update_constant_expression(engines, implementing_type)),
            ArrayExplicit {
                contents,
                elem_type: _,
            } => contents
                .iter_mut()
                .for_each(|x| x.update_constant_expression(engines, implementing_type)),
            ArrayRepeat {
                elem_type: _,
                value,
                length,
            } => {
                value.update_constant_expression(engines, implementing_type);
                length.update_constant_expression(engines, implementing_type);
            }
            ArrayIndex { prefix, index } => {
                (*prefix).update_constant_expression(engines, implementing_type);
                (*index).update_constant_expression(engines, implementing_type);
            }
            StructExpression { fields, .. } => fields.iter_mut().for_each(|x| {
                x.value
                    .update_constant_expression(engines, implementing_type)
            }),
            CodeBlock(block) => {
                block.update_constant_expression(engines, implementing_type);
            }
            FunctionParameter => (),
            MatchExp { desugared, .. } => {
                desugared.update_constant_expression(engines, implementing_type)
            }
            IfExp {
                condition,
                then,
                r#else,
            } => {
                condition.update_constant_expression(engines, implementing_type);
                then.update_constant_expression(engines, implementing_type);
                if let Some(ref mut r#else) = r#else {
                    r#else.update_constant_expression(engines, implementing_type);
                }
            }
            AsmExpression { .. } => {}
            StructFieldAccess { prefix, .. } => {
                prefix.update_constant_expression(engines, implementing_type);
            }
            TupleElemAccess { prefix, .. } => {
                prefix.update_constant_expression(engines, implementing_type);
            }
            EnumInstantiation {
                enum_ref: _,
                contents,
                ..
            } => {
                if let Some(ref mut contents) = contents {
                    contents.update_constant_expression(engines, implementing_type);
                };
            }
            AbiCast { address, .. } => {
                address.update_constant_expression(engines, implementing_type)
            }
            StorageAccess { .. } => (),
            IntrinsicFunction(_) => {}
            EnumTag { exp } => {
                exp.update_constant_expression(engines, implementing_type);
            }
            UnsafeDowncast { exp, .. } => {
                exp.update_constant_expression(engines, implementing_type);
            }
            AbiName(_) => (),
            WhileLoop {
                ref mut condition,
                ref mut body,
            } => {
                condition.update_constant_expression(engines, implementing_type);
                body.update_constant_expression(engines, implementing_type);
            }
            ForLoop { ref mut desugared } => {
                desugared.update_constant_expression(engines, implementing_type);
            }
            Break => (),
            Continue => (),
            Reassignment(reassignment) => {
                reassignment.update_constant_expression(engines, implementing_type)
            }
            ImplicitReturn(exp) | Return(exp) => {
                exp.update_constant_expression(engines, implementing_type)
            }
            Panic(exp) => exp.update_constant_expression(engines, implementing_type),
            Ref(exp) | Deref(exp) => exp.update_constant_expression(engines, implementing_type),
        }
    }
}

fn find_const_decl_from_impl(
    implementing_type: &TyDecl,
    decl_engine: &DeclEngine,
    const_decl: &TyConstantDecl,
) -> Option<TyConstantDecl> {
    match implementing_type {
        TyDecl::ImplSelfOrTrait(ImplSelfOrTrait { decl_id, .. }) => {
            let impl_trait = decl_engine.get_impl_self_or_trait(&decl_id.clone());
            impl_trait
                .items
                .iter()
                .find(|item| match item {
                    TyTraitItem::Constant(decl_id) => {
                        let trait_const_decl =
                            (*decl_engine.get_constant(&decl_id.clone())).clone();
                        const_decl.name().eq(trait_const_decl.name())
                    }
                    _ => false,
                })
                .map(|item| match item {
                    TyTraitItem::Constant(decl_id) => (*decl_engine.get_constant(decl_id)).clone(),
                    _ => unreachable!(),
                })
        }
        TyDecl::AbiDecl(AbiDecl {
            decl_id: _decl_id, ..
        }) => todo!(""),
        _ => unreachable!(),
    }
}

impl DisplayWithEngines for TyExpressionVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        // TODO: Implement user-friendly display strings if needed.
        DebugWithEngines::fmt(self, f, engines)
    }
}

impl DebugWithEngines for TyExpressionVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        let s = match self {
            TyExpressionVariant::ConstGenericExpression { call_path, .. } => {
                format!("const generic {}", call_path.span().as_str())
            }
            TyExpressionVariant::Literal(lit) => format!("literal {lit}"),
            TyExpressionVariant::FunctionApplication {
                call_path: name, ..
            } => {
                format!("\"{}\" fn entry", name.suffix.as_str())
            }
            TyExpressionVariant::LazyOperator { op, .. } => match op {
                LazyOp::And => "&&".into(),
                LazyOp::Or => "||".into(),
            },
            TyExpressionVariant::Tuple { fields } => {
                let fields = fields
                    .iter()
                    .map(|field| format!("{:?}", engines.help_out(field)))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("tuple({fields})")
            }
            TyExpressionVariant::ArrayExplicit { .. } | TyExpressionVariant::ArrayRepeat { .. } => {
                "array".into()
            }
            TyExpressionVariant::ArrayIndex { .. } => "[..]".into(),
            TyExpressionVariant::StructExpression { struct_id, .. } => {
                let decl = engines.de().get(struct_id);
                format!("\"{}\" struct init", decl.name().as_str())
            }
            TyExpressionVariant::CodeBlock(_) => "code block entry".into(),
            TyExpressionVariant::FunctionParameter => "fn param access".into(),
            TyExpressionVariant::MatchExp { .. } | TyExpressionVariant::IfExp { .. } => {
                "if exp".into()
            }
            TyExpressionVariant::AsmExpression { .. } => "inline asm".into(),
            TyExpressionVariant::AbiCast { abi_name, .. } => {
                format!("abi cast {}", abi_name.suffix.as_str())
            }
            TyExpressionVariant::StructFieldAccess {
                resolved_type_of_parent,
                field_to_access,
                ..
            } => {
                format!(
                    "\"{:?}.{}\" struct field access",
                    engines.help_out(*resolved_type_of_parent),
                    field_to_access.name
                )
            }
            TyExpressionVariant::TupleElemAccess {
                resolved_type_of_parent,
                elem_to_access_num,
                ..
            } => {
                format!(
                    "\"{:?}.{}\" tuple index",
                    engines.help_out(*resolved_type_of_parent),
                    elem_to_access_num
                )
            }
            TyExpressionVariant::ConstantExpression { decl, .. } => {
                format!("\"{}\" constant exp", decl.name().as_str())
            }
            TyExpressionVariant::ConfigurableExpression { decl, .. } => {
                format!("\"{}\" configurable exp", decl.name().as_str())
            }
            TyExpressionVariant::VariableExpression { name, .. } => {
                format!("\"{}\" variable exp", name.as_str())
            }
            TyExpressionVariant::EnumInstantiation {
                tag,
                enum_ref,
                variant_name,
                ..
            } => {
                format!(
                    "{}::{} enum instantiation (tag: {})",
                    enum_ref.name().as_str(),
                    variant_name.as_str(),
                    tag
                )
            }
            TyExpressionVariant::StorageAccess(access) => {
                format!("storage field {} access", access.storage_field_name())
            }
            TyExpressionVariant::IntrinsicFunction(kind) => format!("{:?}", engines.help_out(kind)),
            TyExpressionVariant::AbiName(n) => format!("ABI name {n}"),
            TyExpressionVariant::EnumTag { exp } => {
                format!("({:?} as tag)", engines.help_out(exp.return_type))
            }
            TyExpressionVariant::UnsafeDowncast {
                exp,
                variant,
                call_path_decl,
            } => {
                format!(
                    "({:?} as {}::{})",
                    engines.help_out(exp.return_type),
                    engines.help_out(call_path_decl),
                    variant.name
                )
            }
            TyExpressionVariant::WhileLoop { condition, .. } => {
                format!("while loop on {:?}", engines.help_out(&**condition))
            }
            TyExpressionVariant::ForLoop { .. } => "for loop".to_string(),
            TyExpressionVariant::Break => "break".to_string(),
            TyExpressionVariant::Continue => "continue".to_string(),
            TyExpressionVariant::Reassignment(reassignment) => {
                let target = match &reassignment.lhs {
                    TyReassignmentTarget::DerefAccess { exp, indices } => {
                        let mut target = format!("{:?}", engines.help_out(exp));
                        for index in indices {
                            match index {
                                ProjectionKind::StructField {
                                    name,
                                    field_to_access: _,
                                } => {
                                    target.push('.');
                                    target.push_str(name.as_str());
                                }
                                ProjectionKind::TupleField { index, .. } => {
                                    target.push('.');
                                    target.push_str(index.to_string().as_str());
                                }
                                ProjectionKind::ArrayIndex { index, .. } => {
                                    write!(&mut target, "[{:?}]", engines.help_out(index)).unwrap();
                                }
                            }
                        }
                        target
                    }
                    TyReassignmentTarget::ElementAccess {
                        base_name,
                        base_type: _,
                        indices,
                    } => {
                        let mut target = base_name.to_string();
                        for index in indices {
                            match index {
                                ProjectionKind::StructField {
                                    name,
                                    field_to_access: _,
                                } => {
                                    target.push('.');
                                    target.push_str(name.as_str());
                                }
                                ProjectionKind::TupleField { index, .. } => {
                                    target.push('.');
                                    target.push_str(index.to_string().as_str());
                                }
                                ProjectionKind::ArrayIndex { index, .. } => {
                                    write!(&mut target, "[{:?}]", engines.help_out(index)).unwrap();
                                }
                            }
                        }
                        target
                    }
                };

                format!(
                    "reassignment to {target} = {:?}",
                    engines.help_out(&reassignment.rhs)
                )
            }
            TyExpressionVariant::ImplicitReturn(exp) => {
                format!("implicit return {:?}", engines.help_out(&**exp))
            }
            TyExpressionVariant::Return(exp) => {
                format!("return {:?}", engines.help_out(&**exp))
            }
            TyExpressionVariant::Panic(exp) => {
                format!("panic {:?}", engines.help_out(&**exp))
            }
            TyExpressionVariant::Ref(exp) => {
                format!("&({:?})", engines.help_out(&**exp))
            }
            TyExpressionVariant::Deref(exp) => {
                format!("*({:?})", engines.help_out(&**exp))
            }
        };
        write!(f, "{s}")
    }
}

impl TyExpressionVariant {
    /// Returns `self` as a literal, if possible.
    pub(crate) fn extract_literal_value(&self) -> Option<Literal> {
        match self {
            TyExpressionVariant::Literal(value) => Some(value.clone()),
            _ => None,
        }
    }
}

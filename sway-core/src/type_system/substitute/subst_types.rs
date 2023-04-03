use std::{collections::HashMap, hash::Hash};

use sway_types::{Named, Spanned};

use crate::{
    decl_engine::*,
    language::{parsed::Supertrait, ty::*},
    semantic_analysis::TypeCheckContext,
    type_system::priv_prelude::*,
    Engines,
};

pub trait SubstTypes
where
    Self: Sized,
{
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self>;

    fn recur_subst(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        if subst_list.is_empty() {
            Substituted::new(self, false)
        } else {
            self.subst_inner(engines, subst_list, kind)
        }
    }

    fn subst(self, engines: Engines<'_>, subst_list: &SubstList) -> Substituted<Self> {
        self.recur_subst(engines, subst_list, SubstitutionKind::Subst)
    }

    fn flatten_subst(self, engines: Engines<'_>, subst_list: &SubstList) -> Substituted<Self> {
        self.recur_subst(engines, subst_list, SubstitutionKind::Flatten)
    }

    fn fold_subst(self, engines: Engines<'_>) -> Substituted<Self> {
        self.recur_subst(engines, &SubstList::new(), SubstitutionKind::Fold)
    }

    fn apply_subst(self, ctx: &TypeCheckContext) -> Substituted<Self> {
        self.subst(ctx.engines(), &ctx.namespace.subst_list_stack_top())
    }
}

impl<T> SubstTypes for DeclRef<DeclId<T>>
where
    DeclEngine: DeclEngineGet<DeclId<T>, T>,
    T: SubstTypes + Named + Spanned,
    DeclEngine: DeclEngineInsert<T>,
{
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        let decl_engine = engines.de();
        let (name, id, sl, decl_span) = self.into_parts();
        match kind {
            SubstitutionKind::Subst => sl
                .recur_subst(engines, subst_list, SubstitutionKind::Subst)
                .map(|sl| DeclRef::new(name, id, sl, decl_span)),
            SubstitutionKind::Fold => sl
                .recur_subst(engines, subst_list, kind)
                .map(|sl| DeclRef::new(name, id, sl, decl_span))
                .and_then(|decl_ref| {
                    decl_ref.fold(|decl_id, subst_list| {
                        decl_engine
                            .get(&decl_id)
                            .recur_subst(engines, &subst_list, SubstitutionKind::Subst)
                            .map(|decl| decl_engine.insert(decl, subst_list))
                    })
                }),
            SubstitutionKind::Flatten => sl
                .recur_subst(engines, subst_list, kind)
                .map(|sl| DeclRef::new(name, id, sl, decl_span))
                .and_then(|decl_ref| {
                    decl_ref.fold(|decl_id, subst_list| {
                        decl_engine
                            .get(&decl_id)
                            .recur_subst(engines, &subst_list, SubstitutionKind::Flatten)
                            .map(|decl| decl_engine.insert(decl, subst_list))
                    })
                }),
        }
    }
}

impl<T> SubstTypes for Vec<T>
where
    T: SubstTypes,
{
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        self.into_iter()
            .map(|elem| elem.recur_subst(engines, subst_list, kind))
            .collect()
    }
}

impl<T> SubstTypes for Option<T>
where
    T: SubstTypes,
{
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        match self {
            Some(inner) => inner
                .recur_subst(engines, subst_list, kind)
                .map(|inner| Some(inner)),
            None => Substituted::new(None, false),
        }
    }
}

impl<K, V> SubstTypes for HashMap<K, V>
where
    K: Eq + Hash,
    V: SubstTypes,
{
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        self.into_iter()
            .map(|(k, v)| v.recur_subst(engines, subst_list, kind).map(|v| (k, v)))
            .collect()
    }
}

impl SubstTypes for TyStructDecl {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        let TyStructDecl {
            fields,
            type_parameters,
            // these fields contain no types
            call_path,
            visibility,
            span,
            attributes,
        } = self;
        fields
            .recur_subst(engines, subst_list, kind)
            .and(type_parameters.recur_subst(engines, subst_list, kind))
            .and_map(|fields, type_parameters| TyStructDecl {
                fields,
                type_parameters,
                call_path,
                visibility,
                span,
                attributes,
            })
    }
}

impl SubstTypes for TyEnumDecl {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        let TyEnumDecl {
            type_parameters,
            variants,
            // these fields contain no types
            call_path,
            attributes,
            span,
            visibility,
        } = self;
        type_parameters
            .recur_subst(engines, subst_list, kind)
            .and(variants.recur_subst(engines, subst_list, kind))
            .and_map(|type_parameters, variants| TyEnumDecl {
                type_parameters,
                variants,
                call_path,
                attributes,
                span,
                visibility,
            })
    }
}

impl SubstTypes for TyTraitDecl {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        let TyTraitDecl {
            type_parameters,
            interface_surface,
            items,
            supertraits,
            // these fields contain no types
            name,
            visibility,
            attributes,
            span,
        } = self;
        type_parameters
            .recur_subst(engines, subst_list, kind)
            .and(interface_surface.recur_subst(engines, subst_list, kind))
            .and_two(items.recur_subst(engines, subst_list, kind))
            .and_three(supertraits.recur_subst(engines, subst_list, kind))
            .and_map(
                |type_parameters, interface_surface, items, supertraits| TyTraitDecl {
                    type_parameters,
                    interface_surface,
                    items,
                    supertraits,
                    name,
                    visibility,
                    attributes,
                    span,
                },
            )
    }
}

impl SubstTypes for TyTraitInterfaceItem {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        use TyTraitInterfaceItem::*;
        match self {
            TraitFn(trait_ref) => trait_ref
                .recur_subst(engines, subst_list, kind)
                .map(TraitFn),
            Constant(constant_ref) => constant_ref
                .recur_subst(engines, subst_list, kind)
                .map(Constant),
        }
    }
}

impl SubstTypes for TyTraitFn {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        let TyTraitFn {
            parameters,
            return_type,
            // these fields contain no types
            name,
            purity,
            return_type_span,
            attributes,
        } = self;
        parameters
            .recur_subst(engines, subst_list, kind)
            .and(return_type.recur_subst(engines, subst_list, kind))
            .and_map(|parameters, return_type| TyTraitFn {
                parameters,
                return_type,
                name,
                purity,
                return_type_span,
                attributes,
            })
    }
}

impl SubstTypes for TyTraitItem {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        use TyTraitItem::*;
        match self {
            Fn(fn_ref) => fn_ref
                .recur_subst(engines, subst_list, kind)
                .map(TyTraitItem::Fn),
            Constant(constant_ref) => constant_ref
                .recur_subst(engines, subst_list, kind)
                .map(Constant),
        }
    }
}

impl SubstTypes for TyConstantDecl {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        // constants should not contain generics at this point in time
        Substituted::new(self, false)
    }
}

impl SubstTypes for Supertrait {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        let Supertrait { name, decl_ref } = self;
        decl_ref
            .recur_subst(engines, subst_list, kind)
            .map(|decl_ref| Supertrait { name, decl_ref })
    }
}

impl SubstTypes for TyFunctionDecl {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        let TyFunctionDecl {
            body,
            parameters,
            implementing_type,
            type_parameters,
            return_type,
            where_clause,
            // these fields contain no types
            name,
            span,
            attributes,
            visibility,
            is_contract_call,
            purity,
        } = self;
        body.recur_subst(engines, subst_list, kind)
            .and(parameters.recur_subst(engines, subst_list, kind))
            .and_two(implementing_type.recur_subst(engines, subst_list, kind))
            .and_three(type_parameters.recur_subst(engines, subst_list, kind))
            .and_four(return_type.recur_subst(engines, subst_list, kind))
            .and_five(
                where_clause
                    .into_iter()
                    .map(|(name, constraints)| {
                        constraints
                            .recur_subst(engines, subst_list, kind)
                            .map(|constraints| (name, constraints))
                    })
                    .collect(),
            )
            .and_map(
                |body,
                 parameters,
                 implementing_type,
                 type_parameters,
                 return_type,
                 where_clause| {
                    TyFunctionDecl {
                        body,
                        parameters,
                        implementing_type,
                        type_parameters,
                        return_type,
                        where_clause,
                        name,
                        span,
                        attributes,
                        visibility,
                        is_contract_call,
                        purity,
                    }
                },
            )
    }
}

impl SubstTypes for TyCodeBlock {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        let TyCodeBlock { contents } = self;
        contents
            .recur_subst(engines, subst_list, kind)
            .map(|contents| TyCodeBlock { contents })
    }
}

impl SubstTypes for TyAstNode {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        let TyAstNode { content, span } = self;
        content
            .recur_subst(engines, subst_list, kind)
            .map(|content| TyAstNode { content, span })
    }
}

impl SubstTypes for TyAstNodeContent {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        use TyAstNodeContent::*;
        match self {
            Declaration(decl) => decl.recur_subst(engines, subst_list, kind).map(Declaration),
            Expression(exp) => exp.recur_subst(engines, subst_list, kind).map(Expression),
            ImplicitReturnExpression(exp) => exp
                .recur_subst(engines, subst_list, kind)
                .map(ImplicitReturnExpression),
            n @ SideEffect(_) => Substituted::new(n, false),
        }
    }
}

impl SubstTypes for TyDecl {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        use TyDecl::*;
        match self {
            VariableDecl(var_decl) => var_decl
                .recur_subst(engines, subst_list, kind)
                .map(|var_decl| VariableDecl(Box::new(var_decl))),
            FunctionDecl {
                name,
                decl_id,
                subst_list: inner,
                decl_span,
            } => inner
                .unscoped_copy()
                .recur_subst(engines, subst_list, kind)
                .map(|inner| FunctionDecl {
                    name,
                    decl_id,
                    subst_list: Template::new(inner),
                    decl_span,
                }),
            TraitDecl {
                name,
                decl_id,
                subst_list: inner,
                decl_span,
            } => inner
                .unscoped_copy()
                .recur_subst(engines, subst_list, kind)
                .map(|inner| TraitDecl {
                    name,
                    decl_id,
                    subst_list: Template::new(inner),
                    decl_span,
                }),
            StructDecl {
                name,
                decl_id,
                subst_list: inner,
                decl_span,
            } => inner
                .unscoped_copy()
                .recur_subst(engines, subst_list, kind)
                .map(|inner| StructDecl {
                    name,
                    decl_id,
                    subst_list: Template::new(inner),
                    decl_span,
                }),
            EnumDecl {
                name,
                decl_id,
                subst_list: inner,
                decl_span,
            } => inner
                .unscoped_copy()
                .recur_subst(engines, subst_list, kind)
                .map(|inner| EnumDecl {
                    name,
                    decl_id,
                    subst_list: Template::new(inner),
                    decl_span,
                }),
            EnumVariantDecl {
                decl_id,
                subst_list: inner,
                variant_name,
                variant_decl_span,
            } => inner
                .unscoped_copy()
                .recur_subst(engines, subst_list, kind)
                .map(|inner| EnumVariantDecl {
                    decl_id,
                    subst_list: Template::new(inner),
                    variant_name,
                    variant_decl_span,
                }),
            ImplTrait {
                name,
                decl_id,
                subst_list: inner,
                decl_span,
            } => inner
                .unscoped_copy()
                .recur_subst(engines, subst_list, kind)
                .map(|inner| ImplTrait {
                    name,
                    decl_id,
                    subst_list: Template::new(inner),
                    decl_span,
                }),
            GenericTypeForFunctionScope { name, type_id } => type_id
                .recur_subst(engines, subst_list, kind)
                .map(|type_id| GenericTypeForFunctionScope { name, type_id }),
            TypeAliasDecl {
                name,
                decl_id,
                decl_span,
            } => todo!(),
            d @ ConstantDecl {
                name: _,
                decl_id: _,
                decl_span: _,
            } => Substituted::new(d, false),
            d @ AbiDecl {
                name: _,
                decl_id: _,
                decl_span: _,
            } => Substituted::new(d, false),
            d @ ErrorRecovery(_) => Substituted::new(d, false),
            d @ StorageDecl {
                decl_id: _,
                decl_span: _,
            } => Substituted::new(d, false),
        }
    }
}

impl SubstTypes for TyVariableDecl {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        let TyVariableDecl {
            body,
            return_type,
            type_ascription,
            // these fields contain no types
            name,
            mutability,
        } = self;
        body.recur_subst(engines, subst_list, kind)
            .and(return_type.recur_subst(engines, subst_list, kind))
            .and_two(type_ascription.recur_subst(engines, subst_list, kind))
            .and_map(|body, return_type, type_ascription| TyVariableDecl {
                body,
                return_type,
                type_ascription,
                name,
                mutability,
            })
    }
}

impl SubstTypes for TyStructField {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        let TyStructField {
            type_argument,
            // these fields contain no types
            name,
            span,
            attributes,
        } = self;
        type_argument
            .recur_subst(engines, subst_list, kind)
            .map(|type_argument| TyStructField {
                type_argument,
                name,
                span,
                attributes,
            })
    }
}

impl SubstTypes for TyExpression {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        let TyExpression {
            expression,
            return_type,
            // these fields contain no types
            span,
        } = self;
        expression
            .recur_subst(engines, subst_list, kind)
            .and(return_type.recur_subst(engines, subst_list, kind))
            .and_map(|expression, return_type| TyExpression {
                expression,
                return_type,
                span,
            })
    }
}

impl SubstTypes for TyExpressionVariant {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        use TyExpressionVariant::*;
        match self {
            FunctionApplication {
                contract_call_params,
                arguments,
                fn_ref,
                type_binding,
                // these fields contain no types
                call_path,
                self_state_idx,
                selector,
            } => contract_call_params
                .recur_subst(engines, subst_list, kind)
                .and(
                    arguments
                        .into_iter()
                        .map(|(name, exp)| {
                            exp.recur_subst(engines, subst_list, kind)
                                .map(|exp| (name, exp))
                        })
                        .collect(),
                )
                .and_two(fn_ref.recur_subst(engines, subst_list, kind))
                .and_three(type_binding.recur_subst(engines, subst_list, kind))
                .and_map(|contract_call_params, arguments, fn_ref, type_binding| {
                    FunctionApplication {
                        contract_call_params,
                        arguments,
                        fn_ref,
                        type_binding,
                        call_path,
                        self_state_idx,
                        selector,
                    }
                }),
            LazyOperator {
                lhs,
                rhs,
                // these fields contain no types
                op,
            } => lhs
                .recur_subst(engines, subst_list, kind)
                .and(rhs.recur_subst(engines, subst_list, kind))
                .and_map(|lhs, rhs| LazyOperator {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                    op,
                }),
            Tuple { fields } => fields
                .recur_subst(engines, subst_list, kind)
                .map(|fields| Tuple { fields }),
            Array {
                contents,
                elem_type,
            } => contents
                .recur_subst(engines, subst_list, kind)
                .and(elem_type.recur_subst(engines, subst_list, kind))
                .and_map(|contents, elem_type| Array {
                    contents,
                    elem_type,
                }),
            ArrayIndex { prefix, index } => prefix
                .recur_subst(engines, subst_list, kind)
                .and(index.recur_subst(engines, subst_list, kind))
                .and_map(|prefix, index| ArrayIndex {
                    prefix: Box::new(prefix),
                    index: Box::new(index),
                }),
            StructExpression {
                struct_ref,
                fields,
                call_path_binding,
                // these fields contain no types
                instantiation_span,
            } => struct_ref
                .recur_subst(engines, subst_list, kind)
                .and(fields.recur_subst(engines, subst_list, kind))
                .and_two(call_path_binding.recur_subst(engines, subst_list, kind))
                .and_map(|struct_ref, fields, call_path_binding| StructExpression {
                    struct_ref,
                    fields,
                    call_path_binding,
                    instantiation_span,
                }),
            CodeBlock(block) => block.recur_subst(engines, subst_list, kind).map(CodeBlock),
            MatchExp {
                desugared,
                scrutinees,
            } => desugared
                .recur_subst(engines, subst_list, kind)
                .and(scrutinees.recur_subst(engines, subst_list, kind))
                .and_map(|desugared, scrutinees| MatchExp {
                    desugared: Box::new(desugared),
                    scrutinees,
                }),
            IfExp {
                condition,
                then,
                r#else,
            } => condition
                .recur_subst(engines, subst_list, kind)
                .and(then.recur_subst(engines, subst_list, kind))
                .and_two(r#else.map_or_else(
                    || Substituted::new(None, false),
                    |r#else| {
                        r#else
                            .recur_subst(engines, subst_list, kind)
                            .map(|r#else| Some(Box::new(r#else)))
                    },
                ))
                .and_map(|condition, then, r#else| IfExp {
                    condition: Box::new(condition),
                    then: Box::new(then),
                    r#else,
                }),
            AsmExpression {
                registers,
                // these fields contain no types
                body,
                returns,
                whole_block_span,
            } => todo!(),
            StructFieldAccess {
                prefix,
                field_to_access,
                resolved_type_of_parent,
                field_instantiation_span,
            } => todo!(),
            TupleElemAccess {
                prefix,
                resolved_type_of_parent,
                // these fields contain no types
                elem_to_access_num,
                elem_to_access_span,
            } => prefix
                .recur_subst(engines, subst_list, kind)
                .and(resolved_type_of_parent.recur_subst(engines, subst_list, kind))
                .and_map(|prefix, resolved_type_of_parent| TupleElemAccess {
                    prefix: Box::new(prefix),
                    resolved_type_of_parent,
                    elem_to_access_num,
                    elem_to_access_span,
                }),
            EnumInstantiation {
                enum_ref,
                contents,
                call_path_binding,
                // these fields contain no types
                variant_instantiation_span,
                variant_name,
                tag,
            } => enum_ref
                .recur_subst(engines, subst_list, kind)
                .and(contents.map_or_else(
                    || Substituted::new(None, false),
                    |contents| {
                        contents
                            .recur_subst(engines, subst_list, kind)
                            .map(|contents| Some(Box::new(contents)))
                    },
                ))
                .and_two(call_path_binding.recur_subst(engines, subst_list, kind))
                .and_map(|enum_ref, contents, call_path_binding| EnumInstantiation {
                    enum_ref,
                    contents,
                    call_path_binding,
                    variant_instantiation_span,
                    variant_name,
                    tag,
                }),
            AbiCast {
                address,
                // these fields contain no types
                abi_name,
                span,
            } => address
                .recur_subst(engines, subst_list, kind)
                .map(|address| AbiCast {
                    address: Box::new(address),
                    abi_name,
                    span,
                }),
            StorageAccess(access) => access
                .recur_subst(engines, subst_list, kind)
                .map(StorageAccess),
            IntrinsicFunction(intrin) => intrin
                .recur_subst(engines, subst_list, kind)
                .map(IntrinsicFunction),
            AbiName(_) => todo!(),
            EnumTag { exp } => exp
                .recur_subst(engines, subst_list, kind)
                .map(|exp| EnumTag { exp: Box::new(exp) }),
            UnsafeDowncast { exp, variant } => exp
                .recur_subst(engines, subst_list, kind)
                .and(variant.recur_subst(engines, subst_list, kind))
                .and_map(|exp, variant| UnsafeDowncast {
                    exp: Box::new(exp),
                    variant,
                }),
            WhileLoop { condition, body } => condition
                .recur_subst(engines, subst_list, kind)
                .and(body.recur_subst(engines, subst_list, kind))
                .and_map(|condition, body| WhileLoop {
                    condition: Box::new(condition),
                    body,
                }),
            Reassignment(_) => todo!(),
            StorageReassignment(_) => todo!(),
            Return(exp) => exp
                .recur_subst(engines, subst_list, kind)
                .map(|exp| Return(Box::new(exp))),
            e @ Literal(_) => Substituted::new(e, false),
            e @ FunctionParameter | e @ Break | e @ Continue => Substituted::new(e, false),
            e @ VariableExpression {
                name: _,
                span: _,
                mutability: _,
                call_path: _,
            } => Substituted::new(e, false),
        }
    }
}

impl SubstTypes for TyStructExpressionField {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        let TyStructExpressionField {
            value,
            // these fields contain no types
            name,
        } = self;
        value
            .recur_subst(engines, subst_list, kind)
            .map(|value| TyStructExpressionField { value, name })
    }
}

impl SubstTypes for TyScrutinee {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        let TyScrutinee {
            variant,
            type_id,
            // these fields contain no types
            span,
        } = self;
        variant
            .recur_subst(engines, subst_list, kind)
            .and(type_id.recur_subst(engines, subst_list, kind))
            .and_map(|variant, type_id| TyScrutinee {
                variant,
                type_id,
                span,
            })
    }
}

impl SubstTypes for TyScrutineeVariant {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        use TyScrutineeVariant::*;
        match self {
            Constant(_, _, _) => todo!(),
            StructScrutinee {
                struct_ref,
                fields,
                // these fields contain no types
                instantiation_call_path,
            } => struct_ref
                .recur_subst(engines, subst_list, kind)
                .and(fields.recur_subst(engines, subst_list, kind))
                .and_map(|struct_ref, fields| StructScrutinee {
                    struct_ref,
                    fields,
                    instantiation_call_path,
                }),
            EnumScrutinee {
                enum_ref,
                variant,
                value,
                // these fields contain no types
                instantiation_call_path,
            } => enum_ref
                .recur_subst(engines, subst_list, kind)
                .and(variant.recur_subst(engines, subst_list, kind))
                .and_two(value.recur_subst(engines, subst_list, kind))
                .and_map(|enum_ref, variant, value| EnumScrutinee {
                    enum_ref,
                    variant: Box::new(variant),
                    value: Box::new(value),
                    instantiation_call_path,
                }),
            Tuple(_) => todo!(),
            s @ CatchAll | s @ Literal(_) | s @ Variable(_) => Substituted::new(s, false),
        }
    }
}

impl SubstTypes for TyStructScrutineeField {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        let TyStructScrutineeField {
            scrutinee,
            // these fields contain no types
            field,
            span,
            field_def_name,
        } = self;
        scrutinee
            .recur_subst(engines, subst_list, kind)
            .map(|scrutinee| TyStructScrutineeField {
                scrutinee,
                field,
                span,
                field_def_name,
            })
    }
}

impl SubstTypes for TyEnumVariant {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        let TyEnumVariant {
            type_argument,
            // these fields contain no types
            name,
            tag,
            span,
            attributes,
        } = self;
        type_argument
            .recur_subst(engines, subst_list, kind)
            .map(|type_argument| TyEnumVariant {
                type_argument,
                name,
                tag,
                span,
                attributes,
            })
    }
}

impl SubstTypes for TyFunctionParameter {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        let TyFunctionParameter {
            type_argument,
            // these fields contain no types
            name,
            is_reference,
            is_mutable,
            mutability_span,
        } = self;
        type_argument
            .recur_subst(engines, subst_list, kind)
            .map(|type_argument| TyFunctionParameter {
                name,
                is_reference,
                is_mutable,
                mutability_span,
                type_argument,
            })
    }
}

impl SubstTypes for TyIntrinsicFunctionKind {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        subst_kind: SubstitutionKind,
    ) -> Substituted<Self> {
        let TyIntrinsicFunctionKind {
            arguments,
            type_arguments,
            // these fields contain no types
            kind,
            span,
        } = self;
        arguments
            .subst_inner(engines, subst_list, subst_kind)
            .and(type_arguments.subst_inner(engines, subst_list, subst_kind))
            .and_map(|arguments, type_arguments| TyIntrinsicFunctionKind {
                arguments,
                type_arguments,
                kind,
                span,
            })
    }
}

impl SubstTypes for TyStorageAccess {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        let TyStorageAccess {
            fields,
            // these fields contain no types
            ix,
        } = self;
        fields
            .recur_subst(engines, subst_list, kind)
            .map(|fields| TyStorageAccess { fields, ix })
    }
}

impl SubstTypes for TyStorageAccessDescriptor {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        let TyStorageAccessDescriptor {
            type_id,
            // these fields contain no types
            name,
            span,
        } = self;
        type_id
            .recur_subst(engines, subst_list, kind)
            .map(|type_id| TyStorageAccessDescriptor {
                type_id,
                name,
                span,
            })
    }
}

impl SubstTypes for TypeArgument {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        let TypeArgument {
            type_id,
            // the types in these fields don't need to be substituted
            initial_type_id,
            // these fields contain no types
            span,
            call_path_tree,
        } = self;
        type_id
            .recur_subst(engines, subst_list, kind)
            .map(|type_id| TypeArgument {
                type_id,
                initial_type_id,
                span,
                call_path_tree,
            })
    }
}

impl SubstTypes for TypeParameter {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        let TypeParameter {
            type_id,
            trait_constraints,
            // the types in these fields don't need to be substituted
            initial_type_id,
            // these fields contain no types
            name_ident,
            trait_constraints_span,
        } = self;
        type_id
            .recur_subst(engines, subst_list, kind)
            .and(trait_constraints.recur_subst(engines, subst_list, kind))
            .and_map(|type_id, trait_constraints| TypeParameter {
                type_id,
                initial_type_id,
                name_ident,
                trait_constraints,
                trait_constraints_span,
            })
    }
}

impl SubstTypes for TraitConstraint {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        let TraitConstraint {
            type_arguments,
            // these fields contain no types
            trait_name,
        } = self;
        type_arguments
            .recur_subst(engines, subst_list, kind)
            .map(|type_arguments| TraitConstraint {
                trait_name,
                type_arguments,
            })
    }
}

impl<T> SubstTypes for TypeBinding<T> {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        let TypeBinding {
            type_arguments,
            // the types in these fields don't need to be substituted
            inner,
            // these fields contain no types
            span,
        } = self;
        type_arguments
            .recur_subst(engines, subst_list, kind)
            .map(|type_arguments| TypeBinding {
                type_arguments,
                inner,
                span,
            })
    }
}

impl SubstTypes for TypeArgs {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        use TypeArgs::*;
        match self {
            Regular(type_args) => type_args
                .recur_subst(engines, subst_list, kind)
                .map(Regular),
            Prefix(type_args) => type_args.recur_subst(engines, subst_list, kind).map(Prefix),
        }
    }
}

impl SubstTypes for SubstList {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        self.into_elems()
            .recur_subst(engines, subst_list, kind)
            .map(|elems| elems.into())
    }
}

impl SubstTypes for TypeId {
    fn subst_inner(
        self,
        engines: Engines<'_>,
        subst_list: &SubstList,
        kind: SubstitutionKind,
    ) -> Substituted<Self> {
        use TypeInfo::*;
        let type_engine = engines.te();
        let decl_engine = engines.de();
        let type_info = type_engine.get(self);
        let opt = match type_info {
            TypeParam { index, .. } => subst_list
                .index(index)
                .map(|type_param| Substituted::new(type_param.type_id, true)),
            Custom {
                call_path,
                type_arguments,
            } => Some(type_arguments.recur_subst(engines, subst_list, kind).map(
                |type_arguments| {
                    type_engine.insert(
                        decl_engine,
                        Custom {
                            call_path,
                            type_arguments,
                        },
                    )
                },
            )),
            UnknownGeneric {
                name,
                trait_constraints,
            } => Some(
                trait_constraints
                    .0
                    .recur_subst(engines, subst_list, kind)
                    .map(|trait_constraints| {
                        type_engine.insert(
                            decl_engine,
                            UnknownGeneric {
                                name,
                                trait_constraints: VecSet(trait_constraints),
                            },
                        )
                    }),
            ),
            Struct(struct_ref) => Some(
                struct_ref
                    .recur_subst(engines, subst_list, kind)
                    .map(|struct_ref| type_engine.insert(decl_engine, Struct(struct_ref))),
            ),
            Enum(enum_ref) => Some(
                enum_ref
                    .recur_subst(engines, subst_list, kind)
                    .map(|enum_ref| type_engine.insert(decl_engine, Enum(enum_ref))),
            ),
            Array(elem_ty, count) => Some(
                elem_ty
                    .recur_subst(engines, subst_list, kind)
                    .map(|elem_ty| type_engine.insert(decl_engine, Array(elem_ty, count))),
            ),
            Tuple(fields) => Some(
                fields
                    .recur_subst(engines, subst_list, kind)
                    .map(|fields| type_engine.insert(decl_engine, Tuple(fields))),
            ),
            Storage { fields } => Some(
                fields
                    .recur_subst(engines, subst_list, kind)
                    .map(|fields| type_engine.insert(decl_engine, Storage { fields })),
            ),
            Alias { name, ty } => Some(
                ty.recur_subst(engines, subst_list, kind)
                    .map(|ty| type_engine.insert(decl_engine, Alias { name, ty })),
            ),
            ContractCaller { abi_name, address } => address.map(|address| {
                address
                    .recur_subst(engines, subst_list, kind)
                    .map(|address| {
                        type_engine.insert(
                            decl_engine,
                            ContractCaller {
                                abi_name,
                                address: Some(Box::new(address)),
                            },
                        )
                    })
            }),
            Unknown | Placeholder(_) | Str(_) | UnsignedInteger(_) | Boolean | SelfType | B256
            | Numeric | Contract | ErrorRecovery | RawUntypedPtr | RawUntypedSlice => None,
        };
        match opt {
            Some(ret) if ret.marker() => ret,
            _ => Substituted::new(self, false),
        }
    }
}

use crate::{
    decl_engine::*,
    engine_threading::*,
    has_changes,
    language::{ty::*, Literal},
    semantic_analysis::{
        TypeCheckAnalysis, TypeCheckAnalysisContext, TypeCheckContext, TypeCheckFinalization,
        TypeCheckFinalizationContext,
    },
    transform::{AllowDeprecatedState, AttributeKind, AttributesMap},
    type_system::*,
    types::*,
};
use serde::{Deserialize, Serialize};
use std::{fmt, hash::Hasher};
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
    type_error::TypeError,
    warning::{CompileWarning, Warning},
};
use sway_types::{Span, Spanned};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TyExpression {
    pub expression: TyExpressionVariant,
    pub return_type: TypeId,
    pub span: Span,
}

impl EqWithEngines for TyExpression {}
impl PartialEqWithEngines for TyExpression {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        let type_engine = ctx.engines().te();
        self.expression.eq(&other.expression, ctx)
            && type_engine
                .get(self.return_type)
                .eq(&type_engine.get(other.return_type), ctx)
    }
}

impl HashWithEngines for TyExpression {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyExpression {
            expression,
            return_type,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
        } = self;
        let type_engine = engines.te();
        expression.hash(state, engines);
        type_engine.get(*return_type).hash(state, engines);
    }
}

impl SubstTypes for TyExpression {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        has_changes! {
            self.return_type.subst(ctx);
            self.expression.subst(ctx);
        }
    }
}

impl ReplaceDecls for TyExpression {
    fn replace_decls_inner(
        &mut self,
        decl_mapping: &DeclMapping,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
    ) -> Result<bool, ErrorEmitted> {
        self.expression.replace_decls(decl_mapping, handler, ctx)
    }
}

impl UpdateConstantExpression for TyExpression {
    fn update_constant_expression(&mut self, engines: &Engines, implementing_type: &TyDecl) {
        self.expression
            .update_constant_expression(engines, implementing_type)
    }
}

impl DisplayWithEngines for TyExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(
            f,
            "{} ({})",
            engines.help_out(&self.expression),
            engines.help_out(self.return_type)
        )
    }
}

impl DebugWithEngines for TyExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(
            f,
            "{:?} ({:?})",
            engines.help_out(&self.expression),
            engines.help_out(self.return_type)
        )
    }
}

impl TypeCheckAnalysis for TyExpression {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        match &self.expression {
            // Check literal "fits" into assigned typed.
            TyExpressionVariant::Literal(Literal::Numeric(literal_value)) => {
                let t = ctx.engines.te().get(self.return_type);
                if let TypeInfo::UnsignedInteger(bits) = &*t {
                    if bits.would_overflow(*literal_value) {
                        handler.emit_err(CompileError::TypeError(TypeError::LiteralOverflow {
                            expected: format!("{:?}", ctx.engines.help_out(t)),
                            span: self.span.clone(),
                        }));
                    }
                }
            }
            TyExpressionVariant::ArrayExplicit { .. } => {
                self.as_array_unify_elements(handler, ctx.engines);
            }
            _ => {}
        }
        self.expression.type_check_analyze(handler, ctx)
    }
}

impl TypeCheckFinalization for TyExpression {
    fn type_check_finalize(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        let res = self.expression.type_check_finalize(handler, ctx);
        if let TyExpressionVariant::FunctionApplication { fn_ref, .. } = &self.expression {
            let method = ctx.engines.de().get_function(fn_ref);
            self.return_type = method.return_type.type_id;
        }
        res
    }
}

impl CollectTypesMetadata for TyExpression {
    fn collect_types_metadata(
        &self,
        handler: &Handler,
        ctx: &mut CollectTypesMetadataContext,
    ) -> Result<Vec<TypeMetadata>, ErrorEmitted> {
        use TyExpressionVariant::*;
        let decl_engine = ctx.engines.de();
        let mut res = self.return_type.collect_types_metadata(handler, ctx)?;
        match &self.expression {
            FunctionApplication {
                arguments,
                fn_ref,
                call_path,
                type_binding,
                ..
            } => {
                for arg in arguments.iter() {
                    res.append(&mut arg.1.collect_types_metadata(handler, ctx)?);
                }
                let function_decl = decl_engine.get_function(fn_ref);

                ctx.call_site_push();
                for (idx, type_parameter) in function_decl.type_parameters.iter().enumerate() {
                    ctx.call_site_insert(type_parameter.type_id, call_path.span());

                    // Verify type arguments are concrete
                    res.extend(
                        type_parameter
                            .type_id
                            .collect_types_metadata(handler, ctx)?
                            .into_iter()
                            // try to use the caller span for better error messages
                            .map(|x| match x {
                                TypeMetadata::UnresolvedType(ident, original_span) => {
                                    let span = type_binding
                                        .as_ref()
                                        .and_then(|type_binding| {
                                            type_binding.type_arguments.as_slice().get(idx)
                                        })
                                        .map(|type_argument| Some(type_argument.span.clone()))
                                        .unwrap_or(original_span);
                                    TypeMetadata::UnresolvedType(ident, span)
                                }
                                x => x,
                            }),
                    );
                }

                for content in function_decl.body.contents.iter() {
                    res.append(&mut content.collect_types_metadata(handler, ctx)?);
                }
                ctx.call_site_pop();
            }
            Tuple { fields } => {
                for field in fields.iter() {
                    res.append(&mut field.collect_types_metadata(handler, ctx)?);
                }
            }
            AsmExpression { registers, .. } => {
                for register in registers.iter() {
                    if let Some(init) = register.initializer.as_ref() {
                        res.append(&mut init.collect_types_metadata(handler, ctx)?);
                    }
                }
            }
            StructExpression {
                fields,
                instantiation_span,
                struct_id,
                ..
            } => {
                let struct_decl = decl_engine.get_struct(struct_id);
                for type_parameter in &struct_decl.type_parameters {
                    ctx.call_site_insert(type_parameter.type_id, instantiation_span.clone());
                }
                if let TypeInfo::Struct(decl_ref) = &*ctx.engines.te().get(self.return_type) {
                    let decl = decl_engine.get_struct(decl_ref);
                    for type_parameter in &decl.type_parameters {
                        ctx.call_site_insert(type_parameter.type_id, instantiation_span.clone());
                    }
                }
                for field in fields.iter() {
                    res.append(&mut field.value.collect_types_metadata(handler, ctx)?);
                }
            }
            LazyOperator { lhs, rhs, .. } => {
                res.append(&mut lhs.collect_types_metadata(handler, ctx)?);
                res.append(&mut rhs.collect_types_metadata(handler, ctx)?);
            }
            ArrayExplicit {
                elem_type: _,
                contents,
            } => {
                for content in contents.iter() {
                    res.append(&mut content.collect_types_metadata(handler, ctx)?);
                }
            }
            ArrayRepeat {
                elem_type: _,
                value,
                length,
            } => {
                res.append(&mut value.collect_types_metadata(handler, ctx)?);
                res.append(&mut length.collect_types_metadata(handler, ctx)?);
            }
            ArrayIndex { prefix, index } => {
                res.append(&mut (**prefix).collect_types_metadata(handler, ctx)?);
                res.append(&mut (**index).collect_types_metadata(handler, ctx)?);
            }
            CodeBlock(block) => {
                for content in block.contents.iter() {
                    res.append(&mut content.collect_types_metadata(handler, ctx)?);
                }
            }
            MatchExp { desugared, .. } => {
                res.append(&mut desugared.collect_types_metadata(handler, ctx)?)
            }
            IfExp {
                condition,
                then,
                r#else,
            } => {
                res.append(&mut condition.collect_types_metadata(handler, ctx)?);
                res.append(&mut then.collect_types_metadata(handler, ctx)?);
                if let Some(r#else) = r#else {
                    res.append(&mut r#else.collect_types_metadata(handler, ctx)?);
                }
            }
            StructFieldAccess {
                prefix,
                resolved_type_of_parent,
                ..
            } => {
                res.append(&mut prefix.collect_types_metadata(handler, ctx)?);
                res.append(&mut resolved_type_of_parent.collect_types_metadata(handler, ctx)?);
            }
            TupleElemAccess {
                prefix,
                resolved_type_of_parent,
                ..
            } => {
                res.append(&mut prefix.collect_types_metadata(handler, ctx)?);
                res.append(&mut resolved_type_of_parent.collect_types_metadata(handler, ctx)?);
            }
            EnumInstantiation {
                enum_ref,
                contents,
                call_path_binding,
                ..
            } => {
                let enum_decl = decl_engine.get_enum(enum_ref);
                for type_param in enum_decl.type_parameters.iter() {
                    ctx.call_site_insert(type_param.type_id, call_path_binding.inner.suffix.span())
                }
                if let Some(contents) = contents {
                    res.append(&mut contents.collect_types_metadata(handler, ctx)?);
                }
                for variant in enum_decl.variants.iter() {
                    res.append(
                        &mut variant
                            .type_argument
                            .type_id
                            .collect_types_metadata(handler, ctx)?,
                    );
                }
                for type_param in enum_decl.type_parameters.iter() {
                    res.append(&mut type_param.type_id.collect_types_metadata(handler, ctx)?);
                }
            }
            AbiCast { address, .. } => {
                res.append(&mut address.collect_types_metadata(handler, ctx)?);
            }
            IntrinsicFunction(kind) => {
                res.append(&mut kind.collect_types_metadata(handler, ctx)?);
            }
            EnumTag { exp } => {
                res.append(&mut exp.collect_types_metadata(handler, ctx)?);
            }
            UnsafeDowncast {
                exp,
                variant,
                call_path_decl: _,
            } => {
                res.append(&mut exp.collect_types_metadata(handler, ctx)?);
                res.append(
                    &mut variant
                        .type_argument
                        .type_id
                        .collect_types_metadata(handler, ctx)?,
                );
            }
            WhileLoop { condition, body } => {
                res.append(&mut condition.collect_types_metadata(handler, ctx)?);
                for content in body.contents.iter() {
                    res.append(&mut content.collect_types_metadata(handler, ctx)?);
                }
            }
            ForLoop { desugared } => {
                res.append(&mut desugared.collect_types_metadata(handler, ctx)?);
            }
            ImplicitReturn(exp) | Return(exp) => {
                res.append(&mut exp.collect_types_metadata(handler, ctx)?)
            }
            Ref(exp) | Deref(exp) => res.append(&mut exp.collect_types_metadata(handler, ctx)?),
            // storage access can never be generic
            // variable expressions don't ever have return types themselves, they're stored in
            // `TyExpression::return_type`. Variable expressions are just names of variables.
            VariableExpression { .. }
            | ConstantExpression { .. }
            | ConfigurableExpression { .. }
            | ConstGenericExpression { .. }
            | StorageAccess { .. }
            | Literal(_)
            | AbiName(_)
            | Break
            | Continue
            | FunctionParameter => {}
            Reassignment(reassignment) => {
                res.append(&mut reassignment.rhs.collect_types_metadata(handler, ctx)?);
            }
        }
        Ok(res)
    }
}

impl MaterializeConstGenerics for TyExpression {
    fn materialize_const_generics(
        &mut self,
        engines: &Engines,
        handler: &Handler,
        name: &str,
        value: &TyExpression,
    ) -> Result<(), ErrorEmitted> {
        self.return_type
            .materialize_const_generics(engines, handler, name, value)?;
        match &mut self.expression {
            TyExpressionVariant::ConstGenericExpression { decl, .. } => {
                decl.materialize_const_generics(engines, handler, name, value)
            }
            TyExpressionVariant::ImplicitReturn(expr) => {
                expr.materialize_const_generics(engines, handler, name, value)
            }
            TyExpressionVariant::FunctionApplication { arguments, .. } => {
                for (_, expr) in arguments {
                    expr.materialize_const_generics(engines, handler, name, value)?;
                }

                Ok(())
            }
            TyExpressionVariant::WhileLoop { condition, body } => {
                condition.materialize_const_generics(engines, handler, name, value)?;
                body.materialize_const_generics(engines, handler, name, value)
            }
            TyExpressionVariant::Reassignment(expr) => expr
                .rhs
                .materialize_const_generics(engines, handler, name, value),
            TyExpressionVariant::ArrayIndex { prefix, index } => {
                prefix.materialize_const_generics(engines, handler, name, value)?;
                index.materialize_const_generics(engines, handler, name, value)
            }
            TyExpressionVariant::IntrinsicFunction(kind) => {
                for expr in kind.arguments.iter_mut() {
                    expr.materialize_const_generics(engines, handler, name, value)?;
                }
                Ok(())
            }
            TyExpressionVariant::Literal(_) | TyExpressionVariant::VariableExpression { .. } => {
                Ok(())
            }
            TyExpressionVariant::ArrayRepeat {
                elem_type,
                value: elem_value,
                length,
            } => {
                elem_type.materialize_const_generics(engines, handler, name, value)?;
                elem_value.materialize_const_generics(engines, handler, name, value)?;
                length.materialize_const_generics(engines, handler, name, value)
            }
            TyExpressionVariant::Ref(r) => {
                r.materialize_const_generics(engines, handler, name, value)
            }
            _ => Err(handler.emit_err(
                sway_error::error::CompileError::ConstGenericNotSupportedHere {
                    span: self.span.clone(),
                },
            )),
        }
    }
}

impl TyExpression {
    pub(crate) fn error(err: ErrorEmitted, span: Span, engines: &Engines) -> TyExpression {
        let type_engine = engines.te();
        TyExpression {
            expression: TyExpressionVariant::Tuple { fields: vec![] },
            return_type: type_engine.id_of_error_recovery(err),
            span,
        }
    }

    /// gathers the mutability of the expressions within
    pub(crate) fn gather_mutability(&self) -> VariableMutability {
        match &self.expression {
            TyExpressionVariant::VariableExpression { mutability, .. } => *mutability,
            _ => VariableMutability::Immutable,
        }
    }

    /// Returns `self` as a literal, if possible.
    pub(crate) fn extract_literal_value(&self) -> Option<Literal> {
        self.expression.extract_literal_value()
    }

    // Checks if this expression references a deprecated item
    // TODO: Change this fn for more deprecated checks.
    pub(crate) fn check_deprecated(
        &self,
        engines: &Engines,
        handler: &Handler,
        allow_deprecated: &mut AllowDeprecatedState,
    ) {
        fn emit_warning_if_deprecated(
            attributes: &AttributesMap,
            span: &Span,
            handler: &Handler,
            message: &str,
            allow_deprecated: &mut AllowDeprecatedState,
        ) {
            if allow_deprecated.is_allowed() {
                return;
            }

            if let Some(v) = attributes
                .get(&AttributeKind::Deprecated)
                .and_then(|x| x.last())
            {
                let mut message = message.to_string();

                if let Some(sway_ast::Literal::String(s)) = v
                    .args
                    .iter()
                    .find(|x| x.name.as_str() == "note")
                    .and_then(|x| x.value.as_ref())
                {
                    message.push_str(": ");
                    message.push_str(s.parsed.as_str());
                }

                handler.emit_warn(CompileWarning {
                    span: span.clone(),
                    warning_content: Warning::UsingDeprecated { message },
                })
            }
        }

        match &self.expression {
            TyExpressionVariant::Literal(..) => {}
            TyExpressionVariant::FunctionApplication {
                call_path,
                fn_ref,
                arguments,
                ..
            } => {
                for (_, expr) in arguments.iter() {
                    expr.check_deprecated(engines, handler, allow_deprecated);
                }

                let fn_ty = engines.de().get(fn_ref);
                if let Some(TyDecl::ImplSelfOrTrait(t)) = &fn_ty.implementing_type {
                    let t = &engines.de().get(&t.decl_id).implementing_for;
                    if let TypeInfo::Struct(struct_id) = &*engines.te().get(t.type_id) {
                        let s = engines.de().get(struct_id);
                        emit_warning_if_deprecated(
                            &s.attributes,
                            &call_path.span(),
                            handler,
                            "deprecated struct",
                            allow_deprecated,
                        );
                    }
                }

                emit_warning_if_deprecated(
                    &fn_ty.attributes,
                    &call_path.span(),
                    handler,
                    "deprecated function",
                    allow_deprecated,
                );
            }
            TyExpressionVariant::LazyOperator { lhs, rhs, .. } => {
                lhs.check_deprecated(engines, handler, allow_deprecated);
                rhs.check_deprecated(engines, handler, allow_deprecated);
            }
            TyExpressionVariant::ConstantExpression { span, decl, .. } => {
                emit_warning_if_deprecated(
                    &decl.attributes,
                    span,
                    handler,
                    "deprecated constant",
                    allow_deprecated,
                );
            }
            TyExpressionVariant::ConfigurableExpression { span, decl, .. } => {
                emit_warning_if_deprecated(
                    &decl.attributes,
                    span,
                    handler,
                    "deprecated configurable",
                    allow_deprecated,
                );
            }
            TyExpressionVariant::ConstGenericExpression { span, .. } => {
                // Const generics don´t have attributes,
                // so deprecation warnings cannot be turned off
                emit_warning_if_deprecated(
                    &AttributesMap::default(),
                    span,
                    handler,
                    "deprecated configurable",
                    allow_deprecated,
                );
            }
            TyExpressionVariant::VariableExpression { .. } => {}
            TyExpressionVariant::Tuple { fields } => {
                for e in fields.iter() {
                    e.check_deprecated(engines, handler, allow_deprecated);
                }
            }
            TyExpressionVariant::ArrayExplicit { contents, .. } => {
                for e in contents.iter() {
                    e.check_deprecated(engines, handler, allow_deprecated);
                }
            }
            TyExpressionVariant::ArrayRepeat { value, length, .. } => {
                value.check_deprecated(engines, handler, allow_deprecated);
                length.check_deprecated(engines, handler, allow_deprecated);
            }
            TyExpressionVariant::ArrayIndex { prefix, index } => {
                prefix.check_deprecated(engines, handler, allow_deprecated);
                index.check_deprecated(engines, handler, allow_deprecated);
            }
            TyExpressionVariant::StructExpression {
                struct_id,
                instantiation_span,
                ..
            } => {
                let struct_decl = engines.de().get(struct_id);
                emit_warning_if_deprecated(
                    &struct_decl.attributes,
                    instantiation_span,
                    handler,
                    "deprecated struct",
                    allow_deprecated,
                );
            }
            TyExpressionVariant::CodeBlock(block) => {
                block.check_deprecated(engines, handler, allow_deprecated);
            }
            TyExpressionVariant::FunctionParameter => {}
            TyExpressionVariant::MatchExp {
                desugared,
                //scrutinees,
                ..
            } => {
                desugared.check_deprecated(engines, handler, allow_deprecated);
                // TODO: check scrutinees if necessary
            }
            TyExpressionVariant::IfExp {
                condition,
                then,
                r#else,
            } => {
                condition.check_deprecated(engines, handler, allow_deprecated);
                then.check_deprecated(engines, handler, allow_deprecated);
                if let Some(e) = r#else {
                    e.check_deprecated(engines, handler, allow_deprecated);
                }
            }
            TyExpressionVariant::AsmExpression { .. } => {}
            TyExpressionVariant::StructFieldAccess {
                prefix,
                field_to_access,
                field_instantiation_span,
                ..
            } => {
                prefix.check_deprecated(engines, handler, allow_deprecated);
                emit_warning_if_deprecated(
                    &field_to_access.attributes,
                    field_instantiation_span,
                    handler,
                    "deprecated struct field",
                    allow_deprecated,
                );
            }
            TyExpressionVariant::TupleElemAccess { prefix, .. } => {
                prefix.check_deprecated(engines, handler, allow_deprecated);
            }
            TyExpressionVariant::EnumInstantiation {
                enum_ref,
                tag,
                contents,
                variant_instantiation_span,
                ..
            } => {
                let enum_ty = engines.de().get(enum_ref);
                emit_warning_if_deprecated(
                    &enum_ty.attributes,
                    variant_instantiation_span,
                    handler,
                    "deprecated enum",
                    allow_deprecated,
                );
                if let Some(variant_decl) = enum_ty.variants.get(*tag) {
                    emit_warning_if_deprecated(
                        &variant_decl.attributes,
                        variant_instantiation_span,
                        handler,
                        "deprecated enum variant",
                        allow_deprecated,
                    );
                }
                if let Some(expr) = contents {
                    expr.check_deprecated(engines, handler, allow_deprecated);
                }
            }
            TyExpressionVariant::AbiCast { address, .. } => {
                // TODO: check abi name?
                address.check_deprecated(engines, handler, allow_deprecated);
            }
            TyExpressionVariant::StorageAccess(access) => {
                // TODO: check storage access?
                if let Some(expr) = &access.key_expression {
                    expr.check_deprecated(engines, handler, allow_deprecated);
                }
            }
            TyExpressionVariant::IntrinsicFunction(kind) => {
                for arg in kind.arguments.iter() {
                    arg.check_deprecated(engines, handler, allow_deprecated);
                }
            }
            TyExpressionVariant::AbiName(..) => {}
            TyExpressionVariant::EnumTag { exp } => {
                exp.check_deprecated(engines, handler, allow_deprecated);
            }
            TyExpressionVariant::UnsafeDowncast {
                exp,
                //variant,
                ..
            } => {
                exp.check_deprecated(engines, handler, allow_deprecated);
                // TODO: maybe check variant?
            }
            TyExpressionVariant::WhileLoop { condition, body } => {
                condition.check_deprecated(engines, handler, allow_deprecated);
                body.check_deprecated(engines, handler, allow_deprecated);
            }
            TyExpressionVariant::ForLoop { desugared } => {
                desugared.check_deprecated(engines, handler, allow_deprecated);
            }
            TyExpressionVariant::Break => {}
            TyExpressionVariant::Continue => {}
            TyExpressionVariant::Reassignment(reass) => {
                if let TyReassignmentTarget::DerefAccess { exp, indices } = &reass.lhs {
                    exp.check_deprecated(engines, handler, allow_deprecated);
                    for indice in indices {
                        match indice {
                            ProjectionKind::StructField {
                                name: idx_name,
                                field_to_access,
                            } => {
                                if let Some(field_to_access) = field_to_access {
                                    emit_warning_if_deprecated(
                                        &field_to_access.attributes,
                                        &idx_name.span(),
                                        handler,
                                        "deprecated struct field",
                                        allow_deprecated,
                                    );
                                }
                            }
                            ProjectionKind::TupleField {
                                index: _,
                                index_span: _,
                            } => {}
                            ProjectionKind::ArrayIndex {
                                index,
                                index_span: _,
                            } => index.check_deprecated(engines, handler, allow_deprecated),
                        }
                    }
                }
                reass
                    .rhs
                    .check_deprecated(engines, handler, allow_deprecated);
            }
            TyExpressionVariant::ImplicitReturn(expr) => {
                expr.check_deprecated(engines, handler, allow_deprecated);
            }
            TyExpressionVariant::Return(expr) => {
                expr.check_deprecated(engines, handler, allow_deprecated);
            }
            TyExpressionVariant::Ref(expr) => {
                expr.check_deprecated(engines, handler, allow_deprecated);
            }
            TyExpressionVariant::Deref(expr) => {
                expr.check_deprecated(engines, handler, allow_deprecated);
            }
        }
    }

    pub fn as_array(&self) -> Option<(&TypeId, &[TyExpression])> {
        match &self.expression {
            TyExpressionVariant::ArrayExplicit {
                elem_type,
                contents,
            } => Some((elem_type, contents)),
            _ => None,
        }
    }

    pub fn as_intrinsic(&self) -> Option<&TyIntrinsicFunctionKind> {
        match &self.expression {
            TyExpressionVariant::IntrinsicFunction(v) => Some(v),
            _ => None,
        }
    }

    /// Unify elem_type with each element return type.
    /// Must be called on arrays.
    pub fn as_array_unify_elements(&self, handler: &Handler, engines: &Engines) {
        let TyExpressionVariant::ArrayExplicit {
            elem_type,
            contents,
        } = &self.expression
        else {
            unreachable!("Should only be called on Arrays")
        };

        let array_elem_type = engines.te().get(*elem_type);
        if !matches!(&*array_elem_type, TypeInfo::Never) {
            let unify = crate::type_system::unify::unifier::Unifier::new(
                engines,
                "",
                unify::unifier::UnifyKind::Default,
            );
            for element in contents {
                let element_type = engines.te().get(element.return_type);

                // If the element is never, we do not need to check
                if matches!(&*element_type, TypeInfo::Never) {
                    continue;
                }

                let h = Handler::default();
                unify.unify(&h, element.return_type, *elem_type, &element.span, true);

                // unification error points to type that failed
                // we want to report the element type instead
                if h.has_errors() {
                    handler.emit_err(CompileError::TypeError(TypeError::MismatchedType {
                        expected: format!("{:?}", engines.help_out(&array_elem_type)),
                        received: format!("{:?}", engines.help_out(element_type)),
                        help_text: String::new(),
                        span: element.span.clone(),
                    }));
                }
            }
        }
    }
}

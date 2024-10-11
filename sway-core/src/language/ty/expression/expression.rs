use std::{fmt, hash::Hasher};

use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
    type_error::TypeError,
    warning::{CompileWarning, Warning},
};
use sway_types::{Span, Spanned};

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

#[derive(Clone, Debug)]
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
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, ctx: &SubstTypesContext) -> HasChanges {
        has_changes! {
            self.return_type.subst(type_mapping, ctx);
            self.expression.subst(type_mapping, ctx);
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
            TyExpressionVariant::Array { .. } => {
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
            Array {
                elem_type: _,
                contents,
            } => {
                for content in contents.iter() {
                    res.append(&mut content.collect_types_metadata(handler, ctx)?);
                }
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

impl TyExpression {
    pub(crate) fn error(err: ErrorEmitted, span: Span, engines: &Engines) -> TyExpression {
        let type_engine = engines.te();
        TyExpression {
            expression: TyExpressionVariant::Tuple { fields: vec![] },
            return_type: type_engine.insert(engines, TypeInfo::ErrorRecovery(err), None),
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
            TyExpressionVariant::FunctionApplication {
                call_path, fn_ref, ..
            } => {
                if let Some(TyDecl::ImplSelfOrTrait(t)) =
                    &engines.de().get(fn_ref).implementing_type
                {
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
            }
            _ => {}
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
        let TyExpressionVariant::Array {
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

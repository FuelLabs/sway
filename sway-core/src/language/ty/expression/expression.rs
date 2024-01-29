use std::{
    collections::{HashMap, HashSet},
    fmt,
    hash::Hasher,
};

use sway_error::{
    handler::{ErrorEmitted, Handler},
    warning::{CompileWarning, Warning},
};
use sway_types::{Span, Spanned};

use crate::{
    decl_engine::*,
    engine_threading::*,
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
    fn eq(&self, other: &Self, engines: &Engines) -> bool {
        let type_engine = engines.te();
        self.expression.eq(&other.expression, engines)
            && type_engine
                .get(self.return_type)
                .eq(&type_engine.get(other.return_type), engines)
    }
}

impl HashWithEngines for TyExpression {
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
        engines: &Engines,
        already_hashed: &mut HashSet<(usize, std::any::TypeId)>,
    ) {
        let TyExpression {
            expression,
            return_type,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
        } = self;
        let type_engine = engines.te();
        expression.hash(state, engines, already_hashed);
        let key = (return_type.index(), std::any::TypeId::of::<TypeInfo>());
        if already_hashed.contains(&key) {
            return;
        }
        let mut already_hashed_updated = already_hashed.clone();
        already_hashed_updated.insert(key);
        type_engine
            .get(*return_type)
            .hash(state, engines, &mut already_hashed_updated);
    }
}

impl SubstTypes for TyExpression {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) {
        self.return_type.subst(type_mapping, engines);
        self.expression.subst(type_mapping, engines);
    }
}

impl ReplaceDecls for TyExpression {
    fn replace_decls_inner(
        &mut self,
        decl_mapping: &DeclMapping,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
        already_replaced: &mut HashMap<(usize, std::any::TypeId), (usize, Span)>,
    ) -> Result<(), ErrorEmitted> {
        self.expression
            .replace_decls(decl_mapping, handler, ctx, already_replaced)
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
        already_collected: &mut HashSet<(usize, std::any::TypeId)>,
    ) -> Result<Vec<TypeMetadata>, ErrorEmitted> {
        use TyExpressionVariant::*;
        let decl_engine = ctx.engines.de();
        let mut res = self
            .return_type
            .collect_types_metadata(handler, ctx, already_collected)?;
        match &self.expression {
            FunctionApplication {
                arguments,
                fn_ref,
                call_path,
                ..
            } => {
                for arg in arguments.iter() {
                    res.append(&mut arg.1.collect_types_metadata(
                        handler,
                        ctx,
                        already_collected,
                    )?);
                }
                let key = (
                    fn_ref.id().inner(),
                    std::any::TypeId::of::<TyFunctionDecl>(),
                );
                if !already_collected.contains(&key) {
                    let mut already_collected_updated = already_collected.clone();
                    already_collected_updated.insert(key);

                    let function_decl = decl_engine.get_function(fn_ref);

                    ctx.call_site_push();
                    for type_parameter in &function_decl.type_parameters {
                        ctx.call_site_insert(type_parameter.type_id, call_path.span())
                    }

                    for content in function_decl.body.contents.iter() {
                        res.append(&mut content.collect_types_metadata(
                            handler,
                            ctx,
                            &mut already_collected_updated,
                        )?);
                    }
                    ctx.call_site_pop();
                }
            }
            Tuple { fields } => {
                for field in fields.iter() {
                    res.append(&mut field.collect_types_metadata(
                        handler,
                        ctx,
                        already_collected,
                    )?);
                }
            }
            AsmExpression { registers, .. } => {
                for register in registers.iter() {
                    if let Some(init) = register.initializer.as_ref() {
                        res.append(&mut init.collect_types_metadata(
                            handler,
                            ctx,
                            already_collected,
                        )?);
                    }
                }
            }
            StructExpression {
                fields,
                instantiation_span,
                struct_ref,
                ..
            } => {
                let struct_decl = decl_engine.get_struct(struct_ref);
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
                    res.append(&mut field.value.collect_types_metadata(
                        handler,
                        ctx,
                        already_collected,
                    )?);
                }
            }
            LazyOperator { lhs, rhs, .. } => {
                res.append(&mut lhs.collect_types_metadata(handler, ctx, already_collected)?);
                res.append(&mut rhs.collect_types_metadata(handler, ctx, already_collected)?);
            }
            Array {
                elem_type: _,
                contents,
            } => {
                for content in contents.iter() {
                    res.append(&mut content.collect_types_metadata(
                        handler,
                        ctx,
                        already_collected,
                    )?);
                }
            }
            ArrayIndex { prefix, index } => {
                res.append(&mut (**prefix).collect_types_metadata(
                    handler,
                    ctx,
                    already_collected,
                )?);
                res.append(&mut (**index).collect_types_metadata(
                    handler,
                    ctx,
                    already_collected,
                )?);
            }
            CodeBlock(block) => {
                for content in block.contents.iter() {
                    res.append(&mut content.collect_types_metadata(
                        handler,
                        ctx,
                        already_collected,
                    )?);
                }
            }
            MatchExp { desugared, .. } => res.append(&mut desugared.collect_types_metadata(
                handler,
                ctx,
                already_collected,
            )?),
            IfExp {
                condition,
                then,
                r#else,
            } => {
                res.append(&mut condition.collect_types_metadata(
                    handler,
                    ctx,
                    already_collected,
                )?);
                res.append(&mut then.collect_types_metadata(handler, ctx, already_collected)?);
                if let Some(r#else) = r#else {
                    res.append(&mut r#else.collect_types_metadata(
                        handler,
                        ctx,
                        already_collected,
                    )?);
                }
            }
            StructFieldAccess {
                prefix,
                resolved_type_of_parent,
                ..
            } => {
                res.append(&mut prefix.collect_types_metadata(handler, ctx, already_collected)?);
                res.append(&mut resolved_type_of_parent.collect_types_metadata(
                    handler,
                    ctx,
                    already_collected,
                )?);
            }
            TupleElemAccess {
                prefix,
                resolved_type_of_parent,
                ..
            } => {
                res.append(&mut prefix.collect_types_metadata(handler, ctx, already_collected)?);
                res.append(&mut resolved_type_of_parent.collect_types_metadata(
                    handler,
                    ctx,
                    already_collected,
                )?);
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
                    res.append(&mut contents.collect_types_metadata(
                        handler,
                        ctx,
                        already_collected,
                    )?);
                }
                for variant in enum_decl.variants.iter() {
                    res.append(&mut variant.type_argument.type_id.collect_types_metadata(
                        handler,
                        ctx,
                        already_collected,
                    )?);
                }
                for type_param in enum_decl.type_parameters.iter() {
                    res.append(&mut type_param.type_id.collect_types_metadata(
                        handler,
                        ctx,
                        already_collected,
                    )?);
                }
            }
            AbiCast { address, .. } => {
                res.append(&mut address.collect_types_metadata(handler, ctx, already_collected)?);
            }
            IntrinsicFunction(kind) => {
                res.append(&mut kind.collect_types_metadata(handler, ctx, already_collected)?);
            }
            EnumTag { exp } => {
                res.append(&mut exp.collect_types_metadata(handler, ctx, already_collected)?);
            }
            UnsafeDowncast {
                exp,
                variant,
                call_path_decl: _,
            } => {
                res.append(&mut exp.collect_types_metadata(handler, ctx, already_collected)?);
                res.append(&mut variant.type_argument.type_id.collect_types_metadata(
                    handler,
                    ctx,
                    already_collected,
                )?);
            }
            WhileLoop { condition, body } => {
                res.append(&mut condition.collect_types_metadata(
                    handler,
                    ctx,
                    already_collected,
                )?);
                for content in body.contents.iter() {
                    res.append(&mut content.collect_types_metadata(
                        handler,
                        ctx,
                        already_collected,
                    )?);
                }
            }
            Return(exp) => {
                res.append(&mut exp.collect_types_metadata(handler, ctx, already_collected)?)
            }
            Ref(exp) | Deref(exp) => {
                res.append(&mut exp.collect_types_metadata(handler, ctx, already_collected)?)
            }
            // storage access can never be generic
            // variable expressions don't ever have return types themselves, they're stored in
            // `TyExpression::return_type`. Variable expressions are just names of variables.
            VariableExpression { .. }
            | ConstantExpression { .. }
            | StorageAccess { .. }
            | Literal(_)
            | AbiName(_)
            | Break
            | Continue
            | FunctionParameter => {}
            Reassignment(reassignment) => {
                res.append(&mut reassignment.rhs.collect_types_metadata(
                    handler,
                    ctx,
                    already_collected,
                )?);
            }
        }
        Ok(res)
    }
}

impl DeterministicallyAborts for TyExpression {
    fn deterministically_aborts(&self, decl_engine: &DeclEngine, check_call_body: bool) -> bool {
        use TyExpressionVariant::*;
        match &self.expression {
            FunctionApplication {
                fn_ref, arguments, ..
            } => {
                if !check_call_body {
                    return false;
                }
                let function_decl = decl_engine.get_function(fn_ref);
                function_decl
                    .body
                    .deterministically_aborts(decl_engine, check_call_body)
                    || arguments
                        .iter()
                        .any(|(_, x)| x.deterministically_aborts(decl_engine, check_call_body))
            }
            Tuple { fields, .. } => fields
                .iter()
                .any(|x| x.deterministically_aborts(decl_engine, check_call_body)),
            Array { contents, .. } => contents
                .iter()
                .any(|x| x.deterministically_aborts(decl_engine, check_call_body)),
            CodeBlock(contents) => contents.deterministically_aborts(decl_engine, check_call_body),
            LazyOperator { lhs, .. } => lhs.deterministically_aborts(decl_engine, check_call_body),
            StructExpression { fields, .. } => fields.iter().any(|x| {
                x.value
                    .deterministically_aborts(decl_engine, check_call_body)
            }),
            EnumInstantiation { contents, .. } => contents
                .as_ref()
                .map(|x| x.deterministically_aborts(decl_engine, check_call_body))
                .unwrap_or(false),
            AbiCast { address, .. } => {
                address.deterministically_aborts(decl_engine, check_call_body)
            }
            StructFieldAccess { .. }
            | Literal(_)
            | StorageAccess { .. }
            | VariableExpression { .. }
            | ConstantExpression { .. }
            | FunctionParameter
            | TupleElemAccess { .. } => false,
            IntrinsicFunction(kind) => kind.deterministically_aborts(decl_engine, check_call_body),
            ArrayIndex { prefix, index } => {
                prefix.deterministically_aborts(decl_engine, check_call_body)
                    || index.deterministically_aborts(decl_engine, check_call_body)
            }
            AsmExpression { registers, .. } => registers.iter().any(|x| {
                x.initializer
                    .as_ref()
                    .map(|x| x.deterministically_aborts(decl_engine, check_call_body))
                    .unwrap_or(false)
            }),
            MatchExp { desugared, .. } => {
                desugared.deterministically_aborts(decl_engine, check_call_body)
            }
            IfExp {
                condition,
                then,
                r#else,
                ..
            } => {
                condition.deterministically_aborts(decl_engine, check_call_body)
                    || (then.deterministically_aborts(decl_engine, check_call_body)
                        && r#else
                            .as_ref()
                            .map(|x| x.deterministically_aborts(decl_engine, check_call_body))
                            .unwrap_or(false))
            }
            AbiName(_) => false,
            EnumTag { exp } => exp.deterministically_aborts(decl_engine, check_call_body),
            UnsafeDowncast { exp, .. } => {
                exp.deterministically_aborts(decl_engine, check_call_body)
            }
            WhileLoop { condition, body } => {
                condition.deterministically_aborts(decl_engine, check_call_body)
                    || body.deterministically_aborts(decl_engine, check_call_body)
            }
            Break => false,
            Continue => false,
            Reassignment(reassignment) => reassignment
                .rhs
                .deterministically_aborts(decl_engine, check_call_body),
            // TODO: Is this correct?
            // I'm not sure what this function is supposed to do exactly. It's called
            // "deterministically_aborts" which I thought meant it checks for an abort/panic, but
            // it's actually checking for returns.
            //
            // Also, is it necessary to check the expression to see if avoids the return? eg.
            // someone could write `return break;` in a loop, which would mean the return never
            // gets executed.
            Return(_) => true,
            Ref(exp) | Deref(exp) => exp.deterministically_aborts(decl_engine, check_call_body),
        }
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

    /// recurse into `self` and get any return statements -- used to validate that all returns
    /// do indeed return the correct type
    /// This does _not_ extract implicit return statements as those are not control flow! This is
    /// _only_ for explicit returns.
    pub(crate) fn gather_return_statements(&self) -> Vec<&TyExpression> {
        self.expression.gather_return_statements()
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
                struct_ref,
                instantiation_span,
                ..
            } => {
                let s = engines.de().get(struct_ref.id());
                emit_warning_if_deprecated(
                    &s.attributes,
                    instantiation_span,
                    handler,
                    "deprecated struct",
                    allow_deprecated,
                );
            }
            TyExpressionVariant::FunctionApplication {
                call_path, fn_ref, ..
            } => {
                if let Some(TyDecl::ImplTrait(t)) = &engines.de().get(fn_ref).implementing_type {
                    let t = &engines.de().get(&t.decl_id).implementing_for;
                    if let TypeInfo::Struct(struct_ref) = &*engines.te().get(t.type_id) {
                        let s = engines.de().get(struct_ref.id());
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
}

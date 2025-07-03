use crate::{engine_threading::*, has_changes, language::ty::*, type_system::*, types::*};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    hash::{Hash, Hasher},
};
use sway_ast::Intrinsic;
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::Span;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TyIntrinsicFunctionKind {
    pub kind: Intrinsic,
    pub arguments: Vec<TyExpression>,
    pub type_arguments: Vec<GenericArgument>,
    pub span: Span,
}

impl EqWithEngines for TyIntrinsicFunctionKind {}
impl PartialEqWithEngines for TyIntrinsicFunctionKind {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.kind == other.kind
            && self.arguments.eq(&other.arguments, ctx)
            && self.type_arguments.eq(&other.type_arguments, ctx)
    }
}

impl HashWithEngines for TyIntrinsicFunctionKind {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyIntrinsicFunctionKind {
            kind,
            arguments,
            type_arguments,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
        } = self;
        kind.hash(state);
        arguments.hash(state, engines);
        type_arguments.hash(state, engines);
    }
}

impl SubstTypes for TyIntrinsicFunctionKind {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        has_changes! {
            self.arguments.subst(ctx);
            self.type_arguments.subst(ctx);
        }
    }
}

impl DebugWithEngines for TyIntrinsicFunctionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        let targs = self
            .type_arguments
            .iter()
            .map(|targ| format!("{:?}", engines.help_out(targ.type_id())))
            .join(", ");
        let args = self
            .arguments
            .iter()
            .map(|e| format!("{:?}", engines.help_out(e)))
            .join(", ");

        write!(f, "{}::<{}>::({})", self.kind, targs, args)
    }
}

impl CollectTypesMetadata for TyIntrinsicFunctionKind {
    fn collect_types_metadata(
        &self,
        handler: &Handler,
        ctx: &mut CollectTypesMetadataContext,
    ) -> Result<Vec<TypeMetadata>, ErrorEmitted> {
        let mut types_metadata = vec![];
        for type_arg in self.type_arguments.iter() {
            types_metadata.append(&mut type_arg.type_id().collect_types_metadata(handler, ctx)?);
        }
        for arg in self.arguments.iter() {
            types_metadata.append(&mut arg.collect_types_metadata(handler, ctx)?);
        }

        match self.kind {
            Intrinsic::Log => {
                let logged_type_id = TypeMetadata::get_logged_type_id(
                    &self.arguments[0],
                    ctx.experimental.new_encoding,
                )
                .map_err(|err| handler.emit_err(err))?;
                let logged_type = TypeMetadata::new_logged_type(
                    ctx.engines,
                    logged_type_id,
                    ctx.program_name.clone(),
                );
                types_metadata.push(logged_type);
            }
            Intrinsic::Smo => {
                types_metadata.push(TypeMetadata::MessageType(
                    MessageId::new(ctx.message_id_counter()),
                    self.arguments[1].return_type,
                ));
                *ctx.message_id_counter_mut() += 1;
            }
            _ => {}
        }

        Ok(types_metadata)
    }
}

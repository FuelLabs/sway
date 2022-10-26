use std::fmt;

use crate::{error::*, language::ty::*, type_system::*, types::DeterministicallyAborts};
use itertools::Itertools;
use sway_ast::Intrinsic;
use sway_types::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TyIntrinsicFunctionKind {
    pub kind: Intrinsic,
    pub arguments: Vec<TyExpression>,
    pub type_arguments: Vec<TypeArgument>,
    pub span: Span,
}

impl CopyTypes for TyIntrinsicFunctionKind {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping) {
        for arg in &mut self.arguments {
            arg.copy_types(type_mapping);
        }
        for targ in &mut self.type_arguments {
            targ.type_id.copy_types(type_mapping);
        }
    }
}

impl ReplaceSelfType for TyIntrinsicFunctionKind {
    fn replace_self_type(&mut self, self_type: TypeId) {
        for arg in &mut self.arguments {
            arg.replace_self_type(self_type);
        }
        for targ in &mut self.type_arguments {
            targ.type_id.replace_self_type(self_type);
        }
    }
}

impl fmt::Display for TyIntrinsicFunctionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let targs = self
            .type_arguments
            .iter()
            .map(|targ| look_up_type_id(targ.type_id))
            .join(", ");
        let args = self.arguments.iter().map(|e| format!("{}", e)).join(", ");

        write!(f, "{}::<{}>::({})", self.kind, targs, args)
    }
}

impl DeterministicallyAborts for TyIntrinsicFunctionKind {
    fn deterministically_aborts(&self) -> bool {
        matches!(self.kind, Intrinsic::Revert)
            || self.arguments.iter().any(|x| x.deterministically_aborts())
    }
}

impl CollectTypesMetadata for TyIntrinsicFunctionKind {
    fn collect_types_metadata(
        &self,
        ctx: &mut CollectTypesMetadataContext,
    ) -> CompileResult<Vec<TypeMetadata>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut types_metadata = vec![];
        for type_arg in self.type_arguments.iter() {
            types_metadata.append(&mut check!(
                type_arg.type_id.collect_types_metadata(ctx),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }
        for arg in self.arguments.iter() {
            types_metadata.append(&mut check!(
                arg.collect_types_metadata(ctx),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }

        if matches!(self.kind, Intrinsic::Log) {
            types_metadata.push(TypeMetadata::LoggedType(
                LogId::new(ctx.log_id_counter()),
                self.arguments[0].return_type,
            ));
            *ctx.log_id_counter_mut() += 1;
        }

        ok(types_metadata, warnings, errors)
    }
}

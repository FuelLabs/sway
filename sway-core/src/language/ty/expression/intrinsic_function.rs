use std::fmt;

use sway_ast::Intrinsic;
use sway_types::Span;

use crate::{error::*, type_system::*, types::DeterministicallyAborts};

use super::TyExpression;

#[derive(Debug, Clone, PartialEq)]
pub struct TyIntrinsicFunctionKind {
    pub kind: Intrinsic,
    pub arguments: Vec<TyExpression>,
    pub type_arguments: Vec<TypeArgument>,
    pub span: Span,
}

impl CopyTypes for TyIntrinsicFunctionKind {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        for arg in &mut self.arguments {
            arg.copy_types(type_mapping);
        }
        for targ in &mut self.type_arguments {
            targ.type_id.copy_types(type_mapping);
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
    fn collect_types_metadata(&self) -> CompileResult<Vec<TypeMetadata>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut types_metadata = vec![];
        for type_arg in self.type_arguments.iter() {
            types_metadata.append(&mut check!(
                type_arg.type_id.collect_types_metadata(),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }
        for arg in self.arguments.iter() {
            types_metadata.append(&mut check!(
                arg.collect_types_metadata(),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }

        if matches!(self.kind, Intrinsic::Log) {
            types_metadata.push(TypeMetadata::LoggedType(self.arguments[0].return_type));
        }

        ok(types_metadata, warnings, errors)
    }
}

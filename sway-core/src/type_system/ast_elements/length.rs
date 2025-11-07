use crate::{
    ast_elements::type_parameter::ConstGenericExprTyDecl, decl_engine::DeclEngineGet as _, Engines,
};

use super::type_parameter::ConstGenericExpr;

/// Describes a fixed length for types that need it, e.g., [crate::TypeInfo::Array].
///
/// Optionally, if the length is coming from a literal in code, the [Length]
/// also keeps the [Span] of that literal. In that case, we say that the length
/// is annotated.
///
/// E.g., in this example, the two lengths coming from the literal `3` will
/// have two different spans pointing to the two different strings "3":
///
/// ```ignore
/// fn copy(a: [u64;3], b: [u64;3])
/// ```
#[derive(Debug, Clone)]
pub struct Length(pub ConstGenericExpr);

impl Length {
    pub fn expr(&self) -> &ConstGenericExpr {
        &self.0
    }

    pub fn extract_literal(&self, engines: &Engines) -> Option<u64> {
        match &self.0 {
            ConstGenericExpr::Literal { val, .. } => Some(*val as u64),
            ConstGenericExpr::AmbiguousVariableExpression { decl, .. } => {
                match decl.as_ref()? {
                    ConstGenericExprTyDecl::ConstGenericDecl(decl) => {
                        let decl = engines.de().get(&decl.decl_id);
                        let expr = decl.value.as_ref()?;
                        let expr = expr.expression.as_literal()?;
                        expr.cast_value_to_u64()
                    }
                    ConstGenericExprTyDecl::ConstantDecl(_) => None,
                }
            }
        }
    }
}

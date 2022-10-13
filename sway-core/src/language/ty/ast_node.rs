use std::fmt;

use derivative::Derivative;
use sway_types::Span;

use crate::{error::*, language::ty::*, type_system::*, types::DeterministicallyAborts};

#[derive(Clone, Debug, Eq, Derivative)]
#[derivative(PartialEq)]
pub struct TyAstNode {
    pub content: TyAstNodeContent,
    #[derivative(PartialEq = "ignore")]
    pub(crate) span: Span,
}

impl fmt::Display for TyAstNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TyAstNodeContent::*;
        let text = match &self.content {
            Declaration(ref typed_decl) => typed_decl.to_string(),
            Expression(exp) => exp.to_string(),
            ImplicitReturnExpression(exp) => format!("return {}", exp),
            SideEffect => "".into(),
        };
        f.write_str(&text)
    }
}

impl CopyTypes for TyAstNode {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        match self.content {
            TyAstNodeContent::ImplicitReturnExpression(ref mut exp) => exp.copy_types(type_mapping),
            TyAstNodeContent::Declaration(ref mut decl) => decl.copy_types(type_mapping),
            TyAstNodeContent::Expression(ref mut expr) => expr.copy_types(type_mapping),
            TyAstNodeContent::SideEffect => (),
        }
    }
}

impl CollectTypesMetadata for TyAstNode {
    fn collect_types_metadata(&self) -> CompileResult<Vec<TypeMetadata>> {
        self.content.collect_types_metadata()
    }
}

impl DeterministicallyAborts for TyAstNode {
    fn deterministically_aborts(&self) -> bool {
        use TyAstNodeContent::*;
        match &self.content {
            Declaration(_) => false,
            Expression(exp) | ImplicitReturnExpression(exp) => exp.deterministically_aborts(),
            SideEffect => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TyAstNodeContent {
    Declaration(TyDeclaration),
    Expression(TyExpression),
    ImplicitReturnExpression(TyExpression),
    // a no-op node used for something that just issues a side effect, like an import statement.
    SideEffect,
}

impl CollectTypesMetadata for TyAstNodeContent {
    fn collect_types_metadata(&self) -> CompileResult<Vec<TypeMetadata>> {
        use TyAstNodeContent::*;
        match self {
            Declaration(decl) => decl.collect_types_metadata(),
            Expression(expr) => expr.collect_types_metadata(),
            ImplicitReturnExpression(expr) => expr.collect_types_metadata(),
            SideEffect => ok(vec![], vec![], vec![]),
        }
    }
}

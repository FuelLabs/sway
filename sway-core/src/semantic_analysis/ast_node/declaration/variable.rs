use crate::{
    declaration_engine::declaration_engine::DeclarationEngine,
    semantic_analysis::*,
    type_system::*,
    types::{CompileWrapper, ToCompileWrapper},
    Ident, Visibility,
};
use sway_types::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VariableMutability {
    // private + mutable
    Mutable,
    // private + referenceable + mutable
    RefMutable,
    // private + immutable
    Immutable,
    // public + immutable
    ExportedConst,
    // public + mutable is invalid
}

impl Default for VariableMutability {
    fn default() -> Self {
        VariableMutability::Immutable
    }
}
impl VariableMutability {
    pub fn is_mutable(&self) -> bool {
        matches!(
            self,
            VariableMutability::Mutable | VariableMutability::RefMutable
        )
    }
    pub fn visibility(&self) -> Visibility {
        match self {
            VariableMutability::ExportedConst => Visibility::Public,
            _ => Visibility::Private,
        }
    }
    pub fn is_immutable(&self) -> bool {
        !self.is_mutable()
    }
}

pub fn convert_to_variable_immutability(
    is_reference: bool,
    is_mutable: bool,
) -> VariableMutability {
    if is_reference {
        VariableMutability::RefMutable
    } else if is_mutable {
        VariableMutability::Mutable
    } else {
        VariableMutability::Immutable
    }
}

#[derive(Clone, Debug)]
pub struct TypedVariableDeclaration {
    pub name: Ident,
    pub body: TypedExpression,
    pub mutability: VariableMutability,
    pub type_ascription: TypeId,
    pub type_ascription_span: Option<Span>,
}

impl PartialEq for CompileWrapper<'_, TypedVariableDeclaration> {
    fn eq(&self, other: &Self) -> bool {
        let CompileWrapper {
            inner: me,
            declaration_engine: de,
        } = self;
        let CompileWrapper { inner: them, .. } = other;
        me.name == them.name
            && me.body.wrap_ref(de) == them.body.wrap_ref(de)
            && me.mutability == them.mutability
            && look_up_type_id(me.type_ascription).wrap_ref(de)
                == look_up_type_id(them.type_ascription).wrap_ref(de)
    }
}

impl CopyTypes for TypedVariableDeclaration {
    fn copy_types(&mut self, type_mapping: &TypeMapping, de: &DeclarationEngine) {
        self.type_ascription
            .update_type(type_mapping, de, &self.body.span);
        self.body.copy_types(type_mapping, de)
    }
}

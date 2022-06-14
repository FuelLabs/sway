use sway_types::Spanned;

use crate::{
    constants::*,
    error::*,
    namespace::{Path, Root},
    semantic_analysis::*,
    type_engine::*,
    Ident, TypeArgument, Visibility,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VariableMutability {
    // private + mutable
    Mutable,
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
        matches!(self, VariableMutability::Mutable)
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

impl From<bool> for VariableMutability {
    fn from(o: bool) -> Self {
        if o {
            VariableMutability::Mutable
        } else {
            VariableMutability::Immutable
        }
    }
}
// as a bool, true means mutable
impl From<VariableMutability> for bool {
    fn from(o: VariableMutability) -> bool {
        o.is_mutable()
    }
}
#[derive(Clone, Debug, Eq)]
pub struct TypedVariableDeclaration {
    pub name: Ident,
    pub body: TypedExpression,
    pub(crate) is_mutable: VariableMutability,
    pub type_ascription: TypeId,
    pub(crate) const_decl_origin: bool,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypedVariableDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.body == other.body
            && self.is_mutable == other.is_mutable
            && look_up_type_id(self.type_ascription) == look_up_type_id(other.type_ascription)
            && self.const_decl_origin == other.const_decl_origin
    }
}

impl CopyTypes for TypedVariableDeclaration {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.type_ascription
            .update_type(type_mapping, &self.body.span);
        self.body.copy_types(type_mapping)
    }
}

impl ResolveTypes for TypedVariableDeclaration {
    fn resolve_type_with_self(
        &mut self,
        _type_arguments: Vec<TypeArgument>,
        enforce_type_arguments: EnforceTypeArguments,
        self_type: TypeId,
        namespace: &mut Root,
        module_path: &Path,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        self.type_ascription = check!(
            namespace.resolve_type_with_self(
                self.type_ascription,
                self_type,
                &self.name.span(),
                enforce_type_arguments,
                module_path,
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors
        );
        self.body
            .resolve_type_with_self(
                vec![],
                enforce_type_arguments,
                self_type,
                namespace,
                module_path,
            )
            .ok(&mut warnings, &mut errors);
        ok((), warnings, errors)
    }

    fn resolve_type_without_self(
        &mut self,
        _type_arguments: Vec<TypeArgument>,
        namespace: &mut Root,
        module_path: &Path,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        self.type_ascription = check!(
            namespace.resolve_type_without_self(self.type_ascription, module_path,),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors
        );
        self.body
            .resolve_type_without_self(vec![], namespace, module_path)
            .ok(&mut warnings, &mut errors);
        ok((), warnings, errors)
    }
}

// there are probably more names we should check here, this is the only one that will result in an
// actual issue right now, though
pub fn check_if_name_is_invalid(name: &Ident) -> CompileResult<()> {
    INVALID_NAMES
        .iter()
        .find_map(|x| {
            if *x == name.as_str() {
                Some(err(
                    vec![],
                    [CompileError::InvalidVariableName { name: name.clone() }].to_vec(),
                ))
            } else {
                None
            }
        })
        .unwrap_or_else(|| ok((), vec![], vec![]))
}

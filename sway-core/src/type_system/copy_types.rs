use crate::declaration_engine::declaration_engine::DeclarationEngine;

use super::TypeMapping;

pub(crate) trait CopyTypes {
    fn copy_types(&mut self, type_mapping: &TypeMapping, de: &DeclarationEngine);
}

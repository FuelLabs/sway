use crate::TypeEngine;

use super::TypeMapping;

pub(crate) trait CopyTypes {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, type_engine: &TypeEngine);

    fn copy_types(&mut self, type_mapping: &TypeMapping, type_engine: &TypeEngine) {
        if !type_mapping.is_empty() {
            self.copy_types_inner(type_mapping, type_engine);
        }
    }
}

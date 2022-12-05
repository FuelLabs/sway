use crate::engine_threading::Engines;

use super::TypeMapping;

pub(crate) trait CopyTypes {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, engines: Engines<'_>);

    fn copy_types(&mut self, type_mapping: &TypeMapping, engines: Engines<'_>) {
        if !type_mapping.is_empty() {
            self.copy_types_inner(type_mapping, engines);
        }
    }
}

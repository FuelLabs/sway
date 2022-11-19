use crate::TypeEngine;

use super::DeclMapping;

pub(crate) trait ReplaceDecls {
    fn replace_decls_inner(&mut self, decl_mapping: &DeclMapping, type_engine: &TypeEngine);

    fn replace_decls(&mut self, decl_mapping: &DeclMapping, type_engine: &TypeEngine) {
        if !decl_mapping.is_empty() {
            self.replace_decls_inner(decl_mapping, type_engine);
        }
    }
}

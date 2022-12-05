use crate::engine_threading::Engines;

use super::DeclMapping;

pub(crate) trait ReplaceDecls {
    fn replace_decls_inner(&mut self, decl_mapping: &DeclMapping, engines: Engines<'_>);

    fn replace_decls(&mut self, decl_mapping: &DeclMapping, engines: Engines<'_>) {
        if !decl_mapping.is_empty() {
            self.replace_decls_inner(decl_mapping, engines);
        }
    }
}

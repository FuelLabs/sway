use super::DeclMapping;

pub(crate) trait ReplaceDecls {
    fn replace_decls_inner(&mut self, decl_mapping: &DeclMapping);

    fn replace_decls(&mut self, decl_mapping: &DeclMapping) {
        if !decl_mapping.is_empty() {
            self.replace_decls_inner(decl_mapping);
        }
    }
}

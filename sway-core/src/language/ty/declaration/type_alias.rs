use sway_types::{Ident, Named, Span, Spanned};

use crate::{engine_threading::*, language::Visibility, transform, type_system::*};

#[derive(Clone, Debug)]
pub struct TyTypeAliasDecl {
    pub name: Ident,
    pub attributes: transform::AttributesMap,
    pub ty: TypeArgument,
    pub visibility: Visibility,
    pub span: Span,
}

impl Named for TyTypeAliasDecl {
    fn name(&self) -> &Ident {
        &self.name
    }
}

impl CreateTypeId for TyTypeAliasDecl {
    fn create_type_id(&self, engines: Engines<'_>) -> TypeId {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        type_engine.insert(
            decl_engine,
            TypeInfo::Alias {
                name: self.name.clone(),
                ty: self.ty.clone(),
            },
        )
    }
}

impl Spanned for TyTypeAliasDecl {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

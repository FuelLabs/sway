use crate::{
    language::ty::{self, TyAstNode},
    Engines,
};
use sway_types::{Named, Spanned};

#[derive(Default)]
pub struct MarkerTraitsAutoImplInfo {}

pub type MarkerTraitsAutoImplContext<'a, 'b> =
    super::AutoImplContext<'a, 'b, MarkerTraitsAutoImplInfo>;

impl<'a, 'b> MarkerTraitsAutoImplContext<'a, 'b>
where
    'a: 'b,
{
    /// Generates an implementation of the `Enum` marker trait for the user defined enum
    /// represented by the `enum_decl`.
    pub fn generate_enum_marker_trait_impl(
        &mut self,
        engines: &Engines,
        enum_decl: &ty::TyEnumDecl,
    ) -> Option<TyAstNode> {
        self.auto_impl_empty_marker_trait_on_enum(engines, enum_decl, "Enum")
    }

    /// Generates an implementation of the `Error` marker trait for the user defined enum
    /// represented by the `enum_decl`.
    pub fn generate_error_type_marker_trait_impl_for_enum(
        &mut self,
        engines: &Engines,
        enum_decl: &ty::TyEnumDecl,
    ) -> Option<TyAstNode> {
        self.auto_impl_empty_marker_trait_on_enum(engines, enum_decl, "Error")
    }

    fn auto_impl_empty_marker_trait_on_enum(
        &mut self,
        engines: &Engines,
        enum_decl: &ty::TyEnumDecl,
        marker_trait_name: &str,
    ) -> Option<TyAstNode> {
        if self.ctx.namespace.current_module().is_std_marker_module() {
            return None;
        }

        let impl_enum_code = format!(
            "#[allow(dead_code, deprecated)] impl {marker_trait_name} for {} {{ }}",
            enum_decl.name()
        );

        let impl_enum_node = self.parse_impl_trait_to_ty_ast_node(
            engines,
            enum_decl.span().source_id(),
            &impl_enum_code,
            crate::build_config::DbgGeneration::None,
        );

        impl_enum_node.ok()
    }
}

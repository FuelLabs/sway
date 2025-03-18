use crate::{
    decl_engine::DeclEngineGet,
    language::ty::{self, TyAstNode, TyDecl},
    Engines,
};
use sway_error::handler::Handler;
use sway_types::{Named, Spanned};

#[derive(Default)]
pub struct MarkerTraitsAutoImplInfo {}

pub type MarkerTraitsAutoImplContext<'a, 'b> =
    super::AutoImplContext<'a, 'b, MarkerTraitsAutoImplInfo>;

impl<'a, 'b> MarkerTraitsAutoImplContext<'a, 'b>
where
    'a: 'b,
{
    /// Generates and implementation of the `Enum` marker trait for the user defined enum
    /// represented by the `enum_decl`.
    pub fn generate_enum_marker_trait_impl(
        &mut self,
        engines: &Engines,
        enum_decl: &ty::TyDecl,
    ) -> Option<TyAstNode> {
        match enum_decl {
            TyDecl::EnumDecl(_) => self.auto_impl_enum_marker_trait(engines, enum_decl),
            _ => None,
        }
    }

    fn auto_impl_enum_marker_trait(
        &mut self,
        engines: &Engines,
        enum_decl: &TyDecl,
    ) -> Option<TyAstNode> {
        if self.ctx.namespace.current_module().is_std_marker_module() {
            return None;
        }

        let enum_decl_id = enum_decl.to_enum_id(&Handler::default(), engines).unwrap();
        let enum_decl = self.ctx.engines().de().get(&enum_decl_id);

        let impl_enum_code = format!(
            "#[allow(dead_code)] impl Enum for {} {{ }}",
            enum_decl.name()
        );

        let impl_enum_node = self.parse_impl_trait_to_ty_ast_node(
            engines,
            enum_decl.span().source_id().cloned(),
            &impl_enum_code,
            crate::build_config::DbgGeneration::None,
        );

        impl_enum_node.ok()
    }
}

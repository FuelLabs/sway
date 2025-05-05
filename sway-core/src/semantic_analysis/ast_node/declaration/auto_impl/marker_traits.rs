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
        self.auto_impl_empty_marker_trait_on_enum(engines, enum_decl, "Enum", None)
    }

    /// Generates an implementation of the `Error` marker trait for the user defined enum
    /// represented by the `enum_decl`.
    pub fn generate_error_type_marker_trait_impl_for_enum(
        &mut self,
        engines: &Engines,
        enum_decl: &ty::TyEnumDecl,
    ) -> Option<TyAstNode> {
        self.auto_impl_empty_marker_trait_on_enum(engines, enum_decl, "Error", Some("AbiEncode"))
    }

    fn auto_impl_empty_marker_trait_on_enum(
        &mut self,
        engines: &Engines,
        enum_decl: &ty::TyEnumDecl,
        marker_trait_name: &str,
        extra_constraint: Option<&str>,
    ) -> Option<TyAstNode> {
        if self.ctx.namespace.current_module().is_std_marker_module() {
            return None;
        }

        let type_parameters_declaration =
            self.generate_type_parameters_declaration_code(&enum_decl.generic_parameters);
        let type_parameters_constraints = self.generate_type_parameters_constraints_code(
            &enum_decl.generic_parameters,
            extra_constraint,
        );

        let impl_marker_trait_code = format!(
            "#[allow(dead_code, deprecated)] impl{type_parameters_declaration} {marker_trait_name} for {}{type_parameters_declaration}{type_parameters_constraints} {{ }}",
            enum_decl.name().as_raw_ident_str()
        );

        let impl_enum_node = self.parse_impl_trait_to_ty_ast_node(
            engines,
            enum_decl.span().source_id(),
            &impl_marker_trait_code,
            crate::build_config::DbgGeneration::None,
        );

        impl_enum_node.ok()
    }
}

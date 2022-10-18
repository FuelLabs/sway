use crate::descriptor::{Descriptor, DescriptorType};
use anyhow::Result;
use sway_core::{
    language::{
        ty::{TyAstNodeContent, TySubmodule},
        {parsed::ParseProgram, ty::TyProgram},
    },
    CompileResult,
};

pub(crate) type Documentation = Vec<Document>;
/// A finalized Document ready to be rendered. We want to retain all
/// information including spans, fields on structs, variants on enums etc.
pub(crate) struct Document {
    pub(crate) module_prefix: Vec<String>,
    pub(crate) desc_ty: DescriptorType,
}
impl Document {
    // Creates an HTML file name from the [Document].
    pub fn file_name(&self) -> String {
        use DescriptorType::*;
        let name = match &self.desc_ty {
            Struct(ty_struct_decl) => Some(ty_struct_decl.name.as_str()),
            Enum(ty_enum_decl) => Some(ty_enum_decl.name.as_str()),
            Trait(ty_trait_decl) => Some(ty_trait_decl.name.as_str()),
            Abi(ty_abi_decl) => Some(ty_abi_decl.name.as_str()),
            Storage(_) => None, // storage does not have an Ident
            ImplTraitDesc(ty_impl_trait) => Some(ty_impl_trait.trait_name.suffix.as_str()), // TODO: check validity
            Function(ty_fn_decl) => Some(ty_fn_decl.name.as_str()),
            Const(ty_const_decl) => Some(ty_const_decl.name.as_str()),
        };

        Document::create_html_file_name(self.desc_ty.to_path_name(), name)
    }
    fn create_html_file_name(ty: &str, name: Option<&str>) -> String {
        match name {
            Some(name) => {
                format!("{ty}.{name}.html")
            }
            None => {
                format!("{ty}.html") // storage does not have an Ident
            }
        }
    }
    /// Gather [Documentation] from the [CompileResult].
    pub(crate) fn from_ty_program(
        compilation: &CompileResult<(ParseProgram, Option<TyProgram>)>,
        no_deps: bool,
    ) -> Result<Documentation> {
        let mut docs: Documentation = Default::default();
        if let Some((_, Some(typed_program))) = &compilation.value {
            for ast_node in &typed_program.root.all_nodes {
                if let TyAstNodeContent::Declaration(ref decl) = ast_node.content {
                    let desc = Descriptor::from_typed_decl(decl, vec![])?;

                    if let Descriptor::Documentable {
                        module_prefix,
                        desc_ty,
                    } = desc
                    {
                        docs.push(Document {
                            module_prefix,
                            desc_ty: *desc_ty,
                        })
                    }
                }
            }

            if !no_deps && !typed_program.root.submodules.is_empty() {
                // this is the same process as before but for dependencies
                for (_, ref typed_submodule) in &typed_program.root.submodules {
                    let module_prefix = vec![];
                    Document::from_ty_submodule(typed_submodule, &mut docs, &module_prefix)?;
                }
            }
        }

        Ok(docs)
    }
    fn from_ty_submodule(
        typed_submodule: &TySubmodule,
        docs: &mut Documentation,
        module_prefix: &[String],
    ) -> Result<()> {
        let mut new_submodule_prefix = module_prefix.to_owned();
        new_submodule_prefix.push(typed_submodule.library_name.as_str().to_string());
        for ast_node in &typed_submodule.module.all_nodes {
            if let TyAstNodeContent::Declaration(ref decl) = ast_node.content {
                let desc = Descriptor::from_typed_decl(decl, new_submodule_prefix.clone())?;

                if let Descriptor::Documentable {
                    module_prefix,
                    desc_ty,
                } = desc
                {
                    docs.push(Document {
                        module_prefix,
                        desc_ty: *desc_ty,
                    })
                }
            }
        }
        // if there is another submodule we need to go a level deeper
        if let Some((_, submodule)) = typed_submodule.module.submodules.first() {
            Document::from_ty_submodule(submodule, docs, &new_submodule_prefix)?;
        }

        Ok(())
    }
}

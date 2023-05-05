use crate::{
    core::{session::Session, token::get_range_from_span},
    utils::document::get_url_from_span,
};
use std::sync::Arc;
use sway_core::{language::CallPath, Engines, TypeId, TypeInfo};

use sway_types::{Span, Spanned};
use tower_lsp::lsp_types::{Range, Url};

#[derive(Debug, Clone)]
pub struct RelatedType {
    pub name: String,
    pub uri: Url,
    pub range: Range,
    pub callpath: CallPath,
}

#[derive(Debug, Clone)]
pub struct HoverLinkContents<'a> {
    pub related_types: Vec<RelatedType>,
    pub implementations: Vec<Span>,
    session: Arc<Session>,
    engines: Engines<'a>,
}

impl<'a> HoverLinkContents<'a> {
    pub fn new(session: Arc<Session>, engines: Engines<'a>) -> Self {
        Self {
            related_types: Vec::new(),
            implementations: Vec::new(),
            session,
            engines,
        }
    }

    /// Adds the given type and any related type parameters to the list of related types.
    pub fn add_related_types(&mut self, type_id: &TypeId) {
        let type_info = self.engines.te().get(*type_id);
        match type_info {
            TypeInfo::Enum(decl_ref) => {
                let decl = self.engines.de().get_enum(&decl_ref);
                self.add_related_type(decl_ref.name().to_string(), &decl.span(), decl.call_path);
                decl.type_parameters
                    .iter()
                    .for_each(|type_param| self.add_related_types(&type_param.type_id));
            }
            TypeInfo::Struct(decl_ref) => {
                let decl = self.engines.de().get_struct(&decl_ref);
                self.add_related_type(decl_ref.name().to_string(), &decl.span(), decl.call_path);
                decl.type_parameters
                    .iter()
                    .for_each(|type_param| self.add_related_types(&type_param.type_id));
            }
            _ => {}
        }
    }

    /// Adds a single type to the list of related types.
    fn add_related_type(&mut self, name: String, span: &Span, callpath: CallPath) {
        if let Ok(mut uri) = get_url_from_span(span) {
            let converted_url = self.session.sync.temp_to_workspace_url(&uri);
            if let Ok(url) = converted_url {
                uri = url;
            }
            let range = get_range_from_span(span);
            self.related_types.push(RelatedType {
                name,
                uri,
                range,
                callpath,
            });
        };
    }
}

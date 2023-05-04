use crate::{
    core::{session::Session, token::get_range_from_span},
    utils::document::get_url_from_span,
};
use std::sync::Arc;
use sway_core::{
    language::{
        ty::{TyDecl, TyTraitDecl},
        CallPath,
    },
    Engines, TypeId, TypeInfo,
};

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

    /// Adds all implementations of the given [TyTraitDecl] to the list of implementations.
    pub fn add_implementations_for_trait(&mut self, trait_decl: &TyTraitDecl) {
        if let Some(namespace) = self.session.namespace() {
            let call_path = CallPath::from(trait_decl.name.clone()); //.to_fullpath(&namespace);
            let impl_spans = namespace.get_impl_spans_for_trait_name(&call_path);
            self.add_implementations(&trait_decl.span(), impl_spans);
        }
    }

    /// Adds implementations of the given type to the list of implementations using the [TyDecl].
    pub fn add_implementations_for_decl(&mut self, ty_decl: &TyDecl) {
        if let Some(namespace) = self.session.namespace() {
            let impl_spans = namespace.get_impl_spans_for_decl(self.engines, ty_decl);
            self.add_implementations(&ty_decl.span(), impl_spans);
        }
    }

    /// Adds implementations of the given type to the list of implementations using the [TypeId].
    pub fn add_implementations_for_type(&mut self, decl_span: &Span, type_id: &TypeId) {
        if let Some(namespace) = self.session.namespace() {
            let impl_spans = namespace.get_impl_spans_for_type(self.engines, type_id);
            self.add_implementations(decl_span, impl_spans);
        }
    }

    /// Adds implementations to the list of implementation spans, with the declaration span first.
    /// Ensure that all paths are converted to workspace paths before adding them.
    fn add_implementations(&mut self, decl_span: &Span, mut impl_spans: Vec<Span>) {
        let mut all_spans = vec![decl_span.clone()];
        all_spans.append(&mut impl_spans);
        all_spans.dedup();
        all_spans.iter().for_each(|span| {
            let span_result = self.session.sync.temp_to_workspace_span(span);
            if let Ok(span) = span_result {
                self.implementations.push(span);
            }
        });
    }
}

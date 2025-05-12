use crate::{
    core::{session::Session, sync::SyncWorkspace, token::get_range_from_span},
    utils::document::get_url_from_span,
};
use std::sync::Arc;
use sway_core::{
    engine_threading::SpannedWithEngines,
    language::{
        ty::{TyDecl, TyTraitDecl},
        CallPath,
    },
    namespace::TraitMap,
    Engines, TypeId, TypeInfo,
};

use lsp_types::{Range, Url};
use sway_types::{Named, Span, Spanned};

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
    engines: &'a Engines,
}

impl<'a> HoverLinkContents<'a> {
    pub fn new(session: Arc<Session>, engines: &'a Engines) -> Self {
        Self {
            related_types: Vec::new(),
            implementations: Vec::new(),
            session,
            engines,
        }
    }

    /// Adds the given type and any related type parameters to the list of related types.
    pub fn add_related_types(&mut self, type_id: &TypeId, sync: &SyncWorkspace) {
        let type_info = self.engines.te().get(*type_id);
        match &*type_info {
            TypeInfo::Enum(decl_id) => {
                let decl = self.engines.de().get_enum(decl_id);
                self.add_related_type(
                    decl.name().to_string(),
                    &decl.span(),
                    decl.call_path.clone(),
                    sync,
                );
                decl.generic_parameters
                    .iter()
                    .filter_map(|x| x.as_type_parameter())
                    .for_each(|type_param| self.add_related_types(&type_param.type_id, sync));
            }
            TypeInfo::Struct(decl_id) => {
                let decl = self.engines.de().get_struct(decl_id);
                self.add_related_type(
                    decl.name().to_string(),
                    &decl.span(),
                    decl.call_path.clone(),
                    sync,
                );
                decl.generic_parameters
                    .iter()
                    .filter_map(|x| x.as_type_parameter())
                    .for_each(|type_param| self.add_related_types(&type_param.type_id, sync));
            }
            _ => {}
        }
    }

    /// Adds a single type to the list of related types.
    fn add_related_type(&mut self, name: String, span: &Span, callpath: CallPath, sync: &SyncWorkspace) {
        if let Ok(mut uri) = get_url_from_span(self.engines.se(), span) {
            let converted_url = sync.temp_to_workspace_url(&uri);
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

    /// Adds all implementations of the given [`TyTraitDecl`] to the list of implementations.
    pub fn add_implementations_for_trait(&mut self, trait_decl: &TyTraitDecl, sync: &SyncWorkspace) {
        if let Some(namespace) = self.session.namespace() {
            let call_path =
                CallPath::from(trait_decl.name.clone()).to_fullpath(self.engines, &namespace);
            let impl_spans =
                TraitMap::get_impl_spans_for_trait_name(namespace.current_module(), &call_path);
            self.add_implementations(&trait_decl.span(), impl_spans, sync);
        }
    }

    /// Adds implementations of the given type to the list of implementations using the [`TyDecl`].
    pub fn add_implementations_for_decl(&mut self, ty_decl: &TyDecl, sync: &SyncWorkspace) {
        if let Some(namespace) = self.session.namespace() {
            let impl_spans = TraitMap::get_impl_spans_for_decl(
                namespace.current_module(),
                self.engines,
                ty_decl,
            );
            self.add_implementations(&ty_decl.span(self.engines), impl_spans, sync);
        }
    }

    /// Adds implementations of the given type to the list of implementations using the [`TypeId`].
    pub fn add_implementations_for_type(&mut self, decl_span: &Span, type_id: TypeId, sync: &SyncWorkspace) {
        if let Some(namespace) = self.session.namespace() {
            let impl_spans = TraitMap::get_impl_spans_for_type(
                namespace.current_module(),
                self.engines,
                &type_id,
            );
            self.add_implementations(decl_span, impl_spans, sync);
        }
    }

    /// Adds implementations to the list of implementation spans, with the declaration span first.
    /// Ensure that all paths are converted to workspace paths before adding them.
    fn add_implementations(&mut self, decl_span: &Span, mut impl_spans: Vec<Span>, sync: &SyncWorkspace) {
        let mut all_spans = vec![decl_span.clone()];
        all_spans.append(&mut impl_spans);
        all_spans.dedup();
        for span in &all_spans {
            let span_result = sync.temp_to_workspace_span(self.engines.se(), span);
            if let Ok(span) = span_result {
                self.implementations.push(span);
            }
        }
    }
}

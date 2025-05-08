use crate::{
    decl_engine::DeclId,
    language::{ty::TyFunctionDecl, CallPath, Inline, Purity},
};

use sway_ir::{Context, MetadataIndex, Metadatum, Value};
use sway_types::{span::Source, Ident, SourceId, Span, Spanned};

use std::{collections::HashMap, path::PathBuf, sync::Arc};

/// IR metadata needs to be consistent between IR generation (converting [Span]s, etc. to metadata)
/// and ASM generation (converting the metadata back again).  Here we consolidate all of
/// `sway-core`s metadata needs into a single place to enable that consistency.
///
/// The [MetadataManager] also does its best to reduce redundancy by caching certain common
/// elements, such as source paths and storage attributes, and to avoid recreating the same
/// indices repeatedly.

#[derive(Default)]
pub(crate) struct MetadataManager {
    // We want to be able to store more then one `Span` per `MetadataIndex`.
    // E.g., storing the span of the function name, and the whole function declaration.
    // The spans differ then by the tag property of their `Metadatum::Struct`.
    // We could cache all such spans in a single `HashMap` where the key would be (Span, tag).
    // But since the vast majority of stored spans will be tagged with the generic "span" tag,
    // and only a few elements will have additional spans in their `MetadataIndex`, it is
    // more efficient to provide two separate caches, one for the spans tagged with "span",
    // and one for all other spans, tagged with arbitrary tags.
    /// Holds [Span]s tagged with "span".
    md_span_cache: HashMap<MetadataIndex, Span>,
    /// Holds [Span]s tagged with an arbitrary tag.
    md_tagged_span_cache: HashMap<MetadataIndex, (Span, &'static str)>,
    md_file_loc_cache: HashMap<MetadataIndex, (Arc<PathBuf>, Source)>,
    md_purity_cache: HashMap<MetadataIndex, Purity>,
    md_inline_cache: HashMap<MetadataIndex, Inline>,
    md_test_decl_index_cache: HashMap<MetadataIndex, DeclId<TyFunctionDecl>>,

    span_md_cache: HashMap<Span, MetadataIndex>,
    tagged_span_md_cache: HashMap<(Span, &'static str), MetadataIndex>,
    file_loc_md_cache: HashMap<SourceId, MetadataIndex>,
    purity_md_cache: HashMap<Purity, MetadataIndex>,
    inline_md_cache: HashMap<Inline, MetadataIndex>,
    test_decl_index_md_cache: HashMap<DeclId<TyFunctionDecl>, MetadataIndex>,
}

impl MetadataManager {
    pub(crate) fn md_to_span(
        &mut self,
        context: &Context,
        md_idx: Option<MetadataIndex>,
    ) -> Option<Span> {
        Self::for_each_md_idx(context, md_idx, |md_idx| {
            self.md_span_cache.get(&md_idx).cloned().or_else(|| {
                md_idx
                    .get_content(context)
                    .unwrap_struct("span", 3)
                    .and_then(|fields| {
                        // Create a new span and save it in the cache.
                        let span = self.create_span_from_metadatum_fields(context, fields)?;
                        self.md_span_cache.insert(md_idx, span.clone());
                        Some(span)
                    })
            })
        })
    }

    /// Returns the [Span] tagged with `tag` from the `md_idx`,
    /// or `None` if such span does not exist.
    /// If there are more spans tagged with `tag` inside of the
    /// `md_idx`, the first one will be returned.
    pub(crate) fn md_to_tagged_span(
        &mut self,
        context: &Context,
        md_idx: Option<MetadataIndex>,
        tag: &'static str,
    ) -> Option<Span> {
        Self::for_each_md_idx(context, md_idx, |md_idx| {
            let fields = md_idx.get_content(context).unwrap_struct(tag, 3);

            match fields {
                Some(fields) => self
                    .md_tagged_span_cache
                    .get(&md_idx)
                    .map(|span_and_tag| span_and_tag.0.clone())
                    .or_else(|| {
                        // Create a new span and save it in the cache.
                        let span = self.create_span_from_metadatum_fields(context, fields)?;
                        self.md_tagged_span_cache
                            .insert(md_idx, (span.clone(), tag));
                        Some(span)
                    }),
                None => None,
            }
        })
    }

    /// Returns the [Span] pointing to the function name in the function declaration from the `md_idx`,
    /// or `None` if such span does not exist.
    pub(crate) fn md_to_fn_name_span(
        &mut self,
        context: &Context,
        md_idx: Option<MetadataIndex>,
    ) -> Option<Span> {
        self.md_to_tagged_span(context, md_idx, "fn_name_span")
    }

    /// Returns the [Span] pointing to the call path in the function call from the `md_idx`,
    /// or `None` if such span does not exist.
    pub(crate) fn md_to_fn_call_path_span(
        &mut self,
        context: &Context,
        md_idx: Option<MetadataIndex>,
    ) -> Option<Span> {
        self.md_to_tagged_span(context, md_idx, "fn_call_path_span")
    }

    pub(crate) fn md_to_test_decl_index(
        &mut self,
        context: &Context,
        md_idx: Option<MetadataIndex>,
    ) -> Option<DeclId<TyFunctionDecl>> {
        Self::for_each_md_idx(context, md_idx, |md_idx| {
            self.md_test_decl_index_cache
                .get(&md_idx)
                .cloned()
                .or_else(|| {
                    // Create a new decl index and save it in the cache
                    md_idx
                        .get_content(context)
                        .unwrap_struct("decl_index", 1)
                        .and_then(|fields| {
                            let index = fields[0]
                                .unwrap_integer()
                                .map(|index| DeclId::new(index as usize))?;
                            self.md_test_decl_index_cache.insert(md_idx, index);
                            Some(index)
                        })
                })
        })
    }

    pub(crate) fn md_to_purity(
        &mut self,
        context: &Context,
        md_idx: Option<MetadataIndex>,
    ) -> Purity {
        // If the purity metadata is not available, we assume the function to
        // be pure, because in the case of a pure function, we do not store
        // its purity attribute, to avoid bloating the metadata.
        Self::for_each_md_idx(context, md_idx, |md_idx| {
            self.md_purity_cache.get(&md_idx).copied().or_else(|| {
                // Create a new purity and save it in the cache.
                md_idx
                    .get_content(context)
                    .unwrap_struct("purity", 1)
                    .and_then(|fields| {
                        fields[0].unwrap_string().and_then(|purity_str| {
                            let purity = match purity_str {
                                "reads" => Some(Purity::Reads),
                                "writes" => Some(Purity::Writes),
                                "readswrites" => Some(Purity::ReadsWrites),
                                _otherwise => panic!("Invalid purity metadata: {purity_str}."),
                            }?;

                            self.md_purity_cache.insert(md_idx, purity);

                            Some(purity)
                        })
                    })
            })
        })
        .unwrap_or(Purity::Pure)
    }

    /// Gets Inline information from metadata index.
    /// TODO: We temporarily allow this because we need this
    /// in the sway-ir inliner, but cannot access it. So the code
    /// itself has been (modified and) copied there. When we decide
    /// on the right place for Metadata to be
    /// (and how it can be accessed form sway-ir), this will be fixed.
    #[allow(dead_code)]
    pub(crate) fn md_to_inline(
        &mut self,
        context: &Context,
        md_idx: Option<MetadataIndex>,
    ) -> Option<Inline> {
        Self::for_each_md_idx(context, md_idx, |md_idx| {
            self.md_inline_cache.get(&md_idx).copied().or_else(|| {
                // Create a new inline and save it in the cache.
                md_idx
                    .get_content(context)
                    .unwrap_struct("inline", 1)
                    .and_then(|fields| fields[0].unwrap_string())
                    .and_then(|inline_str| {
                        let inline = match inline_str {
                            "always" => Some(Inline::Always),
                            "never" => Some(Inline::Never),
                            _otherwise => None,
                        }?;

                        self.md_inline_cache.insert(md_idx, inline);

                        Some(inline)
                    })
            })
        })
    }

    fn md_to_file_location(
        &mut self,
        context: &Context,
        md: &Metadatum,
    ) -> Option<(Arc<PathBuf>, Source)> {
        md.unwrap_index().and_then(|md_idx| {
            self.md_file_loc_cache.get(&md_idx).cloned().or_else(|| {
                // Create a new file location (path and src) and save it in the cache.
                md_idx
                    .get_content(context)
                    .unwrap_source_id()
                    .and_then(|source_id| {
                        let path_buf = context.source_engine.get_path(source_id);
                        let src = std::fs::read_to_string(&path_buf).ok()?;
                        let path_and_src = (Arc::new(path_buf), src.as_str().into());

                        self.md_file_loc_cache.insert(md_idx, path_and_src.clone());

                        Some(path_and_src)
                    })
            })
        })
    }

    pub(crate) fn val_to_span(&mut self, context: &Context, value: Value) -> Option<Span> {
        self.md_to_span(context, value.get_metadata(context))
    }

    pub(crate) fn span_to_md(
        &mut self,
        context: &mut Context,
        span: &Span,
    ) -> Option<MetadataIndex> {
        self.span_md_cache.get(span).copied().or_else(|| {
            span.source_id().and_then(|source_id| {
                let md_idx = self.create_metadata_from_span(context, source_id, span, "span")?;
                self.span_md_cache.insert(span.clone(), md_idx);
                Some(md_idx)
            })
        })
    }

    /// Returns [MetadataIndex] with [Metadatum::Struct] tagged with `tag`
    /// whose content will be the provided `span`.
    ///
    /// If the `span` does not have [Span::source_id], `None` is returned.
    ///
    /// This [Span] can later be retrieved from the [MetadataIndex] by calling
    /// [Self::md_to_tagged_span].
    pub(crate) fn tagged_span_to_md(
        &mut self,
        context: &mut Context,
        span: &Span,
        tag: &'static str,
    ) -> Option<MetadataIndex> {
        let span_and_tag = (span.clone(), tag);
        self.tagged_span_md_cache
            .get(&span_and_tag)
            .copied()
            .or_else(|| {
                span.source_id().and_then(|source_id| {
                    let md_idx = self.create_metadata_from_span(context, source_id, span, tag)?;
                    self.tagged_span_md_cache.insert(span_and_tag, md_idx);
                    Some(md_idx)
                })
            })
    }

    /// Returns [MetadataIndex] with [Metadatum::Struct]
    /// whose content will be the [Span] of the `fn_name` [Ident].
    ///
    /// If that span does not have [Span::source_id], `None` is returned.
    ///
    /// This [Span] can later be retrieved from the [MetadataIndex] by calling
    /// [Self::md_to_fn_name_span].
    pub(crate) fn fn_name_span_to_md(
        &mut self,
        context: &mut Context,
        fn_name: &Ident,
    ) -> Option<MetadataIndex> {
        self.tagged_span_to_md(context, &fn_name.span(), "fn_name_span")
    }

    /// Returns [MetadataIndex] with [Metadatum::Struct]
    /// whose content will be the [Span] of the `call_path`.
    ///
    /// If that span does not have [Span::source_id], `None` is returned.
    ///
    /// This [Span] can later be retrieved from the [MetadataIndex] by calling
    /// [Self::md_to_fn_call_path_span].
    pub(crate) fn fn_call_path_span_to_md(
        &mut self,
        context: &mut Context,
        call_path: &CallPath,
    ) -> Option<MetadataIndex> {
        self.tagged_span_to_md(context, &call_path.span(), "fn_call_path_span")
    }

    pub(crate) fn test_decl_index_to_md(
        &mut self,
        context: &mut Context,
        decl_index: DeclId<TyFunctionDecl>,
    ) -> Option<MetadataIndex> {
        self.test_decl_index_md_cache
            .get(&decl_index)
            .copied()
            .or_else(|| {
                let md_idx = MetadataIndex::new_struct(
                    context,
                    "decl_index",
                    vec![Metadatum::Integer(decl_index.inner() as u64)],
                );
                self.test_decl_index_md_cache.insert(decl_index, md_idx);

                Some(md_idx)
            })
    }

    pub(crate) fn purity_to_md(
        &mut self,
        context: &mut Context,
        purity: Purity,
    ) -> Option<MetadataIndex> {
        // If the function is pure, we do not store the purity attribute,
        // to avoid bloating the metadata.
        (purity != Purity::Pure).then(|| {
            self.purity_md_cache
                .get(&purity)
                .copied()
                .unwrap_or_else(|| {
                    // Create new metadatum.
                    let field = match purity {
                        Purity::Pure => unreachable!("Already checked for Pure above."),
                        Purity::Reads => "reads",
                        Purity::Writes => "writes",
                        Purity::ReadsWrites => "readswrites",
                    };
                    let md_idx = MetadataIndex::new_struct(
                        context,
                        "purity",
                        vec![Metadatum::String(field.to_owned())],
                    );

                    self.purity_md_cache.insert(purity, md_idx);

                    md_idx
                })
        })
    }

    /// Inserts Inline information into metadata.
    pub(crate) fn inline_to_md(
        &mut self,
        context: &mut Context,
        inline: Inline,
    ) -> Option<MetadataIndex> {
        Some(
            self.inline_md_cache
                .get(&inline)
                .copied()
                .unwrap_or_else(|| {
                    // Create new metadatum.
                    let field = match inline {
                        Inline::Always => "always",
                        Inline::Never => "never",
                    };
                    let md_idx = MetadataIndex::new_struct(
                        context,
                        "inline",
                        vec![Metadatum::String(field.to_owned())],
                    );

                    self.inline_md_cache.insert(inline, md_idx);

                    md_idx
                }),
        )
    }

    fn file_location_to_md(
        &mut self,
        context: &mut Context,
        source_id: SourceId,
    ) -> Option<MetadataIndex> {
        self.file_loc_md_cache.get(&source_id).copied().or_else(|| {
            let md_idx = MetadataIndex::new_source_id(context, source_id);
            self.file_loc_md_cache.insert(source_id, md_idx);

            Some(md_idx)
        })
    }

    fn for_each_md_idx<T, F: FnMut(MetadataIndex) -> Option<T>>(
        context: &Context,
        md_idx: Option<MetadataIndex>,
        mut f: F,
    ) -> Option<T> {
        // If md_idx is not None and is a list then try them all.
        md_idx.and_then(|md_idx| {
            if let Some(md_idcs) = md_idx.get_content(context).unwrap_list() {
                md_idcs.iter().find_map(|md_idx| f(*md_idx))
            } else {
                f(md_idx)
            }
        })
    }

    fn create_span_from_metadatum_fields(
        &mut self,
        context: &Context,
        fields: &[Metadatum],
    ) -> Option<Span> {
        let (path, src) = self.md_to_file_location(context, &fields[0])?;
        let start = fields[1].unwrap_integer()?;
        let end = fields[2].unwrap_integer()?;
        let source_engine = context.source_engine();
        let source_id = source_engine.get_source_id(&path);
        let span = Span::new(src, start as usize, end as usize, Some(source_id))?;
        Some(span)
    }

    fn create_metadata_from_span(
        &mut self,
        context: &mut Context,
        source_id: &SourceId,
        span: &Span,
        tag: &'static str,
    ) -> Option<MetadataIndex> {
        let file_location_md_idx = self.file_location_to_md(context, *source_id)?;
        let md_idx = MetadataIndex::new_struct(
            context,
            tag,
            vec![
                Metadatum::Index(file_location_md_idx),
                Metadatum::Integer(span.start() as u64),
                Metadatum::Integer(span.end() as u64),
            ],
        );
        Some(md_idx)
    }
}

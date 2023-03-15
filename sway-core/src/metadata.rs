use crate::{
    decl_engine::DeclId,
    language::{ty::TyFunctionDeclaration, Inline, Purity},
};

use sway_ir::{Context, MetadataIndex, Metadatum, Value};
use sway_types::Span;

use std::{collections::HashMap, path::PathBuf, rc::Rc, sync::Arc};

///! IR metadata needs to be consistent between IR generation (converting Spans, etc. to metadata)
///! and ASM generation (converting the metadata back again).  Here we consolidate all of
///! `sway-core`s metadata needs into a single place to enable that consistency.
///!
///! The [`MetadataManager`] also does its best to reduce redundancy by caching certain common
///! elements, such as source paths and storage attributes, and to avoid recreating the same
///! indices repeatedly.

#[derive(Default)]
pub(crate) struct MetadataManager {
    md_span_cache: HashMap<MetadataIndex, Span>,
    md_file_loc_cache: HashMap<MetadataIndex, (Arc<PathBuf>, Arc<str>)>,
    md_storage_op_cache: HashMap<MetadataIndex, StorageOperation>,
    md_storage_key_cache: HashMap<MetadataIndex, u64>,
    md_inline_cache: HashMap<MetadataIndex, Inline>,
    md_test_decl_index_cache: HashMap<MetadataIndex, DeclId<TyFunctionDeclaration>>,
    md_config_const_name_cache: HashMap<MetadataIndex, Rc<str>>,

    span_md_cache: HashMap<Span, MetadataIndex>,
    file_loc_md_cache: HashMap<*const PathBuf, MetadataIndex>,
    storage_op_md_cache: HashMap<Purity, MetadataIndex>,
    storage_key_md_cache: HashMap<u64, MetadataIndex>,
    inline_md_cache: HashMap<Inline, MetadataIndex>,
    test_decl_index_md_cache: HashMap<DeclId<TyFunctionDeclaration>, MetadataIndex>,
    config_const_name_md_cache: HashMap<Rc<str>, MetadataIndex>,
}

#[derive(Clone, Copy)]
pub(crate) enum StorageOperation {
    Reads,
    Writes,
    ReadsWrites,
}

impl MetadataManager {
    pub(crate) fn md_to_span(
        &mut self,
        context: &Context,
        md_idx: Option<MetadataIndex>,
    ) -> Option<Span> {
        Self::for_each_md_idx(context, md_idx, |md_idx| {
            self.md_span_cache.get(&md_idx).cloned().or_else(|| {
                // Create a new span and save it in the cache.
                md_idx
                    .get_content(context)
                    .unwrap_struct("span", 3)
                    .and_then(|fields| {
                        let (path, src) = self.md_to_file_location(context, &fields[0])?;
                        let start = fields[1].unwrap_integer()?;
                        let end = fields[2].unwrap_integer()?;
                        let span = Span::new(src, start as usize, end as usize, Some(path))?;

                        self.md_span_cache.insert(md_idx, span.clone());

                        Some(span)
                    })
            })
        })
    }

    pub(crate) fn md_to_test_decl_index(
        &mut self,
        context: &Context,
        md_idx: Option<MetadataIndex>,
    ) -> Option<DeclId<TyFunctionDeclaration>> {
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

    pub(crate) fn md_to_storage_op(
        &mut self,
        context: &Context,
        md_idx: Option<MetadataIndex>,
    ) -> Option<StorageOperation> {
        Self::for_each_md_idx(context, md_idx, |md_idx| {
            self.md_storage_op_cache.get(&md_idx).copied().or_else(|| {
                // Create a new storage op and save it in the cache.
                md_idx
                    .get_content(context)
                    .unwrap_struct("storage", 1)
                    .and_then(|fields| {
                        fields[0].unwrap_string().and_then(|stor_str| {
                            let op = match stor_str {
                                "reads" => Some(StorageOperation::Reads),
                                "writes" => Some(StorageOperation::Writes),
                                "readswrites" => Some(StorageOperation::ReadsWrites),
                                _otherwise => None,
                            }?;

                            self.md_storage_op_cache.insert(md_idx, op);

                            Some(op)
                        })
                    })
            })
        })
    }

    pub(crate) fn md_to_storage_key(
        &mut self,
        context: &Context,
        md_idx: Option<MetadataIndex>,
    ) -> Option<u64> {
        Self::for_each_md_idx(context, md_idx, |md_idx| {
            self.md_storage_key_cache.get(&md_idx).copied().or_else(|| {
                // Create a new storage key and save it in the cache.
                md_idx
                    .get_content(context)
                    .unwrap_struct("state_index", 1)
                    .and_then(|fields| {
                        let key = fields[0].unwrap_integer()?;

                        self.md_storage_key_cache.insert(md_idx, key);

                        Some(key)
                    })
            })
        })
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
    ) -> Option<(Arc<PathBuf>, Arc<str>)> {
        md.unwrap_index().and_then(|md_idx| {
            self.md_file_loc_cache.get(&md_idx).cloned().or_else(|| {
                // Create a new file location (path and src) and save it in the cache.
                md_idx
                    .get_content(context)
                    .unwrap_string()
                    .and_then(|path_buf_str| {
                        let path_buf = PathBuf::from(path_buf_str);
                        let src = std::fs::read_to_string(&path_buf).ok()?;
                        let path_and_src = (Arc::new(path_buf), Arc::from(src));

                        self.md_file_loc_cache.insert(md_idx, path_and_src.clone());

                        Some(path_and_src)
                    })
            })
        })
    }

    pub(crate) fn md_to_config_const_name(
        &mut self,
        context: &Context,
        md_idx: Option<MetadataIndex>,
    ) -> Option<Rc<str>> {
        Self::for_each_md_idx(context, md_idx, |md_idx| {
            self.md_config_const_name_cache
                .get(&md_idx)
                .cloned()
                .or_else(|| {
                    md_idx
                        .get_content(context)
                        .unwrap_struct("config_name", 1)
                        .and_then(|fields| {
                            fields[0].unwrap_string().map(|name| {
                                let name: Rc<str> = Rc::from(name);
                                self.md_config_const_name_cache.insert(md_idx, name.clone());
                                name
                            })
                        })
                })
        })
    }

    pub(crate) fn val_to_span(&mut self, context: &Context, value: Value) -> Option<Span> {
        self.md_to_span(context, value.get_metadata(context))
    }

    pub(crate) fn val_to_storage_key(&mut self, context: &Context, value: Value) -> Option<u64> {
        self.md_to_storage_key(context, value.get_metadata(context))
    }

    pub(crate) fn span_to_md(
        &mut self,
        context: &mut Context,
        span: &Span,
    ) -> Option<MetadataIndex> {
        self.span_md_cache.get(span).copied().or_else(|| {
            span.path().and_then(|path_buf| {
                // Create new metadata.
                let file_location_md_idx = self.file_location_to_md(context, path_buf)?;
                let md_idx = MetadataIndex::new_struct(
                    context,
                    "span",
                    vec![
                        Metadatum::Index(file_location_md_idx),
                        Metadatum::Integer(span.start() as u64),
                        Metadatum::Integer(span.end() as u64),
                    ],
                );

                self.span_md_cache.insert(span.clone(), md_idx);

                Some(md_idx)
            })
        })
    }

    pub(crate) fn test_decl_index_to_md(
        &mut self,
        context: &mut Context,
        decl_index: DeclId<TyFunctionDeclaration>,
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

    pub(crate) fn storage_key_to_md(
        &mut self,
        context: &mut Context,
        storage_key: u64,
    ) -> Option<MetadataIndex> {
        self.storage_key_md_cache
            .get(&storage_key)
            .copied()
            .or_else(|| {
                // Create new metadatum.
                let md_idx = MetadataIndex::new_struct(
                    context,
                    "state_index",
                    vec![Metadatum::Integer(storage_key)],
                );

                self.storage_key_md_cache.insert(storage_key, md_idx);

                Some(md_idx)
            })
    }

    pub(crate) fn purity_to_md(
        &mut self,
        context: &mut Context,
        purity: Purity,
    ) -> Option<MetadataIndex> {
        (purity != Purity::Pure).then(|| {
            self.storage_op_md_cache
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
                        "storage",
                        vec![Metadatum::String(field.to_owned())],
                    );

                    self.storage_op_md_cache.insert(purity, md_idx);

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
        path: &Arc<PathBuf>,
    ) -> Option<MetadataIndex> {
        self.file_loc_md_cache
            .get(&Arc::as_ptr(path))
            .copied()
            .or_else(|| {
                let md_idx = MetadataIndex::new_string(context, path.to_string_lossy());

                self.file_loc_md_cache.insert(Arc::as_ptr(path), md_idx);

                Some(md_idx)
            })
    }

    pub(crate) fn config_const_name_to_md(
        &mut self,
        context: &mut Context,
        name: &Rc<str>,
    ) -> Option<MetadataIndex> {
        self.config_const_name_md_cache
            .get(name)
            .copied()
            .or_else(|| {
                let md_idx = MetadataIndex::new_struct(
                    context,
                    "config_name",
                    vec![Metadatum::String(name.to_string())],
                );

                self.config_const_name_md_cache.insert(name.clone(), md_idx);

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
}

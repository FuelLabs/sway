//! Interactive TUI for browsing IR pass dumps.

use std::collections::BTreeMap;
use std::io::{self, Read, Stdout, Write};
use std::process::Child;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    MouseButton, MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{Frame, Terminal};

use crate::config::{self, FocusName, SessionState};
use crate::highlight;
use crate::parse::{
    diff_stats, find_functions, find_md_refs, parse, parse_source_map, prepare_ir_text,
    print_final_ir, spawn_shell, strip_ansi, strip_metadata, FuncStats, MdMode, ParsedIr,
    SourceMap, VersionMode,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Tree,
    Main,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum InputMode {
    PassFilter,
    FnFilter,
    Search,
    /// Prompt for a directory used to resolve missing source files.
    SourceRoot,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LoadKind {
    Input,
    Diff,
}

struct PassEntry {
    project: String,
    /// Full `// IR:` header, e.g. `[dce] Dead Code Elimination`.
    name: String,
    body: String,
    changed: bool,
    /// Stats for the prepared IR of this pass (current filters/metadata/version).
    stats: FuncStats,
}

struct ProjectGroup {
    name: String,
    expanded: bool,
    passes: Vec<usize>,
}

#[derive(Clone, Copy)]
enum TreeRow {
    Project(usize),
    Pass(usize),
}

enum LoadOutcome {
    Ir(String),
    Diff { text: String, pass_idx: usize },
    Err(String),
    Cancelled,
}

struct LoadingJob {
    kind: LoadKind,
    started: Instant,
    rx: Receiver<LoadOutcome>,
    /// Full command / path shown in the loading modal (not truncated).
    label: String,
    child: Arc<Mutex<Option<Child>>>,
    cancelled: Arc<AtomicBool>,
    /// Bytes read from the command/file so far (updated by the worker).
    bytes_received: Arc<AtomicUsize>,
}

pub(crate) struct TuiOptions {
    pub input_path: Option<String>,
    pub cmd: Option<String>,
    pub filter_fn: Vec<String>,
    pub metadata: MdMode,
    pub version: VersionMode,
    pub source: bool,
    pub start_diff: bool,
}

struct App {
    opts: TuiOptions,
    passes: Vec<PassEntry>,
    projects: Vec<ProjectGroup>,
    tree_rows: Vec<TreeRow>,
    list_state: ListState,
    focus: Focus,
    input_mode: Option<InputMode>,
    input_buf: String,
    pass_filter: String,
    fn_filter: String,
    search: String,
    search_matches: Vec<usize>,
    search_idx: usize,
    scroll: u16,
    h_scroll: u16,
    show_diff: bool,
    show_help: bool,
    show_line_numbers: bool,
    metadata: MdMode,
    version: VersionMode,
    show_source: bool,
    /// Extra directory searched when IR metadata paths are missing on disk.
    source_root: Option<PathBuf>,
    /// All `!N = …` defs seen in the loaded dump (survives function filters).
    cached_source_map: SourceMap,
    status: String,
    status_ttl: Option<Instant>,
    main_text: String,
    main_is_diff: bool,
    stats_line: String,
    cached_lines: Vec<Line<'static>>,
    dirty: bool,
    loading: Option<LoadingJob>,
    tree_area: Rect,
    main_area: Rect,
    /// Visible text rows in the main panel (excluding borders).
    main_view_rows: u16,
    /// When false, main panel shows plain text (SSA highlights still apply).
    syntax_highlight: bool,
    /// SSA name → palette index. Background always overrides syntax.
    ssa_highlights: BTreeMap<String, usize>,
    /// Next palette slot to assign when focusing a new SSA.
    ssa_next_slot: usize,
    /// Cursor into main-panel text (line, char col), set by mouse click/move.
    main_cursor_line: usize,
    main_cursor_col: usize,
    /// Navigation/view restore applied once after the first successful load.
    pending_restore: Option<SessionState>,
    needs_persist: bool,
}

impl App {
    fn new(opts: TuiOptions) -> Self {
        let (cfg, session) = config::load_session(opts.cmd.as_deref(), opts.input_path.as_deref());
        let restored = session.is_some();
        let s = session.unwrap_or_default();

        let metadata = if restored {
            MdMode::from(s.metadata)
        } else {
            opts.metadata
        };
        let version = if restored {
            VersionMode::from(s.version)
        } else {
            opts.version
        };
        let show_source = if restored { s.show_source } else { opts.source };
        let show_diff = if restored { s.show_diff } else { opts.start_diff };
        let fn_filter = if restored {
            s.fn_filter.clone()
        } else {
            opts.filter_fn.join(",")
        };
        let focus = match s.focus {
            FocusName::Tree => Focus::Tree,
            FocusName::Main => Focus::Main,
        };
        let show_line_numbers = if restored { s.show_line_numbers } else { true };
        let syntax_highlight = if restored { s.syntax_highlight } else { true };
        let pending_restore = restored.then_some(s.clone());

        Self {
            metadata,
            version,
            show_source,
            source_root: cfg.source_root,
            cached_source_map: SourceMap::empty(),
            show_diff,
            fn_filter,
            opts,
            passes: Vec::new(),
            projects: Vec::new(),
            tree_rows: Vec::new(),
            list_state: ListState::default(),
            focus,
            input_mode: None,
            input_buf: String::new(),
            pass_filter: if restored {
                s.pass_filter.clone()
            } else {
                String::new()
            },
            search: if restored {
                s.search.clone()
            } else {
                String::new()
            },
            search_matches: Vec::new(),
            search_idx: 0,
            scroll: if restored { s.scroll } else { 0 },
            h_scroll: if restored { s.h_scroll } else { 0 },
            show_help: false,
            show_line_numbers,
            syntax_highlight,
            ssa_highlights: BTreeMap::new(),
            ssa_next_slot: 0,
            main_cursor_line: 0,
            main_cursor_col: 0,
            status: String::new(),
            status_ttl: None,
            main_text: String::from("(waiting for IR…)"),
            main_is_diff: false,
            stats_line: String::new(),
            cached_lines: Vec::new(),
            dirty: true,
            loading: None,
            tree_area: Rect::default(),
            main_area: Rect::default(),
            main_view_rows: 1,
            pending_restore,
            needs_persist: false,
        }
    }

    fn mark_persist(&mut self) {
        self.needs_persist = true;
    }

    fn flush_persist(&mut self) {
        if !self.needs_persist {
            return;
        }
        self.needs_persist = false;
        let session = self.snapshot_session();
        let _ = config::persist(
            self.opts.cmd.as_deref(),
            self.opts.input_path.as_deref(),
            self.source_root.as_deref(),
            session,
        );
    }

    fn snapshot_session(&self) -> SessionState {
        let focus = match self.focus {
            Focus::Tree => FocusName::Tree,
            Focus::Main => FocusName::Main,
        };
        let (selected_project, selected_pass) = match self.selected_pass() {
            Some(p) => (Some(p.project.clone()), Some(p.name.clone())),
            None => (None, None),
        };
        SessionState {
            focus,
            pass_filter: self.pass_filter.clone(),
            fn_filter: self.fn_filter.clone(),
            search: self.search.clone(),
            scroll: self.scroll,
            h_scroll: self.h_scroll,
            show_diff: self.show_diff,
            show_line_numbers: self.show_line_numbers,
            syntax_highlight: self.syntax_highlight,
            metadata: self.metadata.into(),
            version: self.version.into(),
            show_source: self.show_source,
            selected_project,
            selected_pass,
            expanded_projects: self
                .projects
                .iter()
                .filter(|p| p.expanded)
                .map(|p| p.name.clone())
                .collect(),
        }
    }

    fn apply_pending_restore(&mut self) {
        let Some(r) = self.pending_restore.take() else {
            return;
        };
        for p in &mut self.projects {
            p.expanded = r.expanded_projects.iter().any(|n| n == &p.name);
        }
        self.select_pass_by_project_and_name(
            r.selected_project.as_deref(),
            r.selected_pass.as_deref(),
        );
        self.scroll = r.scroll;
        self.h_scroll = r.h_scroll;
    }

    fn is_loading(&self) -> bool {
        self.loading.is_some()
    }

    fn spinner(&self) -> &'static str {
        const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let Some(job) = &self.loading else {
            return FRAMES[0];
        };
        let i = (job.started.elapsed().as_millis() / 80) as usize % FRAMES.len();
        FRAMES[i]
    }

    fn set_status(&mut self, msg: impl Into<String>, secs: u64) {
        self.status = msg.into();
        self.status_ttl = Some(Instant::now() + Duration::from_secs(secs));
    }

    fn fn_filters(&self) -> Vec<String> {
        self.fn_filter
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    fn start_load(&mut self) {
        if self.is_loading() {
            return;
        }
        let cmd = self.opts.cmd.clone();
        let path = self.opts.input_path.clone();
        // Keep the full command — the loading modal wraps as needed.
        let label = if let Some(c) = &cmd {
            format!("running `{c}`")
        } else if let Some(p) = &path {
            format!("reading `{p}`")
        } else {
            String::from("loading")
        };

        let (tx, rx) = mpsc::channel();
        let child_slot = Arc::new(Mutex::new(None));
        let cancelled = Arc::new(AtomicBool::new(false));
        let bytes_received = Arc::new(AtomicUsize::new(0));
        let child_w = Arc::clone(&child_slot);
        let cancel_w = Arc::clone(&cancelled);
        let bytes_w = Arc::clone(&bytes_received);

        thread::spawn(move || {
            let outcome = match (cmd, path) {
                (Some(cmd), _) => match spawn_shell(&cmd) {
                    Ok(mut child) => {
                        let mut out = match child.stdout.take() {
                            Some(s) => s,
                            None => {
                                let _ = tx.send(LoadOutcome::Err(
                                    "failed to capture `--cmd` stdout".into(),
                                ));
                                return;
                            }
                        };
                        if let Ok(mut slot) = child_w.lock() {
                            *slot = Some(child);
                        }
                        let mut data = Vec::new();
                        let mut buf = [0u8; 64 * 1024];
                        let read_res = loop {
                            if cancel_w.load(Ordering::SeqCst) {
                                break Ok(());
                            }
                            match out.read(&mut buf) {
                                Ok(0) => break Ok(()),
                                Ok(n) => {
                                    bytes_w.fetch_add(n, Ordering::Relaxed);
                                    data.extend_from_slice(&buf[..n]);
                                }
                                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                                Err(e) => break Err(e),
                            }
                        };
                        let status = child_w
                            .lock()
                            .ok()
                            .and_then(|mut s| s.take())
                            .and_then(|mut c| c.wait().ok());
                        let text = String::from_utf8_lossy(&data).into_owned();
                        if cancel_w.load(Ordering::SeqCst) {
                            LoadOutcome::Cancelled
                        } else if let Err(e) = read_res {
                            LoadOutcome::Err(format!("failed to read `--cmd` output: {e}"))
                        } else if text.trim().is_empty() && status.is_some_and(|s| !s.success()) {
                            let code = status.and_then(|s| s.code()).unwrap_or(-1);
                            LoadOutcome::Err(format!(
                                "`--cmd` exited with status {code} and produced no output"
                            ))
                        } else {
                            LoadOutcome::Ir(text)
                        }
                    }
                    Err(e) => LoadOutcome::Err(format!("{e:#}")),
                },
                (None, Some(path)) => match std::fs::File::open(&path) {
                    Ok(mut f) => {
                        let mut data = Vec::new();
                        let mut buf = [0u8; 64 * 1024];
                        let read_res = loop {
                            if cancel_w.load(Ordering::SeqCst) {
                                break Ok(());
                            }
                            match f.read(&mut buf) {
                                Ok(0) => break Ok(()),
                                Ok(n) => {
                                    bytes_w.fetch_add(n, Ordering::Relaxed);
                                    data.extend_from_slice(&buf[..n]);
                                }
                                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                                Err(e) => break Err(e),
                            }
                        };
                        if cancel_w.load(Ordering::SeqCst) {
                            LoadOutcome::Cancelled
                        } else if let Err(e) = read_res {
                            LoadOutcome::Err(format!("failed to read {path}: {e}"))
                        } else {
                            LoadOutcome::Ir(String::from_utf8_lossy(&data).into_owned())
                        }
                    }
                    Err(e) => LoadOutcome::Err(format!("failed to read {path}: {e}")),
                },
                _ => LoadOutcome::Err("no input path or --cmd provided".into()),
            };
            let _ = tx.send(outcome);
        });

        self.loading = Some(LoadingJob {
            kind: LoadKind::Input,
            started: Instant::now(),
            rx,
            label,
            child: child_slot,
            cancelled,
            bytes_received,
        });
        self.status.clear();
        self.status_ttl = None;
    }

    fn cancel_load(&mut self) {
        let Some(job) = &self.loading else {
            return;
        };
        job.cancelled.store(true, Ordering::SeqCst);
        if let Ok(mut slot) = job.child.lock() {
            if let Some(mut child) = slot.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
        self.set_status("Cancelled…", 2);
    }

    fn poll_loading(&mut self) -> bool {
        let Some(job) = &self.loading else {
            return false;
        };
        match job.rx.try_recv() {
            Ok(outcome) => {
                let elapsed = job.started.elapsed();
                let kind = job.kind;
                self.loading = None;
                match outcome {
                    LoadOutcome::Ir(raw) => match self.apply_raw(&raw) {
                        Ok(()) => self.set_status(
                            format!(
                                "Loaded {} passes in {:.1}s",
                                self.passes.len(),
                                elapsed.as_secs_f32()
                            ),
                            3,
                        ),
                        Err(e) => {
                            self.main_text = format!("(load failed)\n{e:#}");
                            self.main_is_diff = false;
                            self.dirty = true;
                            self.set_status(format!("Load failed: {e:#}"), 8);
                        }
                    },
                    LoadOutcome::Diff { text, pass_idx } => {
                        if self.selected_pass_idx() == Some(pass_idx) && self.show_diff {
                            self.main_text = text;
                            self.main_is_diff = true;
                            self.dirty = true;
                            self.recompute_search();
                            self.set_status(
                                format!("Diff ready in {:.1}s", elapsed.as_secs_f32()),
                                2,
                            );
                        }
                    }
                    LoadOutcome::Err(e) => {
                        if self.passes.is_empty() {
                            self.main_text = format!("(load failed)\n{e}");
                            self.main_is_diff = false;
                            self.dirty = true;
                        }
                        self.set_status(format!("Load failed: {e}"), 8);
                    }
                    LoadOutcome::Cancelled => {
                        if kind == LoadKind::Input && self.passes.is_empty() {
                            self.main_text =
                                String::from("(cancelled — press F5 to retry)");
                            self.main_is_diff = false;
                            self.dirty = true;
                        }
                        self.set_status("Cancelled", 3);
                    }
                }
                true
            }
            Err(TryRecvError::Empty) => false,
            Err(TryRecvError::Disconnected) => {
                self.loading = None;
                self.set_status("Load worker disconnected", 8);
                true
            }
        }
    }

    fn apply_raw(&mut self, raw: &str) -> Result<()> {
        let keep_project = self
            .selected_pass()
            .map(|p| p.project.clone())
            .or_else(|| {
                self.pending_restore
                    .as_ref()
                    .and_then(|r| r.selected_project.clone())
            });
        let keep_pass = self
            .selected_pass()
            .map(|p| p.name.clone())
            .or_else(|| {
                self.pending_restore
                    .as_ref()
                    .and_then(|r| r.selected_pass.clone())
            })
            .unwrap_or_default();
        let restoring = self.pending_restore.is_some();
        let cleaned = strip_ansi(raw);
        // Cache every metadata definition from the full dump up front. Function
        // filters later drop the `!N = …` decl section from the view, but
        // source overlay still needs those defs to resolve spans.
        let mut sm = SourceMap::empty();
        if let Some(root) = &self.source_root {
            sm.set_source_roots([root.clone()]);
        }
        sm.merge_from_text(&cleaned);
        self.cached_source_map = sm;
        let irs = parse(&cleaned);
        if irs.is_empty() {
            bail!("no `// IR:` dumps found in input");
        }
        self.rebuild_passes(&irs);
        if restoring {
            self.apply_pending_restore();
        } else {
            self.recompute_tree();
            self.select_pass_by_project_and_name(keep_project.as_deref(), Some(&keep_pass));
            self.scroll = 0;
            self.h_scroll = 0;
        }
        self.refresh_main();
        // Clamp scroll after content is known.
        let max = self.main_text.lines().count().saturating_sub(1) as u16;
        self.scroll = self.scroll.min(max);
        self.mark_persist();
        Ok(())
    }

    fn request_reload(&mut self) {
        if self.is_loading() {
            self.set_status("Already loading…", 2);
            return;
        }
        self.start_load();
    }

    fn rebuild_passes(&mut self, irs: &[ParsedIr]) {
        let filters = self.fn_filters();
        let mut previous: Option<String> = None;
        let mut entries = Vec::with_capacity(irs.len());
        for ir in irs {
            if ir.is_initial() {
                previous = None;
            }
            let prepared =
                prepare_ir_text(&filters, self.metadata, self.version, ir).unwrap_or_default();
            let changed = previous.as_ref().map(|p| p != &prepared).unwrap_or(true);
            let stats = FuncStats::compute_stats(&prepared);
            previous = Some(prepared);
            entries.push(PassEntry {
                project: ir.project.clone(),
                name: ir.pass_name.clone(),
                body: ir.body.clone(),
                changed,
                stats,
            });
        }
        self.passes = entries;
        self.rebuild_projects();
    }

    fn rebuild_projects(&mut self) {
        let mut projects = Vec::<ProjectGroup>::new();
        for (i, entry) in self.passes.iter().enumerate() {
            match projects.last_mut() {
                Some(p) if p.name == entry.project => p.passes.push(i),
                _ => projects.push(ProjectGroup {
                    name: entry.project.clone(),
                    expanded: false,
                    passes: vec![i],
                }),
            }
        }
        for p in &mut projects {
            if let Some(old) = self.projects.iter().find(|o| o.name == p.name) {
                p.expanded = old.expanded;
            }
        }
        self.projects = projects;
    }

    fn recompute_tree(&mut self) {
        let pf = self.pass_filter.to_ascii_lowercase();
        let prev = self.selected_pass_idx();
        self.tree_rows.clear();
        for (pi, proj) in self.projects.iter().enumerate() {
            let visible: Vec<usize> = proj
                .passes
                .iter()
                .copied()
                .filter(|&i| {
                    if pf.is_empty() {
                        return true;
                    }
                    let name = &self.passes[i].name;
                    name.to_ascii_lowercase().contains(&pf)
                        || pass_short_label(name).to_ascii_lowercase().contains(&pf)
                })
                .collect();
            if visible.is_empty() {
                continue;
            }
            self.tree_rows.push(TreeRow::Project(pi));
            if proj.expanded {
                for pass_idx in visible {
                    self.tree_rows.push(TreeRow::Pass(pass_idx));
                }
            }
        }
        if let Some(pass_idx) = prev {
            if let Some(row) = self
                .tree_rows
                .iter()
                .position(|r| matches!(r, TreeRow::Pass(i) if *i == pass_idx))
            {
                self.list_state.select(Some(row));
                return;
            }
        }
        let first = self
            .tree_rows
            .iter()
            .position(|r| matches!(r, TreeRow::Pass(_)))
            .or_else(|| (!self.tree_rows.is_empty()).then_some(0));
        self.list_state.select(first);
    }

    fn select_pass_by_project_and_name(
        &mut self,
        project: Option<&str>,
        name: Option<&str>,
    ) {
        let name = name.unwrap_or("").trim();
        if name.is_empty() {
            self.recompute_tree();
            if let Some(row) = self
                .tree_rows
                .iter()
                .position(|r| matches!(r, TreeRow::Pass(_)))
            {
                self.list_state.select(Some(row));
            }
            return;
        }
        let pass_idx = self
            .passes
            .iter()
            .position(|p| {
                p.name == name && project.map(|pr| pr == p.project).unwrap_or(true)
            })
            .or_else(|| self.passes.iter().position(|p| p.name == name));
        let Some(pass_idx) = pass_idx else {
            self.recompute_tree();
            return;
        };
        if let Some(proj) = self
            .projects
            .iter_mut()
            .find(|p| p.passes.contains(&pass_idx))
        {
            proj.expanded = true;
        }
        self.recompute_tree();
        if let Some(row) = self
            .tree_rows
            .iter()
            .position(|r| matches!(r, TreeRow::Pass(i) if *i == pass_idx))
        {
            self.list_state.select(Some(row));
        }
    }

    fn selected_tree_row(&self) -> Option<TreeRow> {
        self.list_state
            .selected()
            .and_then(|i| self.tree_rows.get(i).copied())
    }

    fn selected_pass_idx(&self) -> Option<usize> {
        match self.selected_tree_row()? {
            TreeRow::Pass(i) => Some(i),
            TreeRow::Project(_) => None,
        }
    }

    fn selected_pass(&self) -> Option<&PassEntry> {
        self.selected_pass_idx().map(|i| &self.passes[i])
    }

    fn toggle_selected_project(&mut self) {
        let Some(TreeRow::Project(pi)) = self.selected_tree_row() else {
            return;
        };
        if let Some(p) = self.projects.get_mut(pi) {
            p.expanded = !p.expanded;
        }
        self.recompute_tree();
    }

    fn cancel_diff_only(&mut self) {
        let Some(job) = &self.loading else {
            return;
        };
        if job.kind != LoadKind::Diff {
            return;
        }
        job.cancelled.store(true, Ordering::SeqCst);
        self.loading = None;
    }

    fn start_diff_job(
        &mut self,
        pass_idx: usize,
        prev_text: String,
        text: String,
        prev_name: String,
    ) -> bool {
        if self
            .loading
            .as_ref()
            .is_some_and(|j| j.kind == LoadKind::Input)
        {
            return false;
        }
        self.cancel_diff_only();
        let label = format!("diff vs `{prev_name}`");
        let (tx, rx) = mpsc::channel();
        let cancelled = Arc::new(AtomicBool::new(false));
        let cancel_w = Arc::clone(&cancelled);
        thread::spawn(move || {
            if cancel_w.load(Ordering::SeqCst) {
                let _ = tx.send(LoadOutcome::Cancelled);
                return;
            }
            let changeset = prettydiff::diff_lines(&prev_text, &text);
            let ops = changeset.diff();
            if cancel_w.load(Ordering::SeqCst) {
                let _ = tx.send(LoadOutcome::Cancelled);
                return;
            }
            let (adds, removes) = diff_stats(&ops);
            let mut rendered =
                format!("// Diff vs {prev_name}: adds={adds} removes={removes}\n");
            rendered.push_str(&render_diff(&ops));
            let _ = tx.send(LoadOutcome::Diff {
                text: rendered,
                pass_idx,
            });
        });
        self.loading = Some(LoadingJob {
            kind: LoadKind::Diff,
            started: Instant::now(),
            rx,
            label,
            child: Arc::new(Mutex::new(None)),
            cancelled,
            bytes_received: Arc::new(AtomicUsize::new(0)),
        });
        true
    }

    /// Prepare a pass body for the main panel, optionally embedding source.
    ///
    /// When source overlay is on, metadata references are kept long enough to
    /// resolve spans (even if metadata display is off), then stripped again if
    /// the user has metadata toggled off.
    ///
    /// Returns `(text, needs_source_root, sample_missing_path)`.
    fn prepare_pass_view(&self, pass_idx: usize) -> Option<(String, bool, Option<String>)> {
        let filters = self.fn_filters();
        let pass = &self.passes[pass_idx];
        let ir = ParsedIr {
            project: pass.project.clone(),
            pass_name: pass.name.clone(),
            body: pass.body.clone(),
        };
        let md = if self.show_source && self.metadata == MdMode::Without {
            // Need inline `!N` refs to resolve source spans.
            MdMode::AsParsed
        } else {
            self.metadata
        };
        let mut text = prepare_ir_text(&filters, md, self.version, &ir)?;
        let mut needs_source_root = false;
        let mut sample_missing = None;
        if self.show_source {
            // Prefer the dump-wide cache so function filters can't hide defs.
            let mut sm = if self.cached_source_map.entry_count() > 0 {
                self.cached_source_map.clone_for_lookup()
            } else {
                parse_source_map(&ir.body)
            };
            // Also merge this pass body in case it has newer/extra defs.
            sm.merge_from_text(&ir.body);
            if let Some(root) = &self.source_root {
                sm.set_source_roots([root.clone()]);
            }
            let mut buf = Vec::new();
            let _ = print_final_ir(&mut buf, &text, Some(&mut sm));
            text = String::from_utf8_lossy(&buf).into_owned();
            if self.metadata == MdMode::Without {
                text = strip_metadata(&text);
            }
            needs_source_root = sm.has_source_ids() && sm.resolved_files() == 0;
            sample_missing = sm.missing_paths().iter().next().cloned();
        }
        Some((text, needs_source_root, sample_missing))
    }

    fn open_source_root_modal(&mut self, hint: &str) {
        self.input_mode = Some(InputMode::SourceRoot);
        self.input_buf = self
            .source_root
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        if !hint.is_empty() {
            self.set_status(hint, 6);
        }
    }

    fn apply_source_root(&mut self, raw: &str) -> bool {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            self.set_status("source root: empty path", 3);
            return false;
        }
        let path = PathBuf::from(trimmed);
        let path = if path.is_absolute() {
            path
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(&path))
                .unwrap_or(path)
        };
        if !path.is_dir() {
            self.set_status(
                format!("source root: not a directory: {}", path.display()),
                5,
            );
            return false;
        }
        self.source_root = Some(path.clone());
        self.cached_source_map.set_source_roots([path.clone()]);
        self.mark_persist();
        self.flush_persist();
        self.set_status(format!("source root: {}", path.display()), 3);
        true
    }

    fn refresh_main(&mut self) {
        self.cancel_diff_only();
        let Some(idx) = self.selected_pass_idx() else {
            self.main_text = if matches!(self.selected_tree_row(), Some(TreeRow::Project(_))) {
                String::from("(select a pass under the project)")
            } else {
                String::from("(no pass selected)")
            };
            self.main_is_diff = false;
            self.stats_line.clear();
            self.dirty = true;
            return;
        };

        let Some((text, needs_root, missing)) = self.prepare_pass_view(idx) else {
            self.main_text = String::from("(no functions match the current filter)");
            self.main_is_diff = false;
            self.stats_line.clear();
            self.dirty = true;
            return;
        };
        if needs_root && self.input_mode.is_none() {
            let hint = match missing {
                Some(p) => format!(
                    "missing `{p}` — enter the project root that contains your .sw sources"
                ),
                None => String::from(
                    "source files not found — enter the project root directory"
                ),
            };
            self.open_source_root_modal(&hint);
        }

        // Stats from IR without the source overlay so comment lines don't
        // inflate instruction counts.
        let stats_src = {
            let filters = self.fn_filters();
            let ir = ParsedIr {
                project: self.passes[idx].project.clone(),
                pass_name: self.passes[idx].name.clone(),
                body: self.passes[idx].body.clone(),
            };
            prepare_ir_text(&filters, self.metadata, self.version, &ir)
                .unwrap_or_else(|| text.clone())
        };
        let stats = FuncStats::compute_stats(&stats_src);
        let prev_stats = if idx > 0 && self.passes[idx - 1].project == self.passes[idx].project {
            Some(self.passes[idx - 1].stats.clone())
        } else {
            None
        };
        self.stats_line = format_full_stats(&stats, prev_stats.as_ref());
        self.passes[idx].stats = stats;

        if self.show_diff && idx > 0 {
            let prev_project = self.passes[idx - 1].project.clone();
            let prev_name = self.passes[idx - 1].name.clone();
            if prev_project == self.passes[idx].project {
                if let Some((prev_text, _, _)) = self.prepare_pass_view(idx - 1) {
                    if self.start_diff_job(idx, prev_text.clone(), text.clone(), prev_name.clone())
                    {
                        self.main_text = String::from("(computing diff…)");
                        self.main_is_diff = false;
                        self.dirty = true;
                        return;
                    }
                    let changeset = prettydiff::diff_lines(&prev_text, &text);
                    let ops = changeset.diff();
                    let (adds, removes) = diff_stats(&ops);
                    let mut rendered =
                        format!("// Diff vs {prev_name}: adds={adds} removes={removes}\n");
                    rendered.push_str(&render_diff(&ops));
                    self.main_text = rendered;
                    self.main_is_diff = true;
                    self.dirty = true;
                    self.recompute_search();
                    return;
                }
            }
        }

        self.main_text = text;
        self.main_is_diff = false;
        self.dirty = true;
        self.recompute_search();
    }
    fn recompute_search(&mut self) {
        self.search_matches.clear();
        if self.search.is_empty() {
            return;
        }
        let needle = self.search.to_ascii_lowercase();
        for (i, line) in self.main_text.lines().enumerate() {
            if line.to_ascii_lowercase().contains(&needle) {
                self.search_matches.push(i);
            }
        }
        self.search_idx = 0;
        if let Some(&line) = self.search_matches.first() {
            self.scroll = line as u16;
        }
    }

    fn ensure_cache(&mut self) {
        if !self.dirty {
            self.clamp_scroll();
            return;
        }
        let cur = self.search_matches.get(self.search_idx).copied();
        let mut lines = highlight::highlight_ir_with_search(
            &self.main_text,
            self.main_is_diff,
            &self.search,
            cur,
            self.syntax_highlight,
        );
        let ssa: Vec<(String, ratatui::style::Color)> = self
            .ssa_highlights
            .iter()
            .map(|(name, &slot)| {
                (
                    name.clone(),
                    highlight::SYMBOL_PALETTE[slot % highlight::SYMBOL_PALETTE.len()],
                )
            })
            .collect();
        highlight::apply_symbol_highlights(&mut lines, &ssa);
        self.cached_lines = lines;
        self.dirty = false;
        self.clamp_scroll();
    }

    fn line_number_gutter_width(&self) -> u16 {
        if self.show_line_numbers {
            7 // "{:>4} │ "
        } else {
            0
        }
    }

    /// Map a screen cell inside the main panel to (line, col) in `main_text`.
    fn main_pos_from_screen(&self, col: u16, row: u16) -> Option<(usize, usize)> {
        let inner = self.main_area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
        if col < inner.x
            || row < inner.y
            || col >= inner.x.saturating_add(inner.width)
            || row >= inner.y.saturating_add(inner.height)
        {
            return None;
        }
        let rel_row = (row - inner.y) as usize;
        let line = self.scroll as usize + rel_row;
        let gutter = self.line_number_gutter_width();
        let rel_col = col.saturating_sub(inner.x);
        if rel_col < gutter {
            return Some((line, 0));
        }
        let col_in_text = (rel_col - gutter) as usize + self.h_scroll as usize;
        Some((line, col_in_text))
    }

    fn set_main_cursor_from_screen(&mut self, col: u16, row: u16) {
        if let Some((line, c)) = self.main_pos_from_screen(col, row) {
            self.main_cursor_line = line;
            self.main_cursor_col = c;
        }
    }

    fn toggle_syntax_highlight(&mut self) {
        self.syntax_highlight = !self.syntax_highlight;
        self.dirty = true;
        self.mark_persist();
        self.set_status(
            if self.syntax_highlight {
                "syntax highlight: ON"
            } else {
                "syntax highlight: OFF"
            },
            2,
        );
    }

    fn toggle_symbol_at_cursor(&mut self) {
        if self.focus != Focus::Main {
            self.set_status("F2: focus the IR panel first", 3);
            return;
        }
        let line = self.main_text.lines().nth(self.main_cursor_line).unwrap_or("");
        let Some(name) = highlight::symbol_at_col(line, self.main_cursor_col, &self.main_text)
        else {
            self.set_status(
                "F2: cursor is not on an SSA / local / method name",
                3,
            );
            return;
        };
        if self.ssa_highlights.remove(&name).is_some() {
            self.set_status(format!("highlight OFF: {name}"), 3);
        } else {
            let slot = self.ssa_next_slot;
            self.ssa_next_slot = self.ssa_next_slot.wrapping_add(1);
            self.ssa_highlights.insert(name.clone(), slot);
            self.set_status(
                format!(
                    "highlight ON: {name} (color {})",
                    (slot % highlight::SYMBOL_PALETTE.len()) + 1
                ),
                3,
            );
        }
        self.dirty = true;
    }

    /// Keep vertical scroll inside the current main-panel content. Toggles like
    /// metadata/version rewrite the text in place and can otherwise leave
    /// `scroll` past the last line, which panics when slicing `cached_lines`.
    fn clamp_scroll(&mut self) {
        let max = self.main_text.lines().count().saturating_sub(1) as u16;
        if self.scroll > max {
            self.scroll = max;
        }
    }

    fn select_next(&mut self) {
        let len = self.tree_rows.len();
        if len == 0 {
            return;
        }
        let i = self.list_state.selected().unwrap_or(0);
        self.list_state.select(Some((i + 1).min(len - 1)));
        self.scroll = 0;
        self.refresh_main();
    }

    fn select_prev(&mut self) {
        if self.tree_rows.is_empty() {
            return;
        }
        let i = self.list_state.selected().unwrap_or(0);
        self.list_state.select(Some(i.saturating_sub(1)));
        self.scroll = 0;
        self.refresh_main();
    }

    fn activate_selection(&mut self) {
        match self.selected_tree_row() {
            Some(TreeRow::Project(_)) => self.toggle_selected_project(),
            Some(TreeRow::Pass(_)) => {
                self.scroll = 0;
                self.refresh_main();
            }
            None => {}
        }
    }

    fn click_tree_at(&mut self, row: usize) {
        if row >= self.tree_rows.len() {
            return;
        }
        self.focus = Focus::Tree;
        self.list_state.select(Some(row));
        self.activate_selection();
    }

    fn scroll_by(&mut self, delta: i32, step: u16) {
        let max = self.main_text.lines().count().saturating_sub(1) as u16;
        if delta < 0 {
            self.scroll = self.scroll.saturating_sub(step);
        } else {
            self.scroll = self.scroll.saturating_add(step).min(max);
        }
    }

    fn jump_next_fn(&mut self) {
        let decls = find_functions(&self.main_text);
        if decls.is_empty() {
            self.set_status("No functions in view", 2);
            return;
        }
        let cur = self.scroll as usize;
        if let Some(d) = decls.iter().find(|d| d.start > cur).or_else(|| decls.first()) {
            self.scroll = d.start as u16;
            self.set_status(format!("→ fn {}", d.name), 2);
        }
    }

    fn jump_prev_fn(&mut self) {
        let decls = find_functions(&self.main_text);
        if decls.is_empty() {
            return;
        }
        let cur = self.scroll as usize;
        if let Some(d) = decls
            .iter()
            .rev()
            .find(|d| d.start < cur)
            .or_else(|| decls.last())
        {
            self.scroll = d.start as u16;
            self.set_status(format!("→ fn {}", d.name), 2);
        }
    }

    fn copy_main(&mut self) {
        match copy_text(&self.main_text) {
            Ok(via) => self.set_status(
                format!("Copied {} bytes via {via}", self.main_text.len()),
                3,
            ),
            Err(e) => self.set_status(format!("Clipboard error: {e}"), 5),
        }
    }

    /// Toggle metadata display on/off (`AsParsed` ↔ `Without`).
    fn toggle_metadata(&mut self) {
        let on = self.metadata != MdMode::Without;
        self.metadata = if on {
            MdMode::Without
        } else {
            MdMode::AsParsed
        };
        self.rederive_changed();
        self.refresh_main();
        self.set_status(
            if on {
                "metadata: OFF"
            } else {
                "metadata: ON"
            },
            2,
        );
    }

    /// Toggle version suffixes on/off (`AsParsed` ↔ `Without`).
    fn toggle_version(&mut self) {
        let on = self.version != VersionMode::Without;
        self.version = if on {
            VersionMode::Without
        } else {
            VersionMode::AsParsed
        };
        self.rederive_changed();
        self.refresh_main();
        self.set_status(
            if on {
                "version: OFF"
            } else {
                "version: ON"
            },
            2,
        );
    }

    fn rederive_changed(&mut self) {
        let filters = self.fn_filters();
        let mut previous: Option<String> = None;
        for entry in &mut self.passes {
            if entry.name == "Initial" {
                previous = None;
            }
            let ir = ParsedIr {
                project: entry.project.clone(),
                pass_name: entry.name.clone(),
                body: entry.body.clone(),
            };
            let prepared =
                prepare_ir_text(&filters, self.metadata, self.version, &ir).unwrap_or_default();
            entry.changed = previous.as_ref().map(|p| p != &prepared).unwrap_or(true);
            entry.stats = FuncStats::compute_stats(&prepared);
            previous = Some(prepared);
        }
    }

    fn handle_mouse(&mut self, col: u16, row: u16, kind: MouseEventKind) {
        let in_tree = contains(self.tree_area, col, row);
        let in_main = contains(self.main_area, col, row);
        match kind {
            MouseEventKind::Down(MouseButton::Left) if in_tree => {
                let inner = self.tree_area.inner(Margin {
                    horizontal: 1,
                    vertical: 1,
                });
                if row >= inner.y {
                    let rel = (row - inner.y) as usize;
                    let idx = self.list_state.offset().saturating_add(rel);
                    self.click_tree_at(idx);
                } else {
                    self.focus = Focus::Tree;
                }
            }
            MouseEventKind::Down(MouseButton::Left) if in_main => {
                self.focus = Focus::Main;
                self.set_main_cursor_from_screen(col, row);
            }
            MouseEventKind::Moved if in_main => {
                self.set_main_cursor_from_screen(col, row);
            }
            MouseEventKind::Drag(MouseButton::Left) if in_main => {
                self.focus = Focus::Main;
                self.set_main_cursor_from_screen(col, row);
            }
            MouseEventKind::ScrollDown if in_tree => self.select_next(),
            MouseEventKind::ScrollUp if in_tree => self.select_prev(),
            MouseEventKind::ScrollDown => self.scroll_by(1, 3),
            MouseEventKind::ScrollUp => self.scroll_by(-1, 3),
            _ => {}
        }
    }
}

fn contains(area: Rect, col: u16, row: u16) -> bool {
    col >= area.x
        && row >= area.y
        && col < area.x.saturating_add(area.width)
        && row < area.y.saturating_add(area.height)
}

/// Full function stats for the status bar, with optional deltas vs the previous pass.
fn format_full_stats(s: &FuncStats, prev: Option<&FuncStats>) -> String {
    format_full_stats_spans(s, prev)
        .into_iter()
        .map(|s| s.content.to_string())
        .collect()
}

fn append_stat_delta_spans(spans: &mut Vec<Span<'static>>, label: &str, cur: usize, prev: Option<usize>) {
    if !spans.is_empty() {
        spans.push(Span::styled(" ", Style::default().fg(Color::DarkGray)));
    }
    spans.push(Span::styled(
        format!("{label}={cur}"),
        Style::default().fg(Color::DarkGray),
    ));
    if let Some(p) = prev {
        if p != cur {
            let d = cur as isize - p as isize;
            let style = if d > 0 {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };
            spans.push(Span::styled(format!("({d:+})"), style));
        }
    }
}

/// Styled status-bar stats: absolute counts in gray, `+` deltas green, `-` deltas red.
fn format_full_stats_spans(s: &FuncStats, prev: Option<&FuncStats>) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    append_stat_delta_spans(&mut spans, "args", s.args, prev.map(|p| p.args));
    append_stat_delta_spans(&mut spans, "locals", s.locals, prev.map(|p| p.locals));
    append_stat_delta_spans(&mut spans, "blocks", s.blocks, prev.map(|p| p.blocks));
    append_stat_delta_spans(
        &mut spans,
        "instr",
        s.instructions,
        prev.map(|p| p.instructions),
    );

    let mut names: std::collections::BTreeSet<&str> = s.ops.keys().map(String::as_str).collect();
    if let Some(p) = prev {
        for k in p.ops.keys() {
            names.insert(k.as_str());
        }
    }
    for name in names {
        let cur = s.ops.get(name).copied().unwrap_or(0);
        let p = prev.and_then(|ps| ps.ops.get(name).copied());
        append_stat_delta_spans(&mut spans, name, cur, p);
    }
    spans
}

/// Short pass label for the tree: keep `[name]`, drop the long description.
/// Headers without brackets (`Initial`, `Final`) are kept as-is.
fn pass_short_label(full: &str) -> &str {
    let trimmed = full.trim();
    if let Some(open) = trimmed.find('[') {
        if let Some(close) = trimmed[open..].find(']') {
            return &trimmed[open..open + close + 1];
        }
    }
    trimmed
}

/// Signed delta only (no absolute count). `None` when unchanged or no previous.
fn format_diff_only(cur: usize, prev: Option<usize>) -> Option<String> {
    let p = prev?;
    if p == cur {
        return None;
    }
    let d = cur as isize - p as isize;
    Some(format!("{d:+}"))
}

/// Compact stats shown beside each pass in the tree: deltas vs previous pass only.
fn format_tree_stats(s: &FuncStats, prev: Option<&FuncStats>) -> String {
    let mut parts = Vec::new();
    if let Some(d) = format_diff_only(s.args, prev.map(|p| p.args)) {
        parts.push(format!("a={d}"));
    }
    if let Some(d) = format_diff_only(s.locals, prev.map(|p| p.locals)) {
        parts.push(format!("l={d}"));
    }
    if let Some(d) = format_diff_only(s.blocks, prev.map(|p| p.blocks)) {
        parts.push(format!("b={d}"));
    }
    if let Some(d) = format_diff_only(s.instructions, prev.map(|p| p.instructions)) {
        parts.push(format!("i={d}"));
    }

    // Include every opcode that changed (including removals).
    let mut names: std::collections::BTreeSet<&str> = s.ops.keys().map(String::as_str).collect();
    if let Some(p) = prev {
        for k in p.ops.keys() {
            names.insert(k.as_str());
        }
    }
    for name in names {
        let cur = s.ops.get(name).copied().unwrap_or(0);
        let p = prev.and_then(|ps| ps.ops.get(name).copied());
        if let Some(d) = format_diff_only(cur, p) {
            parts.push(format!("{name}={d}"));
        }
    }

    if parts.is_empty() {
        String::new()
    } else {
        parts.join(" ")
    }
}

fn render_diff(ops: &[prettydiff::basic::DiffOp<&str>]) -> String {
    let mut out = String::new();
    for op in ops {
        match op {
            prettydiff::basic::DiffOp::Equal(lines) => {
                for l in *lines {
                    out.push_str("  ");
                    out.push_str(l);
                    out.push('\n');
                }
            }
            prettydiff::basic::DiffOp::Insert(lines) => {
                for l in *lines {
                    out.push_str("+ ");
                    out.push_str(l);
                    out.push('\n');
                }
            }
            prettydiff::basic::DiffOp::Remove(lines) => {
                for l in *lines {
                    out.push_str("- ");
                    out.push_str(l);
                    out.push('\n');
                }
            }
            prettydiff::basic::DiffOp::Replace(removed, inserted) => {
                for l in *removed {
                    out.push_str("- ");
                    out.push_str(l);
                    out.push('\n');
                }
                for l in *inserted {
                    out.push_str("+ ");
                    out.push_str(l);
                    out.push('\n');
                }
            }
        }
    }
    out
}

fn copy_text(text: &str) -> Result<&'static str> {
    match arboard::Clipboard::new().and_then(|mut c| {
        c.set_text(text.to_string())?;
        Ok(())
    }) {
        Ok(()) => Ok("clipboard"),
        Err(_) => {
            let encoded = base64_encode(text.as_bytes());
            let seq = format!("\x1b]52;c;{encoded}\x07");
            let mut stdout = io::stdout();
            stdout.write_all(seq.as_bytes())?;
            stdout.flush()?;
            Ok("OSC52")
        }
    }
}

fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let mut buf = [0u8; 3];
        for (i, b) in chunk.iter().enumerate() {
            buf[i] = *b;
        }
        let n = chunk.len();
        let triple = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);
        out.push(TABLE[((triple >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3f) as usize] as char);
        out.push(if n > 1 {
            TABLE[((triple >> 6) & 0x3f) as usize] as char
        } else {
            '='
        });
        out.push(if n > 2 {
            TABLE[(triple & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    out
}

pub(crate) fn run(opts: TuiOptions) -> Result<()> {
    let mut app = App::new(opts);
    app.start_load();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = event_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    result
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        if let Some(deadline) = app.status_ttl {
            if Instant::now() >= deadline {
                app.status.clear();
                app.status_ttl = None;
            }
        }
        let _ = app.poll_loading();
        terminal.draw(|f| draw(f, app))?;
        app.flush_persist();

        let poll_ms = if app.is_loading() { 80 } else { 200 };
        if !event::poll(Duration::from_millis(poll_ms))? {
            continue;
        }

        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                if app.show_help {
                    app.show_help = false;
                    continue;
                }
                if app.input_mode.is_none() {
                    if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                        if app.is_loading() {
                            app.cancel_load();
                        }
                        app.mark_persist();
                        app.flush_persist();
                        return Ok(());
                    }
                    if key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        if app.is_loading() {
                            app.cancel_load();
                            continue;
                        }
                        app.mark_persist();
                        app.flush_persist();
                        return Ok(());
                    }
                }
                if app.is_loading() {
                    continue;
                }
                if let Some(mode) = app.input_mode {
                    match key.code {
                        KeyCode::Esc => {
                            app.input_mode = None;
                            app.input_buf.clear();
                        }
                        KeyCode::Enter => {
                            match mode {
                                InputMode::PassFilter => {
                                    app.pass_filter = app.input_buf.clone();
                                    app.recompute_tree();
                                    app.scroll = 0;
                                    app.refresh_main();
                                }
                                InputMode::FnFilter => {
                                    app.fn_filter = app.input_buf.clone();
                                    app.rederive_changed();
                                    app.scroll = 0;
                                    app.refresh_main();
                                    app.set_status(format!("fn filter: {}", app.fn_filter), 2);
                                }
                                InputMode::Search => {
                                    app.search = app.input_buf.clone();
                                    app.dirty = true;
                                    app.recompute_search();
                                }
                                InputMode::SourceRoot => {
                                    let raw = app.input_buf.clone();
                                    if app.apply_source_root(&raw) {
                                        app.input_mode = None;
                                        app.input_buf.clear();
                                        app.refresh_main();
                                    }
                                    // Keep modal open on invalid path.
                                    continue;
                                }
                            }
                            app.input_mode = None;
                            app.input_buf.clear();
                        }
                        KeyCode::Backspace => {
                            app.input_buf.pop();
                        }
                        KeyCode::Char(c) => app.input_buf.push(c),
                        _ => {}
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::F(1) => app.toggle_syntax_highlight(),
                    KeyCode::F(2) => app.toggle_symbol_at_cursor(),
                    KeyCode::F(5) => app.request_reload(),
                    KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.request_reload();
                    }
                    KeyCode::Tab => {
                        app.focus = match app.focus {
                            Focus::Tree => Focus::Main,
                            Focus::Main => Focus::Tree,
                        };
                    }
                    KeyCode::Char('?') => app.show_help = true,
                    KeyCode::Char('y') => app.copy_main(),
                    KeyCode::Char('d') => {
                        app.show_diff = !app.show_diff;
                        app.scroll = 0;
                        app.refresh_main();
                        app.set_status(
                            if app.show_diff {
                                "Diff view ON"
                            } else {
                                "Diff view OFF"
                            },
                            2,
                        );
                    }
                    KeyCode::Char('m') => app.toggle_metadata(),
                    KeyCode::Char('v') => app.toggle_version(),
                    KeyCode::Char('s') => {
                        app.show_source = !app.show_source;
                        if !app.show_source {
                            app.set_status("source: OFF", 2);
                        } else if app.cached_source_map.entry_count() == 0 {
                            let has_refs = app
                                .selected_pass()
                                .is_some_and(|p| ir_has_inline_md_refs(&p.body));
                            if has_refs {
                                app.set_status(
                                    "source: ON — IR has !N refs but no defs; re-run with `--ir all,print-md`",
                                    8,
                                );
                            } else {
                                app.set_status(
                                    "source: ON (no metadata — use `--ir all,print-md`)",
                                    6,
                                );
                            }
                        } else {
                            app.set_status(
                                format!(
                                    "source: ON ({} metadata defs cached)",
                                    app.cached_source_map.entry_count()
                                ),
                                3,
                            );
                        }
                        app.refresh_main();
                    }
                    KeyCode::Char('S') => {
                        app.open_source_root_modal(
                            "enter the directory that contains your .sw sources",
                        );
                    }
                    KeyCode::Char('l') => {
                        app.show_line_numbers = !app.show_line_numbers;
                        app.dirty = true;
                    }
                    KeyCode::Char('/') => {
                        app.input_mode = Some(InputMode::Search);
                        app.input_buf = app.search.clone();
                    }
                    KeyCode::Char('p') => {
                        app.input_mode = Some(InputMode::PassFilter);
                        app.input_buf = app.pass_filter.clone();
                    }
                    KeyCode::Char('f') => {
                        app.input_mode = Some(InputMode::FnFilter);
                        app.input_buf = app.fn_filter.clone();
                    }
                    KeyCode::Char('n') if !app.search_matches.is_empty() => {
                        app.search_idx = (app.search_idx + 1) % app.search_matches.len();
                        app.scroll = app.search_matches[app.search_idx] as u16;
                        app.dirty = true;
                    }
                    KeyCode::Char('N') if !app.search_matches.is_empty() => {
                        app.search_idx = if app.search_idx == 0 {
                            app.search_matches.len() - 1
                        } else {
                            app.search_idx - 1
                        };
                        app.scroll = app.search_matches[app.search_idx] as u16;
                        app.dirty = true;
                    }
                    KeyCode::Char(']') => app.jump_next_fn(),
                    KeyCode::Char('[') => app.jump_prev_fn(),
                    KeyCode::Char('g') => app.scroll = 0,
                    KeyCode::Char('G') => {
                        app.scroll = app.main_text.lines().count().saturating_sub(1) as u16;
                    }
                    KeyCode::Down | KeyCode::Char('j') => match app.focus {
                        Focus::Tree => app.select_next(),
                        Focus::Main => app.scroll_by(1, 1),
                    },
                    KeyCode::Up | KeyCode::Char('k') => match app.focus {
                        Focus::Tree => app.select_prev(),
                        Focus::Main => app.scroll_by(-1, 1),
                    },
                    KeyCode::PageDown => {
                        let h = terminal.size()?.height.saturating_sub(6);
                        app.scroll_by(1, h);
                    }
                    KeyCode::PageUp => {
                        let h = terminal.size()?.height.saturating_sub(6);
                        app.scroll_by(-1, h);
                    }
                    KeyCode::Char(' ') | KeyCode::Enter if app.focus == Focus::Tree => {
                        app.activate_selection();
                    }
                    KeyCode::Char(' ') if app.focus == Focus::Main => {
                        let h = terminal.size()?.height.saturating_sub(6);
                        app.scroll_by(1, h);
                    }
                    KeyCode::Left | KeyCode::Char('h') if app.focus == Focus::Tree => {
                        match app.selected_tree_row() {
                            Some(TreeRow::Project(_)) => {
                                if matches!(
                                    app.selected_tree_row(),
                                    Some(TreeRow::Project(pi))
                                        if app.projects.get(pi).is_some_and(|p| p.expanded)
                                ) {
                                    app.toggle_selected_project();
                                }
                            }
                            Some(TreeRow::Pass(pass_idx)) => {
                                if let Some((row, pi)) =
                                    app.tree_rows.iter().enumerate().rev().find_map(|(row, r)| {
                                        match r {
                                            TreeRow::Project(pi)
                                                if app.projects[*pi].passes.contains(&pass_idx) =>
                                            {
                                                Some((row, *pi))
                                            }
                                            _ => None,
                                        }
                                    })
                                {
                                    app.projects[pi].expanded = false;
                                    app.recompute_tree();
                                    if !app.tree_rows.is_empty() {
                                        app.list_state
                                            .select(Some(row.min(app.tree_rows.len() - 1)));
                                    }
                                }
                            }
                            None => {}
                        }
                    }
                    KeyCode::Right if app.focus == Focus::Tree => {
                        if let Some(TreeRow::Project(pi)) = app.selected_tree_row() {
                            if app.projects.get(pi).is_some_and(|p| !p.expanded) {
                                app.toggle_selected_project();
                            } else if let Some(sel) = app.list_state.selected() {
                                if sel + 1 < app.tree_rows.len() {
                                    app.list_state.select(Some(sel + 1));
                                    app.activate_selection();
                                }
                            }
                        }
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        app.h_scroll = app.h_scroll.saturating_sub(4);
                    }
                    KeyCode::Right => {
                        app.h_scroll = app.h_scroll.saturating_add(4);
                    }
                    KeyCode::Home => {
                        if app.focus == Focus::Tree && !app.tree_rows.is_empty() {
                            app.list_state.select(Some(0));
                            app.activate_selection();
                        } else {
                            app.scroll = 0;
                        }
                    }
                    KeyCode::End => {
                        if app.focus == Focus::Tree && !app.tree_rows.is_empty() {
                            app.list_state.select(Some(app.tree_rows.len() - 1));
                            app.activate_selection();
                        } else {
                            app.scroll =
                                app.main_text.lines().count().saturating_sub(1) as u16;
                        }
                    }
                    _ => {}
                }
                app.mark_persist();
            }
            Event::Mouse(m) if !app.is_loading() => {
                app.handle_mouse(m.column, m.row, m.kind);
                app.mark_persist();
            }
            _ => {}
        }
    }
}

fn draw(f: &mut Frame, app: &mut App) {
    let status_h = status_panel_height(app, f.area());
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),
            Constraint::Length(status_h),
            Constraint::Length(1),
        ])
        .split(f.area());

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(28), Constraint::Percentage(72)])
        .split(chunks[0]);

    app.tree_area = body[0];
    app.main_area = body[1];

    draw_tree(f, app, body[0]);
    draw_main(f, app, body[1]);
    draw_status(f, app, chunks[1]);
    draw_keys(f, chunks[2]);

    if app.is_loading() {
        draw_loading(f, app);
    }
    if let Some(mode) = app.input_mode {
        draw_input(f, app, mode);
    }
    if app.show_help {
        draw_help(f);
    }
}

/// How many terminal rows a single logical line needs when wrapped to `width`.
fn wrapped_line_count(text: &str, width: u16) -> u16 {
    let width = width.max(1) as usize;
    if text.is_empty() {
        return 1;
    }
    let chars = text.chars().count().max(1);
    ((chars + width - 1) / width) as u16
}


fn main_view_range(app: &App) -> (usize, usize) {
    let start = app.scroll as usize;
    let rows = app.main_view_rows.max(1) as usize;
    (start, start.saturating_add(rows))
}

fn ssa_offscreen_plain(app: &App) -> String {
    if app.ssa_highlights.is_empty() || app.is_loading() {
        return String::new();
    }
    let (start, end) = main_view_range(app);
    let mut parts = Vec::new();
    for name in app.ssa_highlights.keys() {
        let (above, below) = highlight::count_token_offscreen(&app.main_text, name, start, end);
        parts.push(format!("{name} ↑{above} ↓{below}"));
    }
    parts.join("  ")
}

fn ssa_offscreen_spans(app: &App) -> Line<'static> {
    if app.ssa_highlights.is_empty() {
        return Line::from("");
    }
    let (start, end) = main_view_range(app);
    let mut spans = vec![Span::raw(" ")];
    let mut first = true;
    for (name, &slot) in &app.ssa_highlights {
        if !first {
            spans.push(Span::styled("  ", Style::default().fg(Color::DarkGray)));
        }
        first = false;
        let bg = highlight::SYMBOL_PALETTE[slot % highlight::SYMBOL_PALETTE.len()];
        let (above, below) = highlight::count_token_offscreen(&app.main_text, name, start, end);
        spans.push(Span::styled(
            name.clone(),
            Style::default().bg(bg).fg(Color::White).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" ↑{above} ↓{below}"),
            Style::default().fg(Color::DarkGray),
        ));
    }
    Line::from(spans)
}

fn status_stats_plain(app: &App) -> String {
    if app.is_loading() {
        return String::new();
    }
    if let Some(pass) = app.selected_pass() {
        let idx = app.selected_pass_idx().unwrap();
        let prev = if idx > 0 && app.passes[idx - 1].project == pass.project {
            Some(&app.passes[idx - 1].stats)
        } else {
            None
        };
        format_full_stats(&pass.stats, prev)
    } else {
        app.stats_line.clone()
    }
}

fn status_header_plain(app: &App) -> String {
    if app.is_loading() {
        return String::from(" waiting for cmd output");
    }
    let pos = app
        .list_state
        .selected()
        .map(|i| format!("{}/{}", i + 1, app.tree_rows.len()))
        .unwrap_or_else(|| "-".into());
    let search = if app.search.is_empty() {
        String::new()
    } else if app.search_matches.is_empty() {
        format!("  search:\"{}\" (0)", app.search)
    } else {
        format!(
            "  search:\"{}\" ({}/{})",
            app.search,
            app.search_idx + 1,
            app.search_matches.len()
        )
    };
    let fn_f = if app.fn_filter.is_empty() {
        String::new()
    } else {
        format!("  fn:{}", app.fn_filter)
    };
    let status = if app.status.is_empty() {
        String::new()
    } else {
        format!(" │ {}", app.status)
    };
    let pass = app
        .selected_pass()
        .map(|p| format!("  {}", pass_short_label(&p.name)))
        .unwrap_or_default();
    format!(" {pos}{pass}{fn_f}{search}{status}")
}

/// Borders + header (+ wrapped stats). Grows with content; leaves room for body + keys.
fn status_panel_height(app: &App, frame: Rect) -> u16 {
    const KEYS: u16 = 1;
    const MIN_BODY: u16 = 5;
    const BORDERS: u16 = 2;

    let inner_w = frame.width.saturating_sub(2);
    let header_lines = wrapped_line_count(&status_header_plain(app), inner_w);
    let content = if app.is_loading() {
        header_lines
    } else {
        let stats = status_stats_plain(app);
        // Leading space matches what we render in the panel.
        let stats_lines = if stats.is_empty() {
            1
        } else {
            wrapped_line_count(&format!(" {stats}"), inner_w)
        };
        let ssa = ssa_offscreen_plain(app);
        let ssa_lines = if ssa.is_empty() {
            0
        } else {
            wrapped_line_count(&format!(" {ssa}"), inner_w)
        };
        header_lines + stats_lines + ssa_lines
    };

    let needed = content.saturating_add(BORDERS);
    let max = frame.height.saturating_sub(MIN_BODY.saturating_add(KEYS)).max(3);
    needed.min(max).max(3)
}

fn draw_tree(f: &mut Frame, app: &mut App, area: Rect) {
    let pass_rows = app
        .tree_rows
        .iter()
        .filter(|r| matches!(r, TreeRow::Pass(_)))
        .count();
    let title = if app.pass_filter.is_empty() {
        format!(" Tree ({pass_rows}/{}) ", app.passes.len())
    } else {
        format!(
            " Tree ({pass_rows}/{}) [{}] ",
            app.passes.len(),
            app.pass_filter
        )
    };
    let border = if app.focus == Focus::Tree {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let items: Vec<ListItem> = app
        .tree_rows
        .iter()
        .map(|row| match row {
            TreeRow::Project(pi) => {
                let p = &app.projects[*pi];
                let glyph = if p.expanded { "▼" } else { "▶" };
                let n = p.passes.len();
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{glyph} "),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        p.name.clone(),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!(" ({n})"), Style::default().fg(Color::DarkGray)),
                ]))
            }
            TreeRow::Pass(i) => {
                let p = &app.passes[*i];
                let marker = if p.changed { "●" } else { "○" };
                let style = if p.changed {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                let prev_stats = if *i > 0 && app.passes[*i - 1].project == p.project {
                    Some(&app.passes[*i - 1].stats)
                } else {
                    None
                };
                let label = pass_short_label(&p.name);
                let stats = format_tree_stats(&p.stats, prev_stats);
                let mut spans = vec![
                    Span::raw("  "),
                    Span::styled(format!("{marker} "), style),
                    Span::styled(
                        label.to_string(),
                        style.add_modifier(Modifier::BOLD),
                    ),
                ];
                if !stats.is_empty() {
                    spans.push(Span::styled(
                        format!("  {stats}"),
                        Style::default().fg(Color::DarkGray),
                    ));
                }
                ListItem::new(Line::from(spans))
            }
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(40, 60, 90))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");

    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_main(f: &mut Frame, app: &mut App, area: Rect) {
    app.ensure_cache();
    let pass_name = app
        .selected_pass()
        .map(|p| pass_short_label(&p.name))
        .unwrap_or("-");
    let mode = if app.main_is_diff { "DIFF" } else { "IR" };
    let src = if app.show_source { " +src" } else { "" };
    let title = format!(" {mode}{src}: {pass_name} ");
    let border = if app.focus == Focus::Main {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let inner_h = area.height.saturating_sub(2) as usize;
    app.main_view_rows = inner_h.max(1) as u16;
    let len = app.cached_lines.len();
    let start = if len == 0 {
        0
    } else {
        (app.scroll as usize).min(len - 1)
    };
    let end = (start + inner_h).min(len);

    let mut lines = Vec::with_capacity(end.saturating_sub(start));
    for (i, line) in app.cached_lines[start..end].iter().enumerate() {
        let abs = start + i;
        let content = highlight::skip_line_chars(line, app.h_scroll as usize);
        if app.show_line_numbers {
            let mut spans = vec![Span::styled(
                format!("{:>4} │ ", abs + 1),
                Style::default().fg(Color::DarkGray),
            )];
            spans.extend(content.spans.iter().cloned());
            lines.push(Line::from(spans));
        } else {
            lines.push(content);
        }
    }

    let para = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border),
    );
    f.render_widget(para, area);
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let pos = app
        .list_state
        .selected()
        .map(|i| format!("{}/{}", i + 1, app.tree_rows.len()))
        .unwrap_or_else(|| "-".into());
    let search = if app.search.is_empty() {
        String::new()
    } else if app.search_matches.is_empty() {
        format!("  search:\"{}\" (0)", app.search)
    } else {
        format!(
            "  search:\"{}\" ({}/{})",
            app.search,
            app.search_idx + 1,
            app.search_matches.len()
        )
    };
    let fn_f = if app.fn_filter.is_empty() {
        String::new()
    } else {
        format!("  fn:{}", app.fn_filter)
    };

    let (line1, style1) = if app.is_loading() {
        (
            String::from(" waiting for cmd output"),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        let status = if app.status.is_empty() {
            String::new()
        } else {
            format!(" │ {}", app.status)
        };
        let pass = app
            .selected_pass()
            .map(|p| format!("  {}", pass_short_label(&p.name)))
            .unwrap_or_default();
        (
            format!(" {pos}{pass}{fn_f}{search}{status}"),
            Style::default().fg(Color::White),
        )
    };

    // Prefer live stats from the selected pass (with deltas vs previous).
    // Fall back to `stats_line` only when no pass is selected.
    let line2 = if app.is_loading() {
        Line::from("")
    } else if let Some(pass) = app.selected_pass() {
        let idx = app.selected_pass_idx().unwrap();
        let prev = if idx > 0 && app.passes[idx - 1].project == pass.project {
            Some(&app.passes[idx - 1].stats)
        } else {
            None
        };
        let mut spans = vec![Span::styled(" ", Style::default().fg(Color::DarkGray))];
        spans.extend(format_full_stats_spans(&pass.stats, prev));
        Line::from(spans)
    } else {
        Line::from(Span::styled(
            format!(" {}", app.stats_line),
            Style::default().fg(Color::DarkGray),
        ))
    };

    let mut status_lines = vec![
        Line::from(Span::styled(line1, style1)),
        line2,
    ];
    if !app.ssa_highlights.is_empty() && !app.is_loading() {
        status_lines.push(ssa_offscreen_spans(app));
    }
    let para = Paragraph::new(status_lines)
    .wrap(Wrap { trim: false })
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(if app.is_loading() {
                " Loading "
            } else if app.selected_pass().is_some() {
                " Stats "
            } else {
                " Status "
            })
            .border_style(if app.is_loading() {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            }),
    );
    f.render_widget(para, area);
}

fn draw_loading(f: &mut Frame, app: &App) {
    let label = app
        .loading
        .as_ref()
        .map(|j| j.label.as_str())
        .unwrap_or("loading");
    let secs = app
        .loading
        .as_ref()
        .map(|j| j.started.elapsed().as_secs_f32())
        .unwrap_or(0.0);
    let bytes = app
        .loading
        .as_ref()
        .map(|j| j.bytes_received.load(Ordering::Relaxed))
        .unwrap_or(0);
    let bytes_label = format_byte_count(bytes);

    // Size the modal to fit the full command (wrapped), not a truncated label.
    let width = f.area().width.saturating_sub(4).clamp(40, 100);
    let inner_w = width.saturating_sub(4).max(20) as usize;
    let label_lines = wrapped_line_count(label, inner_w as u16).max(1);
    // borders(2) + blank + spinner/label lines + stats line
    let height = (4u16).saturating_add(label_lines).min(f.area().height.saturating_sub(2));
    let area = centered(
        ((width as u32 * 100) / f.area().width.max(1) as u32) as u16,
        height,
        f.area(),
    );

    let spin = app.spinner();
    let mut text = vec![Line::from("")];
    // First line of the command with spinner; remaining wrapped lines indented.
    let mut remaining = label;
    let mut first = true;
    while !remaining.is_empty() {
        let take = remaining
            .chars()
            .take(if first {
                inner_w.saturating_sub(4)
            } else {
                inner_w.saturating_sub(6)
            })
            .count();
        // Find a byte index at a char boundary.
        let mut end = 0;
        for (i, _) in remaining.char_indices().take(take) {
            end = i;
        }
        if end == 0 {
            // Single very wide char or empty take — consume one char.
            end = remaining
                .char_indices()
                .nth(1)
                .map(|(i, _)| i)
                .unwrap_or(remaining.len());
            if end == 0 {
                end = remaining.len();
            }
        } else {
            // Advance past the last included char.
            let last = remaining[end..].chars().next().map(|c| c.len_utf8()).unwrap_or(0);
            end += last;
        }
        let chunk = &remaining[..end.min(remaining.len())];
        remaining = &remaining[end.min(remaining.len())..];
        if first {
            text.push(Line::from(vec![
                Span::styled(
                    format!("  {spin}  "),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(chunk.to_string(), Style::default().fg(Color::White)),
            ]));
            first = false;
        } else {
            text.push(Line::from(Span::styled(
                format!("      {chunk}"),
                Style::default().fg(Color::White),
            )));
        }
    }
    text.push(Line::from(Span::styled(
        format!("      {secs:.1}s  ·  {bytes_label} received  ·  Ctrl-C cancels  ·  q quits"),
        Style::default().fg(Color::DarkGray),
    )));

    let para = Paragraph::new(text)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Working ")
                .border_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
        );
    f.render_widget(Clear, area);
    f.render_widget(para, area);
}

fn ir_has_inline_md_refs(text: &str) -> bool {
    text.lines().any(|line| !find_md_refs(line).is_empty())
}

fn format_byte_count(n: usize) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let n = n as f64;
    if n >= GB {
        format!("{:.2} GiB", n / GB)
    } else if n >= MB {
        format!("{:.2} MiB", n / MB)
    } else if n >= KB {
        format!("{:.1} KiB", n / KB)
    } else {
        format!("{n:.0} B")
    }
}

fn draw_keys(f: &mut Frame, area: Rect) {
    let keys = " Tab:focus  j/k:nav  Enter/←/→:tree  /:search  p/f:filter  d:diff  m:metadata  v:version  s:source  S:src-root  y:copy  F1:syntax  F2:symbol  F5:reload  ?:help  q:quit ";
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            keys,
            Style::default().fg(Color::Rgb(140, 140, 160)),
        ))),
        area,
    );
}

fn draw_input(f: &mut Frame, app: &App, mode: InputMode) {
    let (area, title, body) = match mode {
        InputMode::PassFilter => (
            centered(60, 3, f.area()),
            " Filter passes (substring) ",
            format!("> {}", app.input_buf),
        ),
        InputMode::FnFilter => (
            centered(60, 3, f.area()),
            " Filter functions (comma-separated) ",
            format!("> {}", app.input_buf),
        ),
        InputMode::Search => (
            centered(60, 3, f.area()),
            " Search IR ",
            format!("> {}", app.input_buf),
        ),
        InputMode::SourceRoot => {
            let current = app
                .source_root
                .as_ref()
                .map(|p| format!("current: {}
", p.display()))
                .unwrap_or_else(|| String::from("current: (none)
"));
            (
                centered(72, 7, f.area()),
                " Source root directory ",
                format!(
                    "{current}IR metadata paths were not found on disk.
\
                     Enter the project/workspace root that contains the .sw files:
\
                     > {}",
                    app.input_buf
                ),
            )
        }
    };
    let para = Paragraph::new(body)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(Clear, area);
    f.render_widget(para, area);
}

fn draw_help(f: &mut Frame) {
    let area = centered(86, 42, f.area());
    if area.width < 10 || area.height < 5 {
        return;
    }
    let text = vec![
        Line::from("Sway IR Inspect — keybindings & features"),
        Line::from(""),
        Line::from("Navigation"),
        Line::from("  Tab             Focus tree ↔ IR panel"),
        Line::from("  j/k ↑/↓         Navigate tree / scroll IR"),
        Line::from("  Enter / Space     Expand project or select pass"),
        Line::from("  ← / →           Collapse / expand project"),
        Line::from("  PgUp/PgDn       Page scroll IR"),
        Line::from("  g / G           Top / bottom of IR"),
        Line::from("  Click / Wheel   Select tree rows / scroll IR"),
        Line::from(""),
        Line::from("Search & filters"),
        Line::from("  p / f / /       Filter passes / functions / search text"),
        Line::from("  n / N           Next / previous search match"),
        Line::from(""),
        Line::from("View toggles"),
        Line::from("  F1              Toggle syntax highlighting (SSA/local/method colors still apply)"),
        Line::from("  d               Toggle diff vs previous pass"),
        Line::from("  m               Toggle metadata on/off"),
        Line::from("  v               Toggle version suffixes on/off"),
        Line::from("  s               Toggle source overlay (needs `--ir …,print-md`)"),
        Line::from("  S               Set/change source root directory (persisted)"),
        Line::from("  l               Toggle line numbers"),
        Line::from(""),
        Line::from("Symbol focus (F2) — IR panel focused"),
        Line::from("  Click/move on an SSA (`v0`), local (`x` in `local u64 x` / `get_local …, x`),"),
        Line::from("  or method/function name (`fn foo` / `call foo`), then press F2 to toggle."),
        Line::from("  Multiple symbols can be focused at once; each gets a distinct palette color."),
        Line::from("  Focus colors always override syntax highlighting (F1 on or off)."),
        Line::from("  Status bar shows each focused symbol with ↑N ↓M = occurrences above/below"),
        Line::from("  the visible IR window (defs + uses combined)."),
        Line::from(""),
        Line::from("Session / IO"),
        Line::from("  F5 / Ctrl-R     Reload dump / re-run `--cmd`"),
        Line::from("  y               Copy main panel to clipboard"),
        Line::from("  Ctrl-C          Cancel load/diff, or quit if idle"),
        Line::from("  q / Esc         Quit"),
        Line::from("  (auto-save)     Filters, view, selection, source root → `~/.config/forc-ir-inspect/`"),
        Line::from(""),
        Line::from("Tree markers"),
        Line::from("  ● white         Pass changed the IR vs the previous pass"),
        Line::from("  ○ gray          Pass left the IR unchanged"),
        Line::from(""),
        Line::from("Stats (status bar & tree)"),
        Line::from("  args/locals/blocks/instr   Function shape counts (+/− vs previous pass)"),
        Line::from("  <opcode>=N                 Opcode frequency (tree shows deltas only)"),
        Line::from(""),
        Line::from("  Press any key to close"),
    ];
    let para = Paragraph::new(text)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .border_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .style(Style::default().bg(Color::Black)),
        );
    f.render_widget(Clear, area);
    f.render_widget(para, area);
}

fn centered(percent_x: u16, height: u16, r: Rect) -> Rect {
    // Clamp so a short terminal still gets a visible popup.
    let height = height.min(r.height.saturating_sub(2)).max(3);
    let percent_x = percent_x.min(100);
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(r.height.saturating_sub(height) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vert[1])[1]
}


//! Persistent TUI session state under the OS config directory.
//!
//! - macOS / Linux: `~/.config/forc-ir-inspect/` (or `$XDG_CONFIG_HOME`)
//! - Windows: `%APPDATA%\forc-ir-inspect\`

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::parse::{MdMode, VersionMode};

const CONFIG_VERSION: u32 = 1;
const CONFIG_FILE: &str = "tui.json";
const LEGACY_SOURCE_ROOT: &str = "source_root";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct TuiConfigFile {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub source_root: Option<PathBuf>,
    #[serde(default)]
    pub sessions: BTreeMap<String, SessionState>,
}

fn default_version() -> u32 {
    CONFIG_VERSION
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct SessionState {
    #[serde(default)]
    pub focus: FocusName,
    #[serde(default)]
    pub pass_filter: String,
    #[serde(default)]
    pub fn_filter: String,
    #[serde(default)]
    pub search: String,
    #[serde(default)]
    pub scroll: u16,
    #[serde(default)]
    pub h_scroll: u16,
    #[serde(default)]
    pub show_diff: bool,
    #[serde(default = "default_true")]
    pub show_line_numbers: bool,
    #[serde(default = "default_true")]
    pub syntax_highlight: bool,
    #[serde(default)]
    pub metadata: MdModeName,
    #[serde(default)]
    pub version: VersionModeName,
    #[serde(default)]
    pub show_source: bool,
    #[serde(default)]
    pub selected_project: Option<String>,
    #[serde(default)]
    pub selected_pass: Option<String>,
    #[serde(default)]
    pub expanded_projects: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FocusName {
    #[default]
    Tree,
    Main,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MdModeName {
    #[default]
    AsParsed,
    With,
    Without,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum VersionModeName {
    #[default]
    AsParsed,
    With,
    Without,
}

impl From<MdMode> for MdModeName {
    fn from(m: MdMode) -> Self {
        match m {
            MdMode::AsParsed => Self::AsParsed,
            MdMode::With => Self::With,
            MdMode::Without => Self::Without,
        }
    }
}

impl From<MdModeName> for MdMode {
    fn from(m: MdModeName) -> Self {
        match m {
            MdModeName::AsParsed => Self::AsParsed,
            MdModeName::With => Self::With,
            MdModeName::Without => Self::Without,
        }
    }
}

impl From<VersionMode> for VersionModeName {
    fn from(m: VersionMode) -> Self {
        match m {
            VersionMode::AsParsed => Self::AsParsed,
            VersionMode::With => Self::With,
            VersionMode::Without => Self::Without,
        }
    }
}

impl From<VersionModeName> for VersionMode {
    fn from(m: VersionModeName) -> Self {
        match m {
            VersionModeName::AsParsed => Self::AsParsed,
            VersionModeName::With => Self::With,
            VersionModeName::Without => Self::Without,
        }
    }
}

/// Prefer XDG-style `~/.config` on all Unix platforms (including macOS).
/// Windows keeps `%APPDATA%` via `dirs::config_dir`.
pub(crate) fn config_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        Some(dirs::config_dir()?.join("forc-ir-inspect"))
    }
    #[cfg(not(windows))]
    {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| !p.as_os_str().is_empty())
            .or_else(|| dirs::home_dir().map(|h| h.join(".config")))?;
        Some(base.join("forc-ir-inspect"))
    }
}

pub(crate) fn config_path() -> Option<PathBuf> {
    Some(config_dir()?.join(CONFIG_FILE))
}

pub(crate) fn session_key(cmd: Option<&str>, input_path: Option<&str>) -> String {
    if let Some(cmd) = cmd {
        format!("cmd:{cmd}")
    } else if let Some(path) = input_path {
        let p = PathBuf::from(path);
        let abs = p
            .canonicalize()
            .unwrap_or_else(|_| std::env::current_dir().map(|c| c.join(&p)).unwrap_or(p));
        format!("path:{}", abs.display())
    } else {
        String::from("default")
    }
}

pub(crate) fn load_config() -> TuiConfigFile {
    let mut cfg: TuiConfigFile = config_path()
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default();
    cfg.version = CONFIG_VERSION;

    // Migrate the previous single-file source-root setting.
    if cfg.source_root.is_none() {
        if let Some(legacy) = load_legacy_source_root() {
            cfg.source_root = Some(legacy);
        }
    }
    if let Some(root) = &cfg.source_root {
        if !root.is_dir() {
            cfg.source_root = None;
        }
    }
    cfg
}

fn load_legacy_source_root() -> Option<PathBuf> {
    let path = config_dir()?.join(LEGACY_SOURCE_ROOT);
    let raw = fs::read_to_string(path).ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let p = PathBuf::from(trimmed);
    p.is_dir().then_some(p)
}

pub(crate) fn save_config(cfg: &TuiConfigFile) -> std::io::Result<()> {
    let path = config_path().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "no OS config directory")
    })?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let raw = serde_json::to_string_pretty(cfg)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, raw)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

pub(crate) fn load_session(
    cmd: Option<&str>,
    input_path: Option<&str>,
) -> (TuiConfigFile, Option<SessionState>) {
    let cfg = load_config();
    let key = session_key(cmd, input_path);
    let session = cfg.sessions.get(&key).cloned();
    (cfg, session)
}

/// Update `source_root` and the session for this input, then write to disk.
pub(crate) fn persist(
    cmd: Option<&str>,
    input_path: Option<&str>,
    source_root: Option<&Path>,
    session: SessionState,
) -> std::io::Result<()> {
    let mut cfg = load_config();
    cfg.source_root = source_root.map(Path::to_path_buf);
    cfg.sessions.insert(session_key(cmd, input_path), session);
    save_config(&cfg)
}

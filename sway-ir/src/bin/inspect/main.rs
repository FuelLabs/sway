//! ```text
//! > cargo r --bin inspect -r -- --help
//!
//! Interactive TUI (default on a TTY):
//!   inspect dump.txt
//!   inspect --cmd 'forc build --ir all'
//!
//! Classic stream output:
//!   inspect --classic dump.txt --print diff,without-md
//!   inspect --classic --cmd 'forc build --ir all' --print ir
//! ```

mod classic;
mod highlight;
mod parse;
mod tui;

use std::io::IsTerminal;

use anyhow::{bail, Result};
use clap::Parser;

use parse::{MdMode, PrintMode, VersionMode};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mode = cli.mode()?;
    let metadata = cli.metadata()?;
    let version = cli.version()?;
    let source = cli.source();
    let filter_fn: Vec<String> = cli
        .filter_fn
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if cli.input_path.is_none() && cli.cmd.is_none() {
        bail!("provide an INPUT_PATH or --cmd");
    }
    if cli.input_path.is_some() && cli.cmd.is_some() {
        bail!("provide either an INPUT_PATH or --cmd, not both");
    }

    let use_tui = if cli.classic {
        false
    } else if cli.tui {
        true
    } else {
        std::io::stdout().is_terminal()
    };

    if use_tui {
        return tui::run(tui::TuiOptions {
            input_path: cli.input_path,
            cmd: cli.cmd,
            filter_fn,
            metadata,
            version,
            source,
            start_diff: mode == PrintMode::Diff,
        });
    }

    let raw = load_raw(cli.input_path.as_deref(), cli.cmd.as_deref())?;
    let cleaned = parse::strip_ansi(&raw);
    let irs = parse::parse(&cleaned);
    if irs.is_empty() {
        let src = cli
            .input_path
            .as_deref()
            .or(cli.cmd.as_deref())
            .unwrap_or("input");
        bail!("no `// IR:` dumps found in {src}");
    }
    classic::run(
        &irs, &cleaned, &filter_fn, mode, metadata, version, source,
    )
}

fn load_raw(path: Option<&str>, cmd: Option<&str>) -> Result<String> {
    if let Some(cmd) = cmd {
        parse::capture_shell(cmd)
    } else if let Some(path) = path {
        std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("failed to read {path}: {e}"))
    } else {
        bail!("no input")
    }
}

/// Browse or print Sway IR dumps from `forc build --ir all`.
#[derive(Debug, Parser)]
#[command(
    about = "Browse or print Sway IR dumps from `forc build --ir all`",
    after_help = "TUI keys: F5 reload · y copy · Ctrl-C kill/quit · Tab focus · / search · p/f filters · d diff · m metadata · v version · ? help · q quit"
)]
struct Cli {
    /// Path to an IR dump file (output of `forc build --ir all`).
    /// Mutually exclusive with `--cmd`.
    input_path: Option<String>,

    /// Shell command to spawn; stdout and stderr are captured and parsed as the
    /// IR dump. No input file is needed. In the TUI, press F5 to re-run.
    #[arg(long = "cmd", short = 'c')]
    cmd: Option<String>,

    /// Force the classic (non-interactive) printer.
    #[arg(long)]
    classic: bool,

    /// Force the interactive TUI.
    #[arg(long)]
    tui: bool,

    /// Filter in functions whose name contains one of the given substrings
    /// (comma separated).
    #[arg(long, value_delimiter = ',')]
    filter_fn: Vec<String>,

    /// Comma separated list of items to print / TUI defaults.
    #[arg(long, value_delimiter = ',')]
    print: Vec<PrintItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, clap::ValueEnum)]
enum PrintItem {
    Ir,
    Diff,
    WithMd,
    WithoutMd,
    WithVersion,
    WithoutVersion,
    /// Print the original source code on top of each IR line that has metadata.
    Source,
}

impl Cli {
    fn mode(&self) -> Result<PrintMode> {
        let mut explicit = None::<PrintMode>;
        for item in &self.print {
            match item {
                PrintItem::Ir => {
                    if explicit == Some(PrintMode::Diff) {
                        bail!("--print items 'ir' and 'diff' are mutually exclusive");
                    }
                    explicit = Some(PrintMode::Ir);
                }
                PrintItem::Diff => {
                    if explicit == Some(PrintMode::Ir) {
                        bail!("--print items 'ir' and 'diff' are mutually exclusive");
                    }
                    explicit = Some(PrintMode::Diff);
                }
                _ => {}
            }
        }
        Ok(explicit.unwrap_or_default())
    }

    fn metadata(&self) -> Result<MdMode> {
        let mut metadata = MdMode::default();
        for item in &self.print {
            match item {
                PrintItem::WithMd => {
                    if metadata == MdMode::Without {
                        bail!("--print items 'with-md' and 'without-md' are mutually exclusive");
                    }
                    metadata = MdMode::With;
                }
                PrintItem::WithoutMd => {
                    if metadata == MdMode::With {
                        bail!("--print items 'with-md' and 'without-md' are mutually exclusive");
                    }
                    metadata = MdMode::Without;
                }
                _ => {}
            }
        }
        Ok(metadata)
    }

    fn version(&self) -> Result<VersionMode> {
        let mut version = VersionMode::default();
        for item in &self.print {
            match item {
                PrintItem::WithVersion => {
                    if version == VersionMode::Without {
                        bail!(
                            "--print items 'with-version' and 'without-version' are mutually exclusive"
                        );
                    }
                    version = VersionMode::With;
                }
                PrintItem::WithoutVersion => {
                    if version == VersionMode::With {
                        bail!(
                            "--print items 'with-version' and 'without-version' are mutually exclusive"
                        );
                    }
                    version = VersionMode::Without;
                }
                _ => {}
            }
        }
        Ok(version)
    }

    fn source(&self) -> bool {
        self.print
            .iter()
            .any(|item| matches!(item, PrintItem::Source))
    }
}

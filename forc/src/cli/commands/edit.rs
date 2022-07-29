use clap::{Parser, Args};
use anyhow::Result;
// A utility for managing cargo dependencies from the command line.

#[derive(Debug, Parser)]
#[clap(bin_name = "forc")]
pub enum Command {
    Add(crate::add::AddArgs),
}

impl Command {
    pub fn exec(self) -> Result<()> {
        match self {
            Self::Add(add) => add.exec(),
        }
    }
}

#![allow(clippy::bool_assert_comparison)]
/// Add dependencies to a Cargo.toml manifest file.
#[derive(Debug, Args)]
#[clap(version)]
#[clap(setting = clap::AppSettings::DeriveDisplayOrder)]
#[clap(after_help = "\
Examples:
  $ cargo add regex --build
  $ cargo add trycmd --dev
  $ cargo add ./crate/parser/
  $ cargo add serde +derive serde_json
")]
#[clap(override_usage = "\
    cargo add [OPTIONS] <DEP>[@<VERSION>] [+<FEATURE>,...] ...
    cargo add [OPTIONS] <DEP_PATH> [+<FEATURE>,...] ...")]
pub struct AddArgs {
    /// Reference to a package to add as a dependency
    ///
    /// You can reference a packages by:{n}
    /// - `<name>`, like `cargo add serde` (latest version will be used){n}
    /// - `<name>@<version-req>`, like `cargo add serde@1` or `cargo add serde@=1.0.38`{n}
    /// - `<path>`, like `cargo add ./crates/parser/`
    ///
    /// Additionally, you can specify features for a dependency by following it with a
    /// `+<FEATURE>`.
    #[clap(value_name = "DEP_ID")]
    pub crates: Vec<String>,

    /// Disable the default features
    #[clap(long)]
    no_default_features: bool,
    /// Re-enable the default features
    #[clap(long, overrides_with = "no-default-features")]
    default_features: bool,

    /// Space-separated list of features to add
    ///
    /// Alternatively, you can specify features for a dependency by following it with a
    /// `+<FEATURE>`.
    #[clap(short = 'F', long)]
    pub features: Option<Vec<String>>,

    /// Mark the dependency as optional
    ///
    /// The package name will be exposed as feature of your crate.
    #[clap(long, conflicts_with = "dev")]
    pub optional: bool,

    /// Mark the dependency as required
    ///
    /// The package will be removed from your features.
    #[clap(long, conflicts_with = "dev", overrides_with = "optional")]
    pub no_optional: bool,

    /// Rename the dependency
    ///
    /// Example uses:{n}
    /// - Depending on multiple versions of a crate{n}
    /// - Depend on crates with the same name from different registries
    #[clap(long, short)]
    pub rename: Option<String>,

    /// Package registry for this dependency
    #[clap(long, conflicts_with = "git")]
    pub registry: Option<String>,

    /// Add as development dependency
    ///
    /// Dev-dependencies are not used when compiling a package for building, but are used for compiling tests, examples, and benchmarks.
    ///
    /// These dependencies are not propagated to other packages which depend on this package.
    #[clap(short = 'D', long, help_heading = "SECTION", group = "section")]
    pub dev: bool,

    /// Add as build dependency
    ///
    /// Build-dependencies are the only dependencies available for use by build scripts (`build.rs`
    /// files).
    #[clap(short = 'B', long, help_heading = "SECTION", group = "section")]
    pub build: bool,

    /// Add as dependency to the given target platform.
    #[clap(
        long,
        forbid_empty_values = true,
        help_heading = "SECTION",
        group = "section"
    )]
    pub target: Option<String>,

    /// Path to `Cargo.toml`
    #[clap(long, value_name = "PATH", parse(from_os_str))]
    pub manifest_path: Option<std::path::PathBuf>,

    /// Package to modify
    #[clap(short = 'p', long = "package", value_name = "PKGID")]
    pub pkgid: Option<String>,

    /// Run without accessing the network
    #[clap(long)]
    pub offline: bool,

    /// Don't actually write the manifest
    #[clap(long)]
    pub dry_run: bool,

    /// Do not print any output in case of success.
    #[clap(long)]
    pub quiet: bool,

    /// Git repository location
    ///
    /// Without any other information, cargo will use latest commit on the main branch.
    #[clap(long, value_name = "URI", help_heading = "UNSTABLE")]
    pub git: Option<String>,

    /// Git branch to download the crate from.
    #[clap(
        long,
        value_name = "BRANCH",
        help_heading = "UNSTABLE",
        requires = "git",
        group = "git-ref"
    )]
    pub branch: Option<String>,

    /// Git tag to download the crate from.
    #[clap(
        long,
        value_name = "TAG",
        help_heading = "UNSTABLE",
        requires = "git",
        group = "git-ref"
    )]
    pub tag: Option<String>,

    /// Git reference to download the crate from
    ///
    /// This is the catch all, handling hashes to named references in remote repositories.
    #[clap(
        long,
        value_name = "REV",
        help_heading = "UNSTABLE",
        requires = "git",
        group = "git-ref"
    )]
    pub rev: Option<String>,
}

impl AddArgs {
    pub fn exec(self) -> CargoResult<()> {
        anyhow::bail!(
            "`cargo add` has been merged into cargo 1.62+ as of cargo-edit 0.10, either
- Upgrade cargo, like with `rustup update`
- Downgrade `cargo-edit`, like with `cargo install cargo-edit --version 0.9.1`"
        );
    }
}

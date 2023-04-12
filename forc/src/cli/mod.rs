use std::str::FromStr;

use self::commands::{
    addr2line, build, check, clean, completions, contract_id, init, new, parse_bytecode, plugins,
    predicate_root, template, test, update,
};
use addr2line::Command as Addr2LineCommand;
use anyhow::{anyhow, Result};
pub use build::Command as BuildCommand;
pub use check::Command as CheckCommand;
use clap::{Parser, Subcommand};
pub use clean::Command as CleanCommand;
pub use completions::Command as CompletionsCommand;
pub(crate) use contract_id::Command as ContractIdCommand;
use forc_tracing::{init_tracing_subscriber, TracingSubscriberOptions};
pub use init::Command as InitCommand;
pub use new::Command as NewCommand;
use parse_bytecode::Command as ParseBytecodeCommand;
pub use plugins::Command as PluginsCommand;
pub(crate) use predicate_root::Command as PredicateRootCommand;
pub use template::Command as TemplateCommand;
pub use test::Command as TestCommand;
use tracing::metadata::LevelFilter;
pub use update::Command as UpdateCommand;

mod commands;
mod plugin;
pub mod shared;

#[derive(Debug, Parser)]
#[clap(name = "forc", about = "Fuel Orchestrator", version)]
struct Opt {
    /// The command to run
    #[clap(subcommand)]
    command: Forc,

    /// Use verbose output
    #[clap(short, long, parse(from_occurrences), global = true)]
    verbose: u8,

    /// Silence all output
    #[clap(short, long, global = true)]
    silent: bool,

    /// Set the log level
    #[clap(short='L', long, global = true, parse(try_from_str = LevelFilter::from_str))]
    log_level: Option<LevelFilter>,
}

#[derive(Subcommand, Debug)]
enum Forc {
    #[clap(name = "addr2line")]
    Addr2Line(Addr2LineCommand),
    #[clap(visible_alias = "b")]
    Build(BuildCommand),
    Check(CheckCommand),
    Clean(CleanCommand),
    Completions(CompletionsCommand),
    New(NewCommand),
    Init(InitCommand),
    ParseBytecode(ParseBytecodeCommand),
    #[clap(visible_alias = "t")]
    Test(TestCommand),
    Update(UpdateCommand),
    Plugins(PluginsCommand),
    Template(TemplateCommand),
    ContractId(ContractIdCommand),
    PredicateRoot(PredicateRootCommand),
    /// This is a catch-all for unknown subcommands and their arguments.
    ///
    /// When we receive an unknown subcommand, we check for a plugin exe named
    /// `forc-<unknown-subcommand>` and try to execute it:
    ///
    /// ```ignore
    /// forc-<unknown-subcommand> <args>
    /// ```
    #[clap(external_subcommand)]
    Plugin(Vec<String>),
}

pub async fn run_cli() -> Result<()> {
    let opt = Opt::parse();
    let tracing_options = TracingSubscriberOptions {
        verbosity: Some(opt.verbose),
        silent: Some(opt.silent),
        log_level: opt.log_level,
        ansi: Some(true),
        ..Default::default()
    };

    init_tracing_subscriber(tracing_options);

    match opt.command {
        Forc::Addr2Line(command) => addr2line::exec(command),
        Forc::Build(command) => build::exec(command),
        Forc::Check(command) => check::exec(command),
        Forc::Clean(command) => clean::exec(command),
        Forc::Completions(command) => completions::exec(command),
        Forc::Init(command) => init::exec(command),
        Forc::New(command) => new::exec(command),
        Forc::ParseBytecode(command) => parse_bytecode::exec(command),
        Forc::Plugins(command) => plugins::exec(command),
        Forc::Test(command) => test::exec(command),
        Forc::Update(command) => update::exec(command).await,
        Forc::Template(command) => template::exec(command),
        Forc::ContractId(command) => contract_id::exec(command),
        Forc::PredicateRoot(command) => predicate_root::exec(command),
        Forc::Plugin(args) => {
            let output = plugin::execute_external_subcommand(args)?;
            let code = output
                .status
                .code()
                .ok_or_else(|| anyhow!("plugin exit status unknown"))?;
            std::process::exit(code);
        }
    }
}

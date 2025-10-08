use self::commands::{
    add, addr2line, build, check, clean, completions, contract_id, init, new, parse_bytecode,
    plugins, predicate_root, remove, template, test, update,
};
pub use add::Command as AddCommand;
use addr2line::Command as Addr2LineCommand;
use anyhow::anyhow;
pub use build::Command as BuildCommand;
pub use check::Command as CheckCommand;
use clap::{Parser, Subcommand};
pub use clean::Command as CleanCommand;
pub use completions::Command as CompletionsCommand;
pub(crate) use contract_id::Command as ContractIdCommand;
use forc_tracing::{init_tracing_subscriber, TracingSubscriberOptions};
use forc_util::ForcResult;
pub use init::Command as InitCommand;
pub use new::Command as NewCommand;
use parse_bytecode::Command as ParseBytecodeCommand;
pub use plugins::Command as PluginsCommand;
pub(crate) use predicate_root::Command as PredicateRootCommand;
pub use remove::Command as RemoveCommand;
use std::str::FromStr;
pub use template::Command as TemplateCommand;
pub use test::Command as TestCommand;
use tracing::metadata::LevelFilter;
pub use update::Command as UpdateCommand;

mod commands;
mod plugin;
pub mod shared;

fn help() -> &'static str {
    Box::leak(
        format!(
            "Examples:\n{}{}{}{}",
            plugins::examples(),
            test::examples(),
            build::examples(),
            check::examples(),
        )
        .trim_end()
        .to_string()
        .into_boxed_str(),
    )
}

#[derive(Debug, Parser)]
#[clap(name = "forc", about = "Fuel Orchestrator", version, after_long_help = help())]
struct Opt {
    /// The command to run
    #[clap(subcommand)]
    command: Forc,

    /// Use verbose output
    #[clap(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    /// Silence all output
    #[clap(short, long, global = true)]
    silent: bool,

    /// Set the log level
    #[clap(short='L', long, global = true, value_parser = LevelFilter::from_str)]
    log_level: Option<LevelFilter>,
}

#[derive(Subcommand, Debug)]
enum Forc {
    Add(AddCommand),
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
    Remove(RemoveCommand),
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

impl Forc {
    #[allow(dead_code)]
    pub fn possible_values() -> Vec<&'static str> {
        vec![
            "add",
            "addr2line",
            "build",
            "check",
            "clean",
            "completions",
            "init",
            "new",
            "parse-bytecode",
            "plugins",
            "test",
            "update",
            "template",
            "remove",
            "contract-id",
            "predicate-root",
        ]
    }
}

pub async fn run_cli() -> ForcResult<()> {
    let opt = Opt::parse();
    let tracing_options = TracingSubscriberOptions {
        verbosity: Some(opt.verbose),
        silent: Some(opt.silent),
        log_level: opt.log_level,
        ..Default::default()
    };

    init_tracing_subscriber(tracing_options);

    match opt.command {
        Forc::Add(command) => add::exec(command),
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
        Forc::Update(command) => update::exec(command),
        Forc::Remove(command) => remove::exec(command),
        Forc::Template(command) => template::exec(command),
        Forc::ContractId(command) => contract_id::exec(command),
        Forc::PredicateRoot(command) => predicate_root::exec(command),
        Forc::Plugin(args) => {
            let output = plugin::execute_external_subcommand(&args)?;
            let code = output
                .status
                .code()
                .ok_or_else(|| anyhow!("plugin exit status unknown"))?;
            std::process::exit(code);
        }
    }
}

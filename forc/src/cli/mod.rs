use self::commands::{
    addr2line, build, check, clean, completions, init, json_abi, new, parse_bytecode, plugins,
    template, test, update,
};
use addr2line::Command as Addr2LineCommand;
use anyhow::{anyhow, Result};
pub use build::Command as BuildCommand;
pub use check::Command as CheckCommand;
use clap::Parser;
pub use clean::Command as CleanCommand;
pub use completions::Command as CompletionsCommand;
pub use init::Command as InitCommand;
pub use json_abi::Command as JsonAbiCommand;
pub use new::Command as NewCommand;
use parse_bytecode::Command as ParseBytecodeCommand;
pub use plugins::Command as PluginsCommand;
pub use template::Command as TemplateCommand;
use test::Command as TestCommand;
pub use update::Command as UpdateCommand;

mod commands;
mod plugin;

#[derive(Debug, Parser)]
#[clap(name = "forc", about = "Fuel Orchestrator", version)]
struct Opt {
    /// the command to run
    #[clap(subcommand)]
    command: Forc,
}

#[derive(Debug, Parser)]
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
    JsonAbi(JsonAbiCommand),
    Plugins(PluginsCommand),
    Template(TemplateCommand),
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
        Forc::JsonAbi(command) => json_abi::exec(command),
        Forc::Template(command) => template::exec(command),
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

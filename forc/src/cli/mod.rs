use self::commands::{
    addr2line, build, clean, completions, deploy, init, json_abi, parse_bytecode, plugins, run,
    test, update,
};
use addr2line::Command as Addr2LineCommand;
use anyhow::{anyhow, Result};
pub use build::Command as BuildCommand;
use clap::Parser;
pub use clean::Command as CleanCommand;
pub use completions::Command as CompletionsCommand;
pub use deploy::Command as DeployCommand;
pub use init::Command as InitCommand;
pub use json_abi::Command as JsonAbiCommand;
use parse_bytecode::Command as ParseBytecodeCommand;
pub use plugins::Command as PluginsCommand;
pub use run::Command as RunCommand;
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
    Clean(CleanCommand),
    #[clap(after_help = completions::COMPLETIONS_HELP)]
    Completions(CompletionsCommand),
    Deploy(DeployCommand),
    Init(InitCommand),
    ParseBytecode(ParseBytecodeCommand),
    Run(RunCommand),
    #[clap(visible_alias = "t")]
    Test(TestCommand),
    Update(UpdateCommand),
    JsonAbi(JsonAbiCommand),
    Plugins(PluginsCommand),
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
        Forc::Clean(command) => clean::exec(command),
        Forc::Completions(command) => completions::exec(command),
        Forc::Deploy(command) => deploy::exec(command).await,
        Forc::Init(command) => init::exec(command),
        Forc::ParseBytecode(command) => parse_bytecode::exec(command),
        Forc::Plugins(command) => plugins::exec(command),
        Forc::Run(command) => run::exec(command).await,
        Forc::Test(command) => test::exec(command),
        Forc::Update(command) => update::exec(command).await,
        Forc::JsonAbi(command) => json_abi::exec(command),
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

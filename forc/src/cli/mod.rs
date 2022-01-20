use structopt::StructOpt;

mod commands;
use self::commands::{
    build, deploy, explorer, format, init, json_abi, lsp, parse_bytecode, run, test, update,
};

pub use build::Command as BuildCommand;
pub use deploy::Command as DeployCommand;
pub use explorer::Command as ExplorerCommand;
pub use format::Command as FormatCommand;
use init::Command as InitCommand;
pub use json_abi::Command as JsonAbiCommand;
use lsp::Command as LspCommand;
use parse_bytecode::Command as ParseBytecodeCommand;
pub use run::Command as RunCommand;
use test::Command as TestCommand;
pub use update::Command as UpdateCommand;

#[derive(Debug, StructOpt)]
#[structopt(name = "forc", about = "Fuel Orchestrator")]
struct Opt {
    /// the command to run
    #[structopt(subcommand)]
    command: Forc,
}

#[derive(Debug, StructOpt)]
enum Forc {
    Build(BuildCommand),
    Deploy(DeployCommand),
    Explorer(ExplorerCommand),
    #[structopt(name = "fmt")]
    Format(FormatCommand),
    Init(InitCommand),
    ParseBytecode(ParseBytecodeCommand),
    Run(RunCommand),
    Test(TestCommand),
    Update(UpdateCommand),
    JsonAbi(JsonAbiCommand),
    Lsp(LspCommand),
}

pub(crate) async fn run_cli() -> Result<(), String> {
    let opt = Opt::from_args();
    match opt.command {
        Forc::Build(command) => build::exec(command),
        Forc::Deploy(command) => deploy::exec(command).await,
        Forc::Explorer(command) => explorer::exec(command).await,
        Forc::Format(command) => format::exec(command),
        Forc::Init(command) => init::exec(command),
        Forc::ParseBytecode(command) => parse_bytecode::exec(command),
        Forc::Run(command) => run::exec(command).await,
        Forc::Test(command) => test::exec(command),
        Forc::Update(command) => update::exec(command).await,
        Forc::JsonAbi(command) => json_abi::exec(command),
        Forc::Lsp(command) => lsp::exec(command).await,
    }
}

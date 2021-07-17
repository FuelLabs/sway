use structopt::StructOpt;

mod commands;
use self::commands::{build, deploy, format, init, parse_bytecode, run, test, update};

pub use build::Command as BuildCommand;
pub use deploy::Command as DeployCommand;
pub use format::Command as FormatCommand;
use init::Command as InitCommand;
use parse_bytecode::Command as ParseBytecodeCommand;
pub use run::Command as RunCommand;
use test::Command as TestCommand;
pub use update::Command as UpdateCommand;

#[derive(Debug, StructOpt)]
#[structopt(name = "forc", about = "Fuel HLL Orchestrator")]
struct Opt {
    /// the command to run
    #[structopt(subcommand)]
    command: Forc,
}

#[derive(Debug, StructOpt)]
enum Forc {
    Build(BuildCommand),
    #[structopt(name = "fmt")]
    Format(FormatCommand),
    Deploy(DeployCommand),
    Init(InitCommand),
    Run(RunCommand),
    Test(TestCommand),
    ParseBytecode(ParseBytecodeCommand),
    Update(UpdateCommand),
}

pub(crate) async fn run_cli() -> Result<(), String> {
    let opt = Opt::from_args();
    match opt.command {
        Forc::Build(command) => build::exec(command),
        Forc::Format(command) => format::exec(command),
        Forc::Deploy(command) => deploy::exec(command).await,
        Forc::Init(command) => init::exec(command),
        Forc::Run(command) => run::exec(command).await,
        Forc::Test(command) => test::exec(command),
        Forc::ParseBytecode(command) => parse_bytecode::exec(command),
        Forc::Update(command) => update::exec(command).await,
    }?;

    Ok(())
}

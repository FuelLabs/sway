use structopt::StructOpt;

mod commands;
use self::commands::{
    analysis, benchmark, build, coverage, deploy, format, init, parse_bytecode, publish, run,
    serve, test, update,
};

use analysis::Command as AnalysisCommand;
use benchmark::Command as BenchmarkCommand;
pub use build::Command as BuildCommand;
use coverage::Command as CoverageCommand;
pub use deploy::Command as DeployCommand;
pub use format::Command as FormatCommand;
use init::Command as InitCommand;
use parse_bytecode::Command as ParseBytecodeCommand;
use publish::Command as PublishCommand;
pub use run::Command as RunCommand;
use serve::Command as ServeCommand;
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
    Analysis(AnalysisCommand),
    Benchmark(BenchmarkCommand),
    Build(BuildCommand),
    Coverage(CoverageCommand),

    #[structopt(name = "fmt")]
    Format(FormatCommand),
    Deploy(DeployCommand),
    Init(InitCommand),
    Run(RunCommand),
    Publish(PublishCommand),
    Serve(ServeCommand),
    Test(TestCommand),
    ParseBytecode(ParseBytecodeCommand),
    Update(UpdateCommand),
}

pub(crate) async fn run_cli() -> Result<(), String> {
    let opt = Opt::from_args();
    match opt.command {
        Forc::Analysis(command) => analysis::exec(command),
        Forc::Benchmark(command) => benchmark::exec(command),
        Forc::Build(command) => build::exec(command),
        Forc::Coverage(command) => coverage::exec(command),
        Forc::Format(command) => format::exec(command),
        Forc::Deploy(command) => deploy::exec(command).await,
        Forc::Init(command) => init::exec(command),
        Forc::Run(command) => run::exec(command).await,
        Forc::Publish(command) => publish::exec(command),
        Forc::Serve(command) => serve::exec(command),
        Forc::Test(command) => test::exec(command),
        Forc::ParseBytecode(command) => parse_bytecode::exec(command),
        Forc::Update(command) => update::exec(command).await,
    }?;

    Ok(())
}

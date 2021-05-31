use structopt::StructOpt;

mod commands;
use self::commands::{analysis, benchmark, build, coverage, deploy, init, publish, serve, test, mvprun};

use analysis::Command as AnalysisCommand;
use benchmark::Command as BenchmarkCommand;
use build::Command as BuildCommand;
use coverage::Command as CoverageCommand;
use deploy::Command as DeployCommand;
use init::Command as InitCommand;
use publish::Command as PublishCommand;
use serve::Command as ServeCommand;
use test::Command as TestCommand;
use mvprun::Command as MvprunCommand;

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
    Deploy(DeployCommand),
    Init(InitCommand),
    Mvprun(MvprunCommand),
    Publish(PublishCommand),
    Serve(ServeCommand),
    Test(TestCommand),
}

pub(crate) fn run_cli() -> Result<(), String> {
    let opt = Opt::from_args();
    match opt.command {
        Forc::Analysis(command) => analysis::exec(command),
        Forc::Benchmark(command) => benchmark::exec(command),
        Forc::Build(command) => build::exec(command),
        Forc::Coverage(command) => coverage::exec(command),
        Forc::Deploy(command) => deploy::exec(command),
        Forc::Init(command) => init::exec(command),
        Forc::Mvprun(command) => mvprun::exec(command),
        Forc::Publish(command) => publish::exec(command),
        Forc::Serve(command) => serve::exec(command),
        Forc::Test(command) => test::exec(command),
    }?;
    /*
    let content = fs::read_to_string(opt.input.clone())?;

    let res = compile(&content);

    */

    Ok(())
}

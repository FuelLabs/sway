use structopt::StructOpt;

mod commands;
use self::commands::{analysis, benchmark, build, coverage, deploy, init, publish, serve, test};

#[derive(Debug, StructOpt)]
#[structopt(name = "forc", about = "Fuel HLL Orchestrator")]
struct Opt {
    /// the command to run
    #[structopt(subcommand)]
    command: Forc,
}

#[derive(Debug, StructOpt)]
enum Forc {
    Analysis(analysis::Command),
    Benchmark(benchmark::Command),
    Build(build::Command),
    Coverage(coverage::Command),
    Deploy(deploy::Command),
    Init(init::Command),
    Publish(publish::Command),
    Serve(serve::Command),
    Test(test::Command),
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

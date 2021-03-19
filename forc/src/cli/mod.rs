use structopt::StructOpt;

mod build;
mod init;

#[derive(Debug, StructOpt)]
#[structopt(name = "forc", about = "Fuel HLL Orchestrator")]
struct Opt {
    /// the command to run
    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "init")]
    Init { project_name: String },
    #[structopt(name = "build")]
    Build {
        #[structopt(short = "p")]
        path: Option<String>,
    },
}

pub(crate) fn run_cli() -> Result<(), String> {
    let opt = Opt::from_args();
    match opt.command {
        Command::Init { project_name } => {
            init::init_new_project(project_name).map_err(|e| e.to_string())
        }
        Command::Build { path } => build::build(path),
    }?;
    /*
    let content = fs::read_to_string(opt.input.clone())?;

    let res = compile(&content);

    */

    Ok(())
}

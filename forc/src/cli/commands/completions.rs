use clap::Command as ClapCommand;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Generator, Shell};

/// If provided, outputs the completion file for given shell
#[derive(Debug, Parser)]
pub struct Command {
    #[clap(long = "generate", short, arg_enum)]
    generator: Shell,
}

pub(crate) fn exec(command: Command) -> Result<(), String> {
    let mut cmd = super::super::Opt::command();
    println!("Generating completion file for {:?}...", command.generator);
    print_completions(command.generator, &mut cmd);
    Ok(())
}

fn print_completions<G: Generator>(gen: G, cmd: &mut ClapCommand) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}

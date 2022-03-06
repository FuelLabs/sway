use anyhow::Result;
use clap::Command as ClapCommand;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Generator, Shell};

pub static COMPLETIONS_HELP: &str = r#"
DISCUSSION:
    Enable tab completion for Bash, Fish, Zsh, or PowerShell
    The script is output on `stdout`, allowing one to re-direct the
    output to the file of their choosing. Where you place the file
    will depend on which shell, and which operating system you are
    using. Your particular configuration may also determine where
    these scripts need to be placed.

    Here are some common set ups for the three supported shells under
    Unix and similar operating systems (such as GNU/Linux).

    BASH:

    Completion files are commonly stored in `/etc/bash_completion.d/` for
    system-wide commands, but can be stored in
    `~/.local/share/bash-completion/completions` for user-specific commands.
    Run the command:

        $ mkdir -p ~/.local/share/bash-completion/completions
        $ forc completions --shell=bash >> ~/.local/share/bash-completion/completions/forc

    This installs the completion script. You may have to log out and
    log back in to your shell session for the changes to take effect.

    BASH (macOS/Homebrew):

    Homebrew stores bash completion files within the Homebrew directory.
    With the `bash-completion` brew formula installed, run the command:

        $ mkdir -p $(brew --prefix)/etc/bash_completion.d
        $ forc completions --shell=bash > $(brew --prefix)/etc/bash_completion.d/forc.bash-completion

    FISH:

    Fish completion files are commonly stored in
    `$HOME/.config/fish/completions`. Run the command:

        $ mkdir -p ~/.config/fish/completions
        $ forc completions --shell=fish > ~/.config/fish/completions/forc.fish

    This installs the completion script. You may have to log out and
    log back in to your shell session for the changes to take effect.

    ZSH:

    ZSH completions are commonly stored in any directory listed in
    your `$fpath` variable. To use these completions, you must either
    add the generated script to one of those directories, or add your
    own to this list.

    Adding a custom directory is often the safest bet if you are
    unsure of which directory to use. First create the directory; for
    this example we'll create a hidden directory inside our `$HOME`
    directory:

        $ mkdir ~/.zfunc

    Then add the following lines to your `.zshrc` just before
    `compinit`:

        fpath+=~/.zfunc

    Now you can install the completions script using the following
    command:

        $ forc completions --shell=zsh > ~/.zfunc/_forc

    You must then either log out and log back in, or simply run

        $ exec zsh

    for the new completions to take effect.

    CUSTOM LOCATIONS:

    Alternatively, you could save these files to the place of your
    choosing, such as a custom directory inside your $HOME. Doing so
    will require you to add the proper directives, such as `source`ing
    inside your login script. Consult your shells documentation for
    how to add such directives.

    POWERSHELL:

    The powershell completion scripts require PowerShell v5.0+ (which
    comes with Windows 10, but can be downloaded separately for windows 7
    or 8.1).

    First, check if a profile has already been set

        PS C:\> Test-Path $profile

    If the above command returns `False` run the following

        PS C:\> New-Item -path $profile -type file -force

    Now open the file provided by `$profile` (if you used the
    `New-Item` command it will be
    `${env:USERPROFILE}\Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1`

    Next, we either save the completions file into our profile, or
    into a separate file and source it inside our profile. To save the
    completions into our profile simply use

        PS C:\> forc completions --shell=powershell >>
${env:USERPROFILE}\Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1
"#;

/// Generate tab-completion scripts for your shell
#[derive(Debug, Parser)]
pub struct Command {
    #[clap(
        short,
        long,
        help("[possible values: zsh, bash, fish, powershell, elvish]")
    )]
    shell: Shell,
}

pub(crate) fn exec(command: Command) -> Result<()> {
    let mut cmd = super::super::Opt::command();
    print_completions(command.shell, &mut cmd);
    Ok(())
}

fn print_completions<G: Generator>(gen: G, cmd: &mut ClapCommand) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}

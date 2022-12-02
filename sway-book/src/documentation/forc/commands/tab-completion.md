# Tab Completion

[`forc`](../../forc/commands/index.md) supports a feature known as command-line completion which automatically fills in partially typed commands.

After executing one of the commands below close and open a new terminal for the completions to take effect.

For up to date instructions refer to `forc completions --help`.

## Bash

```bash
forc completions --shell=bash > ~/.local/share/bash-completion/completions/forc

# Bash (macOS/Homebrew)
forc completions --shell=bash > $(brew --prefix)/etc/bash_completion.d/forc.bash-completion
```

## Fish

```bash
# Create the `completions` directory if it does not exist
mkdir -p ~/.config/fish/completions

forc completions --shell=fish > ~/.config/fish/completions/forc.fish
```

## Zsh

```bash
forc completions --shell=zsh > ~/.zfunc/_forc
```

## PowerShell

```bash
# PowerShell v5.0+
forc completions --shell=powershell >> $PROFILE.CurrentUserCurrentHost

# or
forc completions --shell=powershell | Out-String | Invoke-Expression
```

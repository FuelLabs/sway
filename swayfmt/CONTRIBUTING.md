# Contributing to the Sway Language Formatter

Firstly, thank you for taking interest in advancing the Sway language formatter! This guide will walk you through installation and best practices for contributing to this project.

> **Pre-Installation:** If you've previously installed `forc-fmt` via `fuelup`, you will need to uninstall it in order to use the binary compiled from source.

```sh
# find fuelup `forc-fmt` binary
which forc-fmt
# output: `~/.fuelup/bin/forc-fmt`
#
# remove fuelup `forc-fmt` binary
rm ~/.fuelup/bin/forc-fmt
```

## Installation

> **Note:** `cargo` is a prerequisite to this build.

```sh
# 1. move to your preferred directory
#    example: cd ~/Code/
#
# 2. clone the Sway repo
git clone https://github.com/FuelLabs/sway.git
#
# 3. build from manifest and move the compiled result to your `.cargo/bin` folder
cargo build --path ~/sway/forc-plugins/forc-fmt/Cargo.toml && mv ~/sway/target/debug/forc-fmt ~/.cargo/bin
```

## Testing

Be sure to have `forc` installed then move into a Sway project directory and execute the binary:

```sh
forc fmt
```

## Contribution Guidelines

**Issues:**

- Please check existing issues before opening a new ticket.
- If a bug or feature you would like to see implemented isn't represented by an issue please use our issue template and submit all relevant details.
  > **Note**: please keep in mind, it is not the goal of `swayfmt` to be identically configurable to `rustfmt` and that some requested features may remain unimplemented until a consensus is reached on adding them

**Pull Requests:**

- Each [issue should be linked to its corresponding pull request](https://docs.github.com/en/issues/tracking-your-work-with-issues/linking-a-pull-request-to-an-issue) with either a closing or reference keyword.
- If not enough context is provided by an issue please request a more detailed explanation or look to `rustfmt` as a definitive reference.
- When taking on a task leave a comment so that a member can assign you. This prevents multiple people from taking on the same work.
- If you are implementing a new feature, or fixing a bug please provide unit tests to show the effectiveness of the changes you've made and follow the formatting guidelines of the test cases currently available.
- Adjustments to formatted whitespace, or adding `char`s should always be behind a `const` if not provided by the `sway_ast`.
- Lastly, keep in mind that we aim to avoid unnecessary memory reallocations e.g. `String::new()` or `.clone()`, destructive operations such as `.pop()`, and prefer using `std::fmt::Write` macros for appending `FormattedCode`.

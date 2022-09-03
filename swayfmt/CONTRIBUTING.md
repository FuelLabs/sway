# Contributing to the Sway Language Formatter

Firstly, thank you for taking interest in advancing the Sway language formatter! This guide will walk you through installation and best practices for contributing to this project.

## Installation

In order to see `swayfmt` in action you will need both `swayfmt` and the [`forc-fmt`](../forc-plugins/forc-fmt/) plugin. As previously stated, you can install these via [`fuelup`](https://github.com/FuelLabs/fuelup) which will install the complete Fuel toolchain, or follow the instructions below to install from source. If you've previously installed `fuelup`, you'll need to remove the binary before installing from source.

> **Note:** `cargo` is a prerequisite to this build.

```sh
# 1. move to your preferred directory
cd ~/Code/
# 2. clone the Sway repo
git clone https://github.com/FuelLabs/sway.git
# 3. build from manifest and move the compiled result to your `.cargo/bin` folder
cargo build --manifest-path ~/sway/forc-plugins/forc-fmt/Cargo.toml mv ~/sway/target/debug/forc-fmt ~/.cargo/bin
```

### Testing

Move to the `src` folder of your project, and execute the binary:

```sh
forc-fmt
```

Be sure that your code compiles before executing the command!

### Contribution Guidelines

**Issues:**

- Please check existing issues before opening a new ticket.
- If there is a bug that isn't represented by a ticket please provide the following details:
  - `forc` version (run the command `forc --version`)
  - `rust` version (run the command `rustc --version`)
  - operating system and version
  - copy and paste the following into separate and labeled code blocks:
    - error output, if any
    - the full code snippet the problem originated from
    - the current output after formatting
    - the desired output after formatting
  - optionally, a solution to the problem
- If a feature you would like to see implemented isn't represented by a current ticket please provide the following details:
  - a detailed description of the feature
  - an example of what the code looks like before being formatted and what the code would look like after formatting
    > **Note**: please keep in mind, it is not the goal of `swayfmt` to be identically configurable to `rustfmt` and that some requested features may remain unimplemented until a consensus is reached on adding them

**Pull Requests:**

- Each pull request should be linked to its corresponding issue with either a closing or reference keyword.
- If not enough context is provided by an issue please request a more detailed explanation or look to `rustfmt` as a definitive reference.
- When taking on a task leave a comment so that a member can assign you. This prevents multiple people from taking on the same work.
- Before committing changes be sure to run `cargo fmt`, `cargo clippy` and `cargo test` to ensure that CI will not fail. These are the most common to fail, however, all checks must pass and have two approving reviews in order to merge into master.
- If you are implementing a new feature, or fixing a bug please provide unit tests to show the effectiveness of the changes you've made and follow the formatting guidelines of the test cases currently available.
- Adjustments to formatted whitespace, or adding `char`s should always be behind a `const` if not provided by the `sway_ast`.
- Lastly, keep in mind that we aim to avoid unnecessary memory reallocations e.g. `String::new()` or `.clone()`, destructive operations such as `.pop()`, and prefer using `std::fmt::Write` macros for appending `FormattedCode`.

# Contributing to the Sway Language Formatter

Firstly, thank you for taking interest in advancing the Sway language formatter! This guide will walk you through installation and best practices for contributing to this project.

## Installation

In order to see `swayfmt` in action you will need both `swayfmt` and the [`forc-fmt`](../forc-plugins/forc-fmt/) plugin. As previously stated, you can install these via [`fuelup`](https://github.com/FuelLabs/fuelup) which will install the complete Fuel toolchain, or follow the below instructions to install from source.

**note:** `cargo` is a pre-requisite to this build

1. move to your preferred directory
2. clone the Sway repo
3. build from manifest and move the compiled result to your `.cargo/bin` folder

```sh
git clone https://github.com/FuelLabs/sway.git
cargo build --manifest-path ~/sway/forc-plugins/forc-fmt/Cargo.toml
mv ~/sway/target/debug/forc-fmt ~/.cargo/bin
```

### Testing

Simply run the `forc-fmt` command on the file you want to format:

```sh
forc fmt
```

Be sure that your code compiles before executing the command!

### Contribution Guidelines

**Issues:**

- Please check existing issues before opening a new ticket
- If a bug you find isn't represented by a current ticket, please provide the following details:
  - `forc-fmt` and `swayfmt` version
  - `rust` and `cargo` version
  - operating system and version
  - error output, if any
  - the full code snippet the problem originated from
  - the current output after formatting
  - the desired output after formatting
  - optionally, a solution to the problem
- If a feature you would like to see implemented isn't represented by a current ticket, please provide the following details:
  - a detailed description of the feature
  - a before and after of how the feature would format the code, if implemented
  - optionally, a solution
  - note: please keep in mind, it is not the goal of `swayfmt` to be identically configurable to `rustfmt` and that some requested features may remain unimplemented until a concensus is reached on adding them

**Pull Requests:**

- Each pull request should be linked to its corresponding issue with either a closing or reference keyword
- If not enough context is provided by an issue, please request a more detailed explanation or look to `rustfmt` as a difinitive reference
- If a change you are making is rather large, or will take ample time to complete please open or convert it to a draft until it is ready for review
- Before committing changes be sure to run `cargo fmt`, `cargo clippy` and `cargo test` to ensure that CI will not fail
- If you are implementing a new feature, or fixing a bug please provide unit tests to show the effectiveness of the changes you've made and follow the formatting guidelines of the test cases currently available
- Adjustments to formatted whitespace, or adding `char`s should always be behind a `const` if not provided by the `sway_ast`
- Lastly, keep in mind that we aim to avoid unnecessary memory reallocations e.g. `String::new()` or `.clone()`, destructive operations such as `.pop()`, and prefer using `std::fmt::Write` macros for appending `FormattedCode`

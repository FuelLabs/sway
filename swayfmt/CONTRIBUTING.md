# Contributing to the Sway Language Formatter

Firstly, thank you for taking interest in advancing the Sway language formatter! This guide will walk you through installation and best practices for contributing to this project.

## Installation

To start working on the formatter, you need the [Rust toolchain](https://www.rust-lang.org/tools/install).

Clone the Sway repo into your preferred directory and run the formatter tests to ensure everything works as expected before you make local changes:

```sh
# from /sway
cd swayfmt && cargo test
```

## Testing

You can either test that your changes work correctly by writing new tests or manually running the formatter.

### Writing new tests (Recommended)

`swayfmt` has an extensive test suite that should pass both locally and within the CI to ensure reliability. This is used
to ensure that there isn't regression introduced along with new changes. If your changes include fixing bugs or adding
a new feature, please also include new tests accompanying your PR where possible.

There are both isolated tests based on an item or expression and full-bodied tests based on source code in the codebase.

The first kind are found in `tests.rs` adjacent to the kind of expression or item file and rely on macros for setup.
For example, you may find the tests related to structs located next to the module that implements struct formatting.

The second kind are found within `tests` folder. These tests ensure that a full piece of source code is correctly parsed and
formatted.

You should look at existing tests for examples on how you may test your changes.

### Running the formatter

To manually run the formatter, you can create a dummy Sway file and execute the formatter from `cargo`:

```sh
# copy paste some Sway code into my_file.sw and run the formatter from cargo
cargo run --bin=forc-fmt my_file.sw
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

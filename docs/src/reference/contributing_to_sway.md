# Contributing To Sway

Thanks for your interest in contributing to Sway! This document outlines the process for installing and setting up the Sway toolchain for development, as well as some conventions on contributing to Sway.

If you run into any difficulties getting started, you can always ask questions on our [Discord](https://discord.gg/xfpK4Pe).

## Building and setting up a development workspace

See the [introduction](../introduction/index.md) section for instructions on installing and setting up the Sway toolchain.

## Getting the repository

1. Visit the [Sway](https://github.com/FuelLabs/sway) repo and fork the project.
2. Then clone your forked copy to your local machine and get to work.

```sh
git clone https://github.com/FuelLabs/sway
cd sway
```

## Building and testing

The following steps will run the sway test suite and ensure that everything is set up correctly.

First, open a new terminal and start `fuel-core` with:

```sh
fuel-core
```

Then open a second terminal, cd into the `sway` repo and run:

```sh
cargo run --bin test
```

After the test suite runs, you should see:

```console
Tests passed.
_n_ tests run (0 skipped)
```

Congratulations! You've now got everything setup and are ready to start making contributions.

## Finding something to work on

There are many ways in which you may contribute to the Sway project, some of which involve coding knowledge and some which do not. A few examples include:

- Reporting bugs
- Adding documentation to the Sway book
- Adding new features or bugfixes for which there is already an open issue
- Making feature requests

Check out our [Help Wanted](https://github.com/FuelLabs/sway/issues?q=is%3Aopen+is%3Aissue+label%3A%22help+wanted%22), [Sway Book](https://github.com/FuelLabs/sway/issues?q=is%3Aopen+is%3Aissue+label%3A%22The+Sway+Book%22) or [Good First Issue](https://github.com/FuelLabs/sway/issues?q=is%3Aopen+is%3Aissue+label%3A%22good+first+issue%22) issues to find a suitable task.

If you are planning something big, for example, related to multiple components or changes current behaviors, make sure to open an issue to discuss with us before starting on the implementation.

## Contribution flow

This is a rough outline of what a contributor's workflow looks like:

- Make sure what you want to contribute is already tracked as an issue.
  - We may discuss the problem and solution in the issue.
- Create a Git branch from where you want to base your work. This is usually master.
- Write code, add test cases, and commit your work.
- Run tests and make sure all tests pass.
- If the PR contains any breaking changes, add the breaking label to your PR.
- Push your changes to a branch in your fork of the repository and submit a pull request.
  - Make sure mention the issue, which is created at step 1, in the commit message.
- Your PR will be reviewed and some changes may be requested.
  - Once you've made changes, your PR must be re-reviewed and approved.
  - If the PR becomes out of date, you can use GitHub's 'update branch' button.
  - If there are conflicts, you can merge and resolve them locally. Then push to your PR branch.
    Any changes to the branch will require a re-review.
- Our CI system (Github Actions) automatically tests all authorized pull requests.
- Use Github to merge the PR once approved.

Thanks for your contributions!

### Linking issues

Pull requests should be linked to at least one issue in the same repo.

If the pull request resolves the relevant issues, and you want GitHub to close these issues automatically after it merged into the default branch, you can use the syntax (`KEYWORD #ISSUE-NUMBER`) like this:

```markdown
close #123
```

If the pull request links an issue but does not close it, you can use the keyword `ref` like this:

```markdown
ref #456
```

Multiple issues should use full syntax for each issue and separate by a comma, like:

```markdown
close #123, ref #456
```

## Debugging strategies

There are a number of valid debugging approaches when debugging in this repo.

Aside from adding print statements or using your own debugging approach, it may
be useful to use `rust-lldb` to debug stack overflow bugs. (`rust-lldb` comes
pre-installed with the `cargo` install).

Here is an example of how to use `rust-lldb`:

```bash
$ cargo build
Finished dev [unoptimized + debuginfo] target(s) in 2.98s
```

```bash
$ rust-lldb ./target/debug/forc -- build --verbose --time-phases --path <your Rust project path>
target create "./target/debug/forc"
Current executable set to < >.
settings set -- target.run-args  "build" "--verbose" "--time-phases" "--path" <your Rust project path>
```

```
(lldb) run
<where the stack overflow is coming from>
```

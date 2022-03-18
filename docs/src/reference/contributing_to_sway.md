# Contributing To Sway

Thanks for your interest in contributing to Sway! This document outlines the process for installing and setting up the Sway toolchain for development, as well as some conventions on contributing to Sway.

If you run into any difficulties getting started, you can always ask questions on our [Discord](https://discord.gg/xfpK4Pe).

## Building and setting up a development workspace

See the Sway book's [Introduction](../introduction/index.md) for instructions on installing and setting up the Sway toolchain..

## Getting the repository

```
git clone https://github.com/FuelLabs/sway
cd sway
```

## Building and testing

The following steps will run the sway test suite and ensure that everything is set up correctly.

First, open a new terminal and start fuel-core with:

```
fuel-core
```

Then, open a second terminal and run:

```
cd sway/test
cargo run
```

After the test suite runs, you should see:

> ---
>
> Tests passed.\
> _n_ tests run (0 skipped)

Congratulations! You've now got everything setup and are ready to start making contributions.

## Finding something to work on

There are many ways in which you may contribute to the Sway project, some of which involve coding knowledge and some which do not. A few examples include:

- Reporting bugs
- Adding documentation to the Sway book
- Adding new features or bugfixes for which there is already an open issue
- making Feature requests

Check out our [Help Wanted](https://github.com/FuelLabs/sway/issues?q=is%3Aopen+is%3Aissue+label%3A%22help+wanted%22), [Sway Book](https://github.com/FuelLabs/sway/issues?q=is%3Aopen+is%3Aissue+label%3A%22The+Sway+Book%22) or [Good First Issue](https://github.com/FuelLabs/sway/issues?q=is%3Aopen+is%3Aissue+label%3A%22good+first+issue%22) issues to find a suitable task.

If you are planning something big, for example, related to multiple components or changes current behaviors, make sure to open an issue to discuss with us before going on.

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

Pull Requests should be linked to at least one issue in the same repo.

If the pull request resolves the relevant issues, and you want GitHub to close these issues automatically after it merged into the default branch, you can use the syntax (`KEYWORD #ISSUE-NUMBER`) like this:

```
close #123
```

If the pull request links an issue but does not close it, you can use the keyword `ref` like this:

```
ref #456
```

Multiple issues should use full syntax for each issue and separate by a comma, like:

```
close #123, ref #456
```

# Rust + Sway integration testing

A cargo-generate template that makes it easy to initialise Rust integration
testing within a Sway project.

This template is designed for Rust developers who wish to test integration of
their Rust application and Sway code.

To use this template with a Sway project:
- Install [cargo-generate](https://github.com/cargo-generate/cargo-generate)
- Create the template project inside the current directory, entering the project name when prompted, with the following command:
```sh
cargo generate FuelLabs/sway
```
- To initialise a Sway project within the template project just created:
```sh
forc init --path <project name>
```

Rust integration testing can then be used by:
- Entering the project
```sh
cd <project name>
```
- Building the Sway elements with:
```sh
forc build
```
- Running the tests with:
```sh
cargo test
```

See the "Testing with Rust" chapter of [the Sway
Book](https://fuellabs.github.io/sway/v0.25.0/) for a thorough guide on how to
use this template for integration testing.

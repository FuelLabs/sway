# Testing

Sway aims to provide facilities for both unit testing and integration testing.

**Unit testing** refers to "in-language" testing which can be triggered via the
`forc test` command. Sway unit testing is currently a high-priority
work-in-progress, you can follow along at [this
issue](https://github.com/FuelLabs/sway/issues/1832).

**Integration testing** refers to the testing of your Sway project's integration
within some wider application. You can add integration testing to your Sway+Rust
projects today using the cargo generate template and Rust SDK.

- [Unit Testing](./unit-testing.md)
- [Testing with Rust](./testing-with-rust.md)

# Project Structure

If you have used Rust, the structure of a Sway project will feel very familiar to you. It looks like this:

<!-- markdownlint-disable-next-line fenced-code-language -->
```
my-project/
├── Forc.toml
├── src
│   └── main.sw
└── tests
    ├── Cargo.toml
    └── harness.rs
```

When initializing a new project via `forc init`, this is the structure that it will default to.

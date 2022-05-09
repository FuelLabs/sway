pub const SUBHEADERS: &[&str] = &["USAGE:", "ARGS:", "OPTIONS:", "SUBCOMMANDS:"];
pub const INDEX_HEADER: &str = "Here are a list of commands available to forc:\n\n";

pub static RUN_WRITE_DOCS_MESSAGE: &str = "please run `cargo run --bin forc-documenter write-docs`. If you have made local changes to any forc native commands, please install forc from path first: `cargo install --path ./forc`, then run the command.";

pub static EXAMPLES_HEADER: &str = "\n## EXAMPLES:\n";

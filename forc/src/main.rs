use forc_util::ForcCliResult;

#[tokio::main]
async fn main() -> ForcCliResult<()> {
    let input =
        "-a --long-argument=value --input-file='file.txt' -o 'foo/path with space/output.txt' ";
    let formatted_args = argument_format(input);
    panic!("{}", formatted_args);
    forc::cli::run_cli().await.into()
}

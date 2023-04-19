use forc_util::ForcCliResult;

#[tokio::main]
async fn main() -> ForcCliResult<()> {
    forc::cli::run_cli().await.into()
}

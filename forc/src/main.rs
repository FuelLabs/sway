use forc_types::ForcCliResult;

#[tokio::main]
async fn main() -> ForcCliResult<()> {
    forc::cli::run_cli().await.into()
}

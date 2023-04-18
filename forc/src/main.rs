use forc_util::ForcResult;

#[tokio::main]
async fn main() -> ForcResult<()> {
    forc::cli::run_cli().await
}

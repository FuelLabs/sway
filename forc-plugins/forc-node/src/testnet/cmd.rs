use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct TestnetCmd {
    #[clap(long = "port")]
    pub port: Option<u16>,
    #[clap(long = "peer-id")]
    pub peer_id: Option<String>,
    #[clap(long = "secret")]
    pub secret: Option<String>,
}

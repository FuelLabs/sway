use clap::{crate_version, Parser};
use forc_publish::credentials::get_auth_token;
use forc_publish::error::Result;
use forc_publish::forc_pub_client::ForcPubClient;
use forc_publish::tarball::create_tarball_from_current_dir;
use forc_tracing::{
    init_tracing_subscriber, println_action_green, println_error, TracingSubscriberOptions,
};
use tempfile::tempdir;
use url::Url;

const FORC_PUB_URL: &str = "https://api.forc.pub";

#[derive(Parser, Debug)]
#[clap(name = "forc-publish", version)]
/// Forc plugin for uploading packages to the registry.
pub struct Opt {
    /// Token to use when uploading
    #[clap(long)]
    pub token: Option<String>,

    /// The registry URL to use
    #[clap(long, default_value = FORC_PUB_URL)]
    pub registry_url: String,
}

#[tokio::main]
async fn main() {
    init_tracing_subscriber(TracingSubscriberOptions::default());

    if let Err(err) = run().await {
        println_error(&format!("{err}"));
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let config = Opt::parse();
    let auth_token = get_auth_token(config.token, None)?;
    let forc_version = crate_version!();
    let client = ForcPubClient::new(Url::parse(&config.registry_url)?);

    // Create the compressed tarball
    let temp_dir = tempdir()?;
    let file_path = create_tarball_from_current_dir(&temp_dir)?;

    // Upload the tarball and publish it
    let upload_id = client.upload(file_path, forc_version).await?;
    let published = client.publish(upload_id, &auth_token).await?;

    println_action_green(
        "Published",
        &format!("{} {}", published.name, published.version),
    );
    Ok(())
}

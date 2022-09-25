//! A `forc` plugin for running the fuel block explorer.
//!
//! Once installed and available via `PATH`, can be executed via `forc explore`.

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use forc_util::init_tracing_subscriber;
use forc_util::println_green;
use serde::Deserialize;
use std::{
    fs::{self, File},
    io::{self, Cursor},
};
use tar::Archive;
use tracing::{error, info};
use warp::Filter;

#[derive(Debug, Parser)]
#[clap(
    name = "forc-explore",
    about = "Forc plugin for running the Fuel Block Explorer.",
    version
)]
struct App {
    /// The port number at which the explorer will run on localhost.
    #[clap(short = 'p', long = "port", default_value = "3030")]
    pub port: String,
    #[clap(subcommand)]
    pub subcmd: Option<Subcommand>,
}

#[derive(Debug, Parser)]
enum Subcommand {
    /// Cleans up any existing state associated with the fuel block explorer.
    Clean,
}

#[derive(Deserialize, Debug)]
struct GitHubRelease {
    assets: Vec<GitHubReleaseAsset>,
    name: String,
}

#[derive(Deserialize, Debug)]
struct GitHubReleaseAsset {
    browser_download_url: String,
}

const REPO_RELEASES_URL: &str = "https://api.github.com/repos/FuelLabs/block-explorer-v2/releases";

#[tokio::main]
async fn main() {
    let app = App::parse();
    let result = match app.subcmd {
        Some(Subcommand::Clean) => clean(),
        None => run(app).await,
    };
    if let Err(err) = result {
        error!("Error: {:?}", err);
        std::process::exit(1);
    }
}

fn clean() -> Result<()> {
    let path = path::web_app();
    if path.exists() {
        fs::remove_dir_all(path).with_context(|| "failed to clean up web app")?;
    }
    Ok(())
}

async fn run(app: App) -> Result<()> {
    init_tracing_subscriber(None);
    let App { port, .. } = app;
    let releases = get_github_releases().await?;
    let release = releases
        .first()
        .ok_or_else(|| anyhow!("no releases to select from"))?;
    let version = release.name.as_str();
    let message = format!("Fuel Network Explorer {}", version);
    println_green(&message);

    // Download and unpack the latest release if we don't have it yet.
    let is_downloaded = check_version_path(version);
    if !is_downloaded {
        let url = release_url(release)?;
        let _arch = download_build(url, version)
            .await
            .map_err(|e| anyhow!("{e}"))
            .with_context(|| "failed to download build")?;
        unpack_archive(version).with_context(|| "failed to unpack build archive")?;
        let src_name = path::build_archive_unpack(version);
        let dst_name = path::web_app_files(version);
        fs::rename(src_name, dst_name).with_context(|| "failed to move static files")?;
        fs::remove_file(path::build_archive(version))
            .with_context(|| "failed to clean up build files")?;
    }

    start_server(&port, version).await
}

async fn get_github_releases() -> Result<Vec<GitHubRelease>, reqwest::Error> {
    let client = reqwest::Client::new();
    let response = client
        .get(REPO_RELEASES_URL)
        .header("User-Agent", "warp")
        .send()
        .await?;
    response.json().await
}

fn check_version_path(version: &str) -> bool {
    let path = path::web_app_version(version);
    path.exists()
}

fn release_url(release: &GitHubRelease) -> Result<&str> {
    release
        .assets
        .first()
        .ok_or_else(|| anyhow!("release contains no assets"))
        .map(|asset| &asset.browser_download_url[..])
}

async fn download_build(url: &str, version: &str) -> Result<File> {
    fs::create_dir_all(path::web_app().join(version))?;
    let mut file = File::create(path::build_archive(version))
        .with_context(|| "failed to create the build archive")?;
    let response = reqwest::get(url).await?;
    let mut content = Cursor::new(response.bytes().await?);
    io::copy(&mut content, &mut file)?;
    Ok(file)
}

fn unpack_archive(version: &str) -> Result<()> {
    let mut ar = Archive::new(File::open(path::build_archive(version))?);
    ar.unpack(path::web_app_version(version))?;
    Ok(())
}

async fn start_server(port: &str, version: &str) -> Result<()> {
    let explorer = warp::path::end().and(warp::fs::dir(path::web_app_files(version)));
    let static_assets = warp::path(end_point_static_files())
        .and(warp::fs::dir(path::web_app_static_assets(version)));
    let routes = static_assets.or(explorer);
    let port_number = port
        .parse::<u16>()
        .with_context(|| "invalid port number, expected integer value in the range [0, 65535]")?;
    info!("Running server on http://127.0.0.1:{}", port_number);
    warp::serve(routes).run(([127, 0, 0, 1], port_number)).await;
    Ok(())
}

fn end_point_static_files() -> String {
    "static".to_string()
}

pub(crate) mod path {
    use std::path::PathBuf;

    pub fn explorer_directory() -> PathBuf {
        forc_util::user_forc_directory().join("explorer")
    }

    pub fn web_app() -> PathBuf {
        explorer_directory()
    }

    pub fn web_app_version(version: &str) -> PathBuf {
        explorer_directory().join(version)
    }

    pub fn web_app_files(version: &str) -> PathBuf {
        explorer_directory().join(version).join("www")
    }

    pub fn build_archive(version: &str) -> PathBuf {
        explorer_directory()
            .join(version)
            .join("build")
            .with_extension("tar")
    }

    pub fn build_archive_unpack(version: &str) -> PathBuf {
        explorer_directory().join(version).join("build")
    }

    pub fn web_app_static_assets(version: &str) -> PathBuf {
        explorer_directory()
            .join(version)
            .join("www")
            .join("static")
    }
}

use std::fs::{create_dir_all, remove_dir_all, remove_file, rename, File};
use std::io::Cursor;
use std::path::PathBuf;

use crate::utils::helpers::user_forc_directory;
use ansi_term::Colour;
use reqwest;
use serde::Deserialize;
use tar::Archive;
use warp::Filter;

use crate::cli::ExplorerCommand;
type DownloadResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Deserialize, Debug)]
struct GitHubRelease {
    url: String,
    assets: Vec<GitHubReleaseAsset>,
    name: String,
}

#[derive(Deserialize, Debug)]
struct GitHubReleaseAsset {
    browser_download_url: String,
}

const REPO_RELEASES_URL: &str = "https://api.github.com/repos/FuelLabs/block-explorer-v2/releases";

struct EndPoints {}

impl EndPoints {
    pub fn static_files() -> String {
        "static".to_string()
    }
}

struct ExplorerAppPaths {}

fn explorer_directory() -> PathBuf {
    user_forc_directory().join("explorer")
}

impl ExplorerAppPaths {
    pub fn web_app_path() -> PathBuf {
        explorer_directory()
    }
    pub fn web_app_version_path(version: &str) -> PathBuf {
        explorer_directory().join(version)
    }
    pub fn web_app_files_path(version: &str) -> PathBuf {
        explorer_directory().join(version).join("www")
    }
    pub fn build_archive_path(version: &str) -> PathBuf {
        explorer_directory()
            .join(version)
            .join("build")
            .with_extension("tar")
    }
    pub fn build_archive_unpack_path(version: &str) -> PathBuf {
        explorer_directory().join(version).join("build")
    }
    pub fn web_app_static_assets_path(version: &str) -> PathBuf {
        explorer_directory()
            .join(version)
            .join("www")
            .join("static")
    }
}

pub(crate) async fn exec(command: ExplorerCommand) -> Result<(), reqwest::Error> {
    if command.clean.is_some() {
        exec_clean().await
    } else {
        exec_start(command).await
    }
}

async fn exec_start(command: ExplorerCommand) -> Result<(), reqwest::Error> {
    let ExplorerCommand { port, .. } = command;
    let releases = get_github_releases().await?;
    let version = get_latest_release_name(releases.as_slice());
    let message = format!("Fuel Network Explorer {}", version);
    println!("{}", Colour::Green.paint(message));
    let is_downloaded = check_version_path(version);

    if !is_downloaded {
        let url = get_release_url(releases.as_slice(), version);
        match download_build(url, version).await {
            Ok(arch) => arch,
            Err(error) => panic!("Failed to download build {:?}", error),
        };
        match unpack_archive(version) {
            Ok(_) => (),
            Err(error) => panic!("Failed to unpack build archive {:?}", error),
        };
        if let Err(error) = rename(
            ExplorerAppPaths::build_archive_unpack_path(version),
            ExplorerAppPaths::web_app_files_path(version),
        ) {
            panic!("Failed to move static files {:?}", error)
        }
        match remove_file(ExplorerAppPaths::build_archive_path(version)) {
            Ok(_) => (),
            Err(error) => eprintln!("Failed clean up files {:?}", error),
        }
    }
    start_server(port.as_str(), version).await;
    Ok(())
}

async fn exec_clean() -> Result<(), reqwest::Error> {
    let path = ExplorerAppPaths::web_app_path();
    if path.exists() {
        match remove_dir_all(path) {
            Ok(_) => (),
            Err(error) => eprintln!("Failed clean up files {:?}", error),
        }
    }
    Ok(())
}

async fn get_github_releases() -> Result<Vec<GitHubRelease>, reqwest::Error> {
    let client = reqwest::Client::new();
    let response = client
        .get(REPO_RELEASES_URL)
        .header("User-Agent", "warp")
        .send()
        .await?;
    Ok(response.json().await?)
}

fn get_latest_release_name(releases: &[GitHubRelease]) -> &str {
    let a = match releases.first() {
        Some(release) => release,
        None => panic!("No version has been released yet!"),
    };
    a.name.as_str()
}

fn check_version_path(version: &str) -> bool {
    let path = ExplorerAppPaths::web_app_version_path(version);
    path.exists()
}

fn get_release_url<'a>(releases: &'a [GitHubRelease], name: &str) -> &'a str {
    let mut url: &'a str = "";

    for release in releases {
        if release.name == name {
            url = &release.assets.first().unwrap().browser_download_url;
            break;
        }
    }
    url
}

async fn download_build(url: &str, version: &str) -> DownloadResult<File> {
    create_dir_all(ExplorerAppPaths::web_app_path().join(version))?;
    let mut file = match File::create(ExplorerAppPaths::build_archive_path(version)) {
        Ok(fc) => fc,
        Err(error) => panic!("Problem creating the build archive: {:?}", error),
    };
    let response = reqwest::get(url).await?;
    let mut content = Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;
    Ok(file)
}

fn unpack_archive(version: &str) -> Result<(), std::io::Error> {
    let mut ar = Archive::new(File::open(ExplorerAppPaths::build_archive_path(version)).unwrap());
    ar.unpack(ExplorerAppPaths::web_app_version_path(version))
        .unwrap();
    Ok(())
}

async fn start_server(port: &str, version: &str) {
    let explorer =
        warp::path::end().and(warp::fs::dir(ExplorerAppPaths::web_app_files_path(version)));
    let static_assets = warp::path(EndPoints::static_files()).and(warp::fs::dir(
        ExplorerAppPaths::web_app_static_assets_path(version),
    ));
    let routes = static_assets.or(explorer);

    let port_number = match port.parse::<u16>() {
        Ok(n) => n,
        Err(_) => panic!(
            "Invalid port number {:?}. Expected integer value in the range [0, 65535].",
            port
        ),
    };
    println!("Started server on http://127.0.0.1:{}", port_number);
    warp::serve(routes).run(([127, 0, 0, 1], port_number)).await
}

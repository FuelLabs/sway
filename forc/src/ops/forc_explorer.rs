use std::fs::{remove_file, rename, File, create_dir_all};
use std::io::Cursor;
use std::path::{PathBuf};

use dirs;
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

impl ExplorerAppPaths {
    pub fn web_app_path() -> PathBuf {
        dirs::home_dir().unwrap().join(".fuel/explorer")
    }
    pub fn build_archive_path() -> PathBuf {
        dirs::home_dir().unwrap().join(".fuel/explorer/build.tar")
    }
    pub fn build_archive_unpack_path() -> PathBuf {
        dirs::home_dir().unwrap().join(".fuel/explorer/build")
    }
    pub fn web_app_files_path() -> PathBuf {
        dirs::home_dir().unwrap().join(".fuel/explorer/www")
    }
    pub fn web_app_static_assets_path() -> PathBuf {
        dirs::home_dir().unwrap().join(".fuel/explorer/www/static")
    }
}

pub(crate) async fn exec(command: ExplorerCommand) -> Result<(), reqwest::Error> {
    let ExplorerCommand { port } = command;
    if !has_static_files() {
        let download_url = match get_release_url().await {
            Ok(url) => url,
            Err(error) => panic!("Failed to get release {:?}", error),
        };
        eprintln!("Downloading Fuel Explorer ...");

        match download_build(&download_url).await {
            Ok(arch) => arch,
            Err(error) => panic!("Failed to download build {:?}", error),
        };

        match unpack_archive() {
            Ok(_) => (),
            Err(error) => panic!("Failed to unpack build archive {:?}", error),
        };

        if let Err(error) = rename(ExplorerAppPaths::build_archive_unpack_path(), ExplorerAppPaths::web_app_files_path()) {
            panic!("Failed to move static files {:?}", error)
        }

        match remove_file(ExplorerAppPaths::build_archive_path()) {
            Ok(_) => (),
            Err(error) => eprintln!("Failed clean up files {:?}", error),
        }
    }

    start_server(port).await;
    Ok(())
}

fn has_static_files() -> bool {
    ExplorerAppPaths::web_app_files_path().clone().join("index.html").exists()
}

async fn get_release_url() -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let releases_url = REPO_RELEASES_URL;
    let response = client
        .get(releases_url)
        .header("User-Agent", "warp")
        .send()
        .await?;
    let response_json: Vec<GitHubRelease> = response.json().await?;
    let download_url = response_json
        .first()
        .unwrap()
        .assets
        .first()
        .unwrap()
        .browser_download_url
        .clone();
    Ok(download_url)
}

async fn download_build(url: &str) -> DownloadResult<File> {
    println!("{:?}", ExplorerAppPaths::build_archive_path());
    create_dir_all(ExplorerAppPaths::web_app_path())?;
    let mut file = match File::create(ExplorerAppPaths::build_archive_path()) {
        Ok(fc) => fc,
        Err(error) => panic!("Problem creating the build archive: {:?}", error),
    };
    let response = reqwest::get(url).await?;
    let mut content = Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;
    Ok(file)
}

fn unpack_archive() -> Result<(), std::io::Error> {
    let mut ar = Archive::new(File::open(ExplorerAppPaths::build_archive_path()).unwrap());
    ar.unpack(ExplorerAppPaths::web_app_path()).unwrap();
    Ok(())
}

async fn start_server(port: String) {
    let explorer = warp::path::end().and(warp::fs::dir(ExplorerAppPaths::web_app_files_path()));
    let static_assets =
        warp::path(EndPoints::static_files()).and(warp::fs::dir(ExplorerAppPaths::web_app_static_assets_path()));
    let routes = static_assets.or(explorer);

    let port_number = match port.parse::<u16>() {
        Ok(n) => n,
        Err(error) => panic!("Invalid port number {:?}", error),
    };
    println!("Running Fuel Network Explorer on 127.0.0.1:{}", port_number);
    warp::serve(routes).run(([127, 0, 0, 1], port_number)).await
}

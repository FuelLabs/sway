use std::fs::{remove_file, rename, File};
use std::io::Cursor;
use std::path::Path;

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

const BUILD_ARCHIVE_PATH: &str = "forc/build.tar";
const STATIC_FILES_PATH: &str = "forc/www";
const REPO_RELEASES_URL: &str = "https://api.github.com/repos/FuelLabs/block-explorer-v2/releases";

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

        match rename("forc/build", STATIC_FILES_PATH) {
            Err(error) => panic!("Failed to move static files {:?}", error),
            Ok(_) => (),
        };

        match remove_file(BUILD_ARCHIVE_PATH) {
            Ok(_) => (),
            Err(error) => eprintln!("Failed clean up files {:?}", error),
        }
    }

    start_server(port).await;
    Ok(())
}

fn has_static_files() -> bool {
    Path::new(&format!("{}/index.html", STATIC_FILES_PATH)).exists()
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

async fn download_build(url: &String) -> DownloadResult<File> {
    let mut file = match File::create(BUILD_ARCHIVE_PATH) {
        Ok(fc) => fc,
        Err(error) => panic!("Problem creating the build archive: {:?}", error),
    };
    let response = reqwest::get(url).await?;
    let mut content = Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;
    Ok(file)
}

fn unpack_archive() -> Result<(), std::io::Error> {
    let mut ar = Archive::new(File::open(BUILD_ARCHIVE_PATH).unwrap());
    ar.unpack("forc/").unwrap();
    Ok(())
}

async fn start_server(port: String) {
    let explorer = warp::path::end().and(warp::fs::dir(STATIC_FILES_PATH));
    let static_assets =
        warp::path("static").and(warp::fs::dir(format!("{}/static", STATIC_FILES_PATH)));
    let routes = static_assets.or(explorer);

    let port_number = match port.parse::<u16>() {
        Ok(n) => n,
        Err(error) => panic!("Invalid port number {:?}", error),
    };
    println!("Running Fuel Network Explorer on 127.0.0.1:{}", port_number);
    warp::serve(routes).run(([127, 0, 0, 1], port_number)).await
}

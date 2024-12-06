use crate::{
    chain_config::ChainConfig,
    consts::{
        CHAIN_CONFIG_REPO_NAME, CONFIG_FOLDER, MAINNET_CONFIG_FOLDER_NAME,
        TESTNET_CONFIG_FOLDER_NAME,
    },
};
use anyhow::{anyhow, bail, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password};
use forc_tracing::{println_action_green, println_warning};
use forc_util::user_forc_directory;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyPair {
    pub peer_id: String,
    pub secret: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubContentDetails {
    name: String,
    sha: String,
    download_url: Option<String>,
    #[serde(rename = "type")]
    content_type: String,
}

pub struct ConfigFetcher {
    client: reqwest::Client,
    #[cfg(test)]
    base_url: String,
    config_vault: PathBuf,
}

impl ConfigFetcher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            #[cfg(test)]
            base_url: "https://api.github.com".to_string(),
            config_vault: user_forc_directory().join(CONFIG_FOLDER),
        }
    }

    #[cfg(test)]
    pub fn with_base_url(base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            config_vault: user_forc_directory().join(CONFIG_FOLDER),
        }
    }

    #[cfg(test)]
    pub fn with_test_config(base_url: String, config_vault: PathBuf) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            config_vault,
        }
    }

    fn get_base_url(&self) -> &str {
        #[cfg(not(test))]
        return "https://api.github.com";

        #[cfg(test)]
        return &self.base_url;
    }

    fn build_api_endpoint(&self, folder_name: &str) -> String {
        format!(
            "{}/repos/FuelLabs/{}/contents/{}",
            self.get_base_url(),
            CHAIN_CONFIG_REPO_NAME,
            folder_name,
        )
    }

    async fn check_github_files(
        &self,
        conf: &ChainConfig,
    ) -> anyhow::Result<Vec<GithubContentDetails>> {
        let folder_name = match conf {
            ChainConfig::Local => bail!("Local configuration should not be fetched from github"),
            ChainConfig::Testnet => TESTNET_CONFIG_FOLDER_NAME,
            ChainConfig::Ignition => MAINNET_CONFIG_FOLDER_NAME,
        };
        let api_endpoint = self.build_api_endpoint(folder_name);

        let response = self
            .client
            .get(&api_endpoint)
            .header("User-Agent", "forc-node")
            .send()
            .await?;

        if !response.status().is_success() {
            bail!("failed to fetch updates from github")
        }

        let contents: Vec<GithubContentDetails> = response.json().await?;
        Ok(contents)
    }

    fn check_local_files(&self, conf: &ChainConfig) -> Result<Option<HashMap<String, String>>> {
        let folder_name = match conf {
            ChainConfig::Local => bail!("Local configuration should not be checked"),
            ChainConfig::Testnet => TESTNET_CONFIG_FOLDER_NAME,
            ChainConfig::Ignition => MAINNET_CONFIG_FOLDER_NAME,
        };

        let folder_path = self.config_vault.join(folder_name);

        if !folder_path.exists() {
            return Ok(None);
        }

        let mut files = HashMap::new();
        for entry in std::fs::read_dir(&folder_path)? {
            let entry = entry?;
            if entry.path().is_file() {
                let content = std::fs::read(entry.path())?;
                // Calculate SHA1 the same way GitHub does
                let mut hasher = Sha1::new();
                hasher.update(b"blob ");
                hasher.update(content.len().to_string().as_bytes());
                hasher.update(&[0]);
                hasher.update(&content);
                let sha = format!("{:x}", hasher.finalize());

                let name = entry.file_name().into_string().unwrap();
                files.insert(name, sha);
            }
        }

        Ok(Some(files))
    }

    /// Checks if a fetch is requried by comparing the hashes of indivual files
    /// of the given chain config in the local instance to the one in github by
    /// utilizing the github content abi.
    pub async fn check_fetch_required(&self, conf: &ChainConfig) -> anyhow::Result<bool> {
        if *conf == ChainConfig::Local {
            return Ok(false);
        }

        let local_files = match self.check_local_files(conf)? {
            Some(files) => files,
            None => return Ok(true), // No local files, need to fetch
        };

        let github_files = self.check_github_files(conf).await?;

        // Compare files
        for github_file in &github_files {
            if github_file.content_type == "file" {
                match local_files.get(&github_file.name) {
                    Some(local_sha) if local_sha == &github_file.sha => continue,
                    _ => return Ok(true), // SHA mismatch or file doesn't exist locally
                }
            }
        }

        // Also check if we have any extra files locally that aren't on GitHub
        let github_filenames: HashSet<_> = github_files
            .iter()
            .filter(|f| f.content_type == "file")
            .map(|f| &f.name)
            .collect();

        let local_filenames: HashSet<_> = local_files.keys().collect();

        if local_filenames != github_filenames {
            return Ok(true);
        }

        Ok(false)
    }

    /// Download the chain config for given mode
    pub async fn download_config(&self, conf: &ChainConfig) -> anyhow::Result<()> {
        let folder_name = match conf {
            ChainConfig::Local => bail!("Local configuration should not be downloaded"),
            ChainConfig::Testnet => TESTNET_CONFIG_FOLDER_NAME,
            ChainConfig::Ignition => MAINNET_CONFIG_FOLDER_NAME,
        };

        let api_endpoint = format!(
            "https://api.github.com/repos/FuelLabs/{}/contents/{}",
            CHAIN_CONFIG_REPO_NAME, folder_name,
        );

        let contents = self.fetch_folder_contents(&api_endpoint).await?;

        // Create config directory if it doesn't exist
        let config_dir = user_forc_directory().join(CONFIG_FOLDER);
        let target_dir = config_dir.join(folder_name);
        fs::create_dir_all(&target_dir)?;

        // Download each file
        for item in contents {
            if item.content_type == "file" {
                if let Some(download_url) = item.download_url {
                    let file_path = target_dir.join(&item.name);

                    let response = self.client.get(&download_url).send().await?;

                    if !response.status().is_success() {
                        bail!("Failed to download file: {}", item.name);
                    }

                    let content = response.bytes().await?;
                    fs::write(file_path, content)?;
                }
            }
        }

        Ok(())
    }

    /// Helper function to fetch folder contents from GitHub
    async fn fetch_folder_contents(&self, url: &str) -> anyhow::Result<Vec<GithubContentDetails>> {
        let response = self
            .client
            .get(url)
            .header("User-Agent", "forc-node")
            .send()
            .await?;

        if !response.status().is_success() {
            bail!("failed to fetch contents from github");
        }

        Ok(response.json().await?)
    }
}

/// Check local state of the configuration file in the vault (if they exists)
/// and compare them to the remote one in github. If a change is detected asks
/// user if they want to update, and does the update for them.
pub async fn update_chain_config(conf: ChainConfig) -> anyhow::Result<()> {
    println_action_green("Checking", "for network configuration updates.");
    let fetcher = ConfigFetcher::new();

    if fetcher.check_fetch_required(&conf).await? {
        println_warning(&format!(
            "A network configuration update detected for {}, this might create problems while syncing with rest of the network",
            conf
        ));
        // Ask user if they want to udpate the chain config.
        let update = ask_user_yes_no_question("Would you like to update network configuration?")?;
        if update {
            println_action_green("Updating", &format!("configuration files for {conf}",));
            fetcher.download_config(&conf).await?;
            println_action_green(
                "Finished",
                &format!("updating configuration files for {conf}",),
            );
        }
    } else {
        println_action_green(&format!("{conf}"), "is up-to-date.");
    }
    Ok(())
}
/// Checks the local fuel-core's version that `forc-node` will be runnning.
pub(crate) fn get_fuel_core_version() -> anyhow::Result<Version> {
    let version_cmd = Command::new("fuel-core")
        .arg("--version")
        .stdout(Stdio::piped())
        .output()
        .expect("failed to run fuel-core, make sure that it is installed.");

    let version_output = String::from_utf8_lossy(&version_cmd.stdout).to_string();

    // Version output is `fuel-core <SEMVER VERSION>`. We should split it to only
    // get the version part of it before parsing as semver.
    let version = version_output
        .split_whitespace()
        .last()
        .ok_or_else(|| anyhow!("fuel-core version parse failed"))?;
    let version_semver = Version::parse(version)?;

    Ok(version_semver)
}

/// Given a `Command`, wrap it to enable generating the actual string that would
/// create this command.
/// Example:
/// ```rust
/// let command = Command::new("fuel-core").arg("run");
/// let command = HumanReadableCommand(command);
/// let formatted = format!("{command}");
/// assert_eq!(&formatted, "fuel-core run");
/// ```
pub struct HumanReadableCommand(Command);

impl Display for HumanReadableCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dbg_out = format!("{:?}", self.0);
        // This is in the ""command-name" "param-name" "param-val"" format.
        let parsed = dbg_out
            .replace("\" \"", " ") // replace " " between items with space
            .replace("\"", ""); // remove remaining quotes at start/end
        write!(f, "{parsed}")
    }
}

pub(crate) fn ask_user_yes_no_question(question: &str) -> anyhow::Result<bool> {
    let answer = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(question)
        .default(false)
        .show_default(false)
        .interact()?;
    Ok(answer)
}

pub(crate) fn ask_user_discreetly(question: &str) -> anyhow::Result<String> {
    let discrete = Password::with_theme(&ColorfulTheme::default())
        .with_prompt(question)
        .interact()?;
    Ok(discrete)
}

pub(crate) fn ask_user_string(question: &str) -> anyhow::Result<String> {
    let response = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(question)
        .interact_text()?;
    Ok(response)
}

impl From<Command> for HumanReadableCommand {
    fn from(value: Command) -> Self {
        Self(value)
    }
}

/// Ask if the user has a keypair generated and if so, collect the details.
/// If not, bails out with a help message about how to generate a keypair.
pub(crate) fn ask_user_keypair() -> Result<KeyPair> {
    let has_keypair = ask_user_yes_no_question("Do you have a keypair in hand?")?;
    if has_keypair {
        // ask the keypair
        let peer_id = ask_user_string("Peer Id:")?;
        let secret = ask_user_discreetly("Secret:")?;
        Ok(KeyPair { peer_id, secret })
    } else {
        bail!("Please create a keypair with `fuel-core-keygen new --key-type peering`");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;
    use tokio;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[test]
    fn test_basic_command() {
        let mut command = Command::new("fuel-core");
        command.arg("run");
        let human_readable = HumanReadableCommand(command);
        assert_eq!(format!("{human_readable}"), "fuel-core run");
    }

    #[test]
    fn test_command_with_multiple_args() {
        let mut command = Command::new("fuel-core");
        command.arg("run");
        command.arg("--config");
        command.arg("config.toml");
        let human_readable = HumanReadableCommand(command);
        assert_eq!(
            format!("{human_readable}"),
            "fuel-core run --config config.toml"
        );
    }

    #[test]
    fn test_command_no_args() {
        let command = Command::new("fuel-core");
        let human_readable = HumanReadableCommand(command);
        assert_eq!(format!("{human_readable}"), "fuel-core");
    }

    #[test]
    fn test_command_with_path() {
        let mut command = Command::new("fuel-core");
        command.arg("--config");
        command.arg("/path/to/config.toml");
        let human_readable = HumanReadableCommand(command);
        assert_eq!(
            format!("{human_readable}"),
            "fuel-core --config /path/to/config.toml"
        );
    }

    #[tokio::test]
    async fn test_fetch_not_required_when_files_match() {
        let mock_server = MockServer::start().await;
        let test_files = [
            ("config.json", "test config content"),
            ("metadata.json", "test metadata content"),
        ];

        // Create test directory and files
        let test_dir = TempDir::new().unwrap();
        let config_path = test_dir.path().to_path_buf();
        let test_folder = config_path.join(TESTNET_CONFIG_FOLDER_NAME);
        fs::create_dir_all(&test_folder).unwrap();

        for (name, content) in &test_files {
            fs::write(test_folder.join(name), content).unwrap();
        }

        // Setup mock response
        let github_response = create_github_response(&test_files);
        Mock::given(method("GET"))
            .and(path(format!(
                "/repos/FuelLabs/{}/contents/{}",
                CHAIN_CONFIG_REPO_NAME, TESTNET_CONFIG_FOLDER_NAME
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(&github_response))
            .mount(&mock_server)
            .await;

        let fetcher = ConfigFetcher::with_test_config(mock_server.uri(), config_path);

        let needs_fetch = fetcher
            .check_fetch_required(&ChainConfig::Testnet)
            .await
            .unwrap();

        assert!(
            !needs_fetch,
            "Fetch should not be required when files match"
        );
    }

    #[tokio::test]
    async fn test_fetch_required_when_files_differ() {
        let mock_server = MockServer::start().await;

        // Create local test files
        let test_dir = TempDir::new().unwrap();
        let config_path = test_dir.path().join("fuel").join("configs");
        let test_folder = config_path.join(TESTNET_CONFIG_FOLDER_NAME);
        fs::create_dir_all(&test_folder).unwrap();

        let local_files = [
            ("config.json", "old config content"),
            ("metadata.json", "old metadata content"),
        ];

        for (name, content) in &local_files {
            fs::write(test_folder.join(name), content).unwrap();
        }

        // Setup mock GitHub response with different content
        let github_files = [
            ("config.json", "new config content"),
            ("metadata.json", "new metadata content"),
        ];
        let github_response = create_github_response(&github_files);

        Mock::given(method("GET"))
            .and(path(format!(
                "/repos/FuelLabs/{}/contents/{}",
                CHAIN_CONFIG_REPO_NAME, TESTNET_CONFIG_FOLDER_NAME
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(&github_response))
            .mount(&mock_server)
            .await;

        let fetcher = ConfigFetcher::with_base_url(mock_server.uri());

        let needs_fetch = fetcher
            .check_fetch_required(&ChainConfig::Testnet)
            .await
            .unwrap();

        assert!(needs_fetch, "Fetch should be required when files differ");
    }

    #[tokio::test]
    async fn test_fetch_required_when_files_missing() {
        let mock_server = MockServer::start().await;

        // Create local test files (missing one file)
        let test_dir = TempDir::new().unwrap();
        let config_path = test_dir.path().join("fuel").join("configs");
        let test_folder = config_path.join(TESTNET_CONFIG_FOLDER_NAME);
        fs::create_dir_all(&test_folder).unwrap();

        let local_files = [("config.json", "test config content")];

        for (name, content) in &local_files {
            fs::write(test_folder.join(name), content).unwrap();
        }

        // Setup mock GitHub response with extra file
        let github_files = [
            ("config.json", "test config content"),
            ("metadata.json", "test metadata content"),
        ];
        let github_response = create_github_response(&github_files);

        Mock::given(method("GET"))
            .and(path(format!(
                "/repos/FuelLabs/{}/contents/{}",
                CHAIN_CONFIG_REPO_NAME, TESTNET_CONFIG_FOLDER_NAME
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(&github_response))
            .mount(&mock_server)
            .await;

        let fetcher = ConfigFetcher::with_base_url(mock_server.uri());

        let needs_fetch = fetcher
            .check_fetch_required(&ChainConfig::Testnet)
            .await
            .unwrap();

        assert!(
            needs_fetch,
            "Fetch should be required when files are missing"
        );
    }

    #[tokio::test]
    async fn test_local_configuration_never_needs_fetch() {
        let fetcher = ConfigFetcher::new();
        let needs_fetch = fetcher
            .check_fetch_required(&ChainConfig::Local)
            .await
            .unwrap();

        assert!(!needs_fetch, "Local configuration should never need fetch");
    }

    #[tokio::test]
    async fn test_fetch_required_when_extra_local_files() {
        let mock_server = MockServer::start().await;

        // Create local test files (with extra file)
        let test_dir = TempDir::new().unwrap();
        let config_path = test_dir.path().join("fuel").join("configs");
        let test_folder = config_path.join(TESTNET_CONFIG_FOLDER_NAME);
        fs::create_dir_all(&test_folder).unwrap();

        let local_files = [
            ("config.json", "test config content"),
            ("metadata.json", "test metadata content"),
            ("extra.json", "extra file content"),
        ];

        for (name, content) in &local_files {
            fs::write(test_folder.join(name), content).unwrap();
        }

        // Setup mock GitHub response with fewer files
        let github_files = [
            ("config.json", "test config content"),
            ("metadata.json", "test metadata content"),
        ];
        let github_response = create_github_response(&github_files);

        Mock::given(method("GET"))
            .and(path(format!(
                "/repos/FuelLabs/{}/contents/{}",
                CHAIN_CONFIG_REPO_NAME, TESTNET_CONFIG_FOLDER_NAME
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(&github_response))
            .mount(&mock_server)
            .await;

        let fetcher = ConfigFetcher::with_base_url(mock_server.uri());

        let needs_fetch = fetcher
            .check_fetch_required(&ChainConfig::Testnet)
            .await
            .unwrap();

        assert!(
            needs_fetch,
            "Fetch should be required when there are extra local files"
        );
    }

    // Helper function to create GitHub response
    fn create_github_response(files: &[(&str, &str)]) -> Vec<GithubContentDetails> {
        files
            .iter()
            .map(|(name, content)| {
                let mut hasher = Sha1::new();
                hasher.update(b"blob ");
                hasher.update(content.len().to_string().as_bytes());
                hasher.update(&[0]);
                hasher.update(content.as_bytes());
                let sha = format!("{:x}", hasher.finalize());

                GithubContentDetails {
                    name: name.to_string(),
                    sha,
                    download_url: Some(format!("https://raw.githubusercontent.com/test/{}", name)),
                    content_type: "file".to_string(),
                }
            })
            .collect()
    }
}

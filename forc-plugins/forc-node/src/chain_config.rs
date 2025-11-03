use crate::{
    consts::{
        CHAIN_CONFIG_REPO_NAME, CONFIG_FOLDER, IGNITION_CONFIG_FOLDER_NAME,
        LOCAL_CONFIG_FOLDER_NAME, TESTNET_CONFIG_FOLDER_NAME,
    },
    util::ask_user_yes_no_question,
};
use anyhow::{bail, Result};
use forc_tracing::{println_action_green, println_warning};
use forc_util::user_forc_directory;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    fs,
    path::PathBuf,
};

/// Different chain configuration options.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ChainConfig {
    Local,
    Testnet,
    Ignition,
}

impl Display for ChainConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainConfig::Local => write!(f, "local"),
            ChainConfig::Testnet => write!(f, "testnet"),
            ChainConfig::Ignition => write!(f, "ignition"),
        }
    }
}

impl From<ChainConfig> for PathBuf {
    fn from(value: ChainConfig) -> Self {
        let folder_name = match value {
            ChainConfig::Local => LOCAL_CONFIG_FOLDER_NAME,
            ChainConfig::Testnet => TESTNET_CONFIG_FOLDER_NAME,
            ChainConfig::Ignition => IGNITION_CONFIG_FOLDER_NAME,
        };
        user_forc_directory().join(CONFIG_FOLDER).join(folder_name)
    }
}

/// A github api, content query response.
/// Mainly used for fetching a download url and hash for configuration files.
#[derive(Serialize, Deserialize, Debug)]
struct GithubContentDetails {
    name: String,
    sha: String,
    download_url: Option<String>,
    #[serde(rename = "type")]
    content_type: String,
}

/// `ConfigFetcher` is responsible for github api integration related to the
/// configuration operations.
/// Basically checks remote hash of the corresponding chain configuration.
/// If there is a mismatch between local and remote instance, overrides the
/// local instance with remote changes for testnet and mainnet configurations.
///
/// For local chain configuration, we only check for existence of it locally.
/// If the local chain configuration is missing in user's local,
/// `ConfigFetcher` fetches it but remote updates are not tracked for it.
pub struct ConfigFetcher {
    client: reqwest::Client,
    base_url: String,
    config_vault: PathBuf,
    non_interactive: bool,
}

impl Default for ConfigFetcher {
    /// Creates a new fetcher to interact with github.
    /// By default user's chain configuration vault is at: `~/.forc/chainspecs`
    fn default() -> Self {
        Self::new(false)
    }
}

impl ConfigFetcher {
    pub fn new(non_interactive: bool) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://api.github.com".to_string(),
            config_vault: user_forc_directory().join(CONFIG_FOLDER),
            non_interactive,
        }
    }

    #[cfg(test)]
    /// Override the base url, to be used in tests.
    pub fn with_base_url(base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            config_vault: user_forc_directory().join(CONFIG_FOLDER),
            non_interactive: false,
        }
    }

    #[cfg(test)]
    pub fn with_test_config(base_url: String, config_vault: PathBuf) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            config_vault,
            non_interactive: false,
        }
    }

    fn non_interactive(&self) -> bool {
        self.non_interactive
    }

    fn get_base_url(&self) -> &str {
        &self.base_url
    }

    fn build_api_endpoint(&self, folder_name: &str) -> String {
        format!(
            "{}/repos/FuelLabs/{}/contents/{}",
            self.get_base_url(),
            CHAIN_CONFIG_REPO_NAME,
            folder_name,
        )
    }

    /// Fetches contents from github to get hashes and download urls for
    /// contents of the remote configuration repo at:
    /// https://github.com/FuelLabs/chain-configuration/
    async fn check_github_files(
        &self,
        conf: &ChainConfig,
    ) -> anyhow::Result<Vec<GithubContentDetails>> {
        let folder_name = match conf {
            ChainConfig::Local => LOCAL_CONFIG_FOLDER_NAME,
            ChainConfig::Testnet => TESTNET_CONFIG_FOLDER_NAME,
            ChainConfig::Ignition => IGNITION_CONFIG_FOLDER_NAME,
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

    /// Calculates the hash for the local configuration instance.
    /// The hash calculation is based on github's hash calculation to match the
    /// github api response.
    fn check_local_files(&self, conf: &ChainConfig) -> Result<Option<HashMap<String, String>>> {
        let folder_name = match conf {
            ChainConfig::Local => bail!("Local configuration should not be checked"),
            ChainConfig::Testnet => TESTNET_CONFIG_FOLDER_NAME,
            ChainConfig::Ignition => IGNITION_CONFIG_FOLDER_NAME,
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
                // Calculate SHA1 the same way github does
                let mut hasher = Sha1::new();
                hasher.update(b"blob ");
                hasher.update(content.len().to_string().as_bytes());
                hasher.update([0]);
                hasher.update(&content);
                let sha = format!("{:x}", hasher.finalize());

                let name = entry.file_name().into_string().unwrap();
                files.insert(name, sha);
            }
        }

        Ok(Some(files))
    }

    /// Checks if a fetch is required by comparing the hashes of individual files
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

    /// Download the chain config for given mode. Fetches the corresponding
    /// directory from: https://github.com/FuelLabs/chain-configuration/.
    pub async fn download_config(&self, conf: &ChainConfig) -> anyhow::Result<()> {
        let folder_name = match conf {
            ChainConfig::Local => LOCAL_CONFIG_FOLDER_NAME,
            ChainConfig::Testnet => TESTNET_CONFIG_FOLDER_NAME,
            ChainConfig::Ignition => IGNITION_CONFIG_FOLDER_NAME,
        };

        let api_endpoint = format!(
            "https://api.github.com/repos/FuelLabs/{CHAIN_CONFIG_REPO_NAME}/contents/{folder_name}",
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

    /// Helper function to fetch folder contents from github.
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

/// Local configuration is validated based on its existence. Meaning that if
/// the configuration exists in user's local it is validated. If it is missing
/// the configuration files are fetched from remote.
async fn validate_local_chainconfig(fetcher: &ConfigFetcher) -> anyhow::Result<()> {
    let user_conf_dir = user_forc_directory().join(CONFIG_FOLDER);
    let local_conf_dir = user_conf_dir.join(LOCAL_CONFIG_FOLDER_NAME);
    if !local_conf_dir.exists() {
        println_warning(&format!(
            "Local node configuration files are missing at {}",
            local_conf_dir.display()
        ));
        let non_interactive = fetcher.non_interactive();
        if non_interactive {
            println_action_green(
                "Downloading",
                "local network configuration (non-interactive mode).",
            );
        }
        let should_download = if non_interactive {
            true
        } else {
            ask_user_yes_no_question("Would you like to download network configuration?")?
        };
        if should_download {
            fetcher.download_config(&ChainConfig::Local).await?;
        } else {
            bail!(
                "Missing local network configuration, create one at {}",
                local_conf_dir.display()
            );
        }
    }
    Ok(())
}

/// Testnet and mainnet chain configurations are validated against the remote
/// versions from github. If local files exists for these configurations, hash
/// values are collected from remote, and compared to find out if there any
/// changes introduced in remote. If there is the chain configuration is
/// fetched again to ensure, the bootstrapped node can sync with the rest of
/// the network without any issues related to a different chain configuration.
async fn validate_remote_chainconfig(
    fetcher: &ConfigFetcher,
    conf: &ChainConfig,
) -> anyhow::Result<()> {
    // For testnet and mainnet configs, we need to check online.
    println_action_green("Checking", "for network configuration updates.");

    if fetcher.check_fetch_required(conf).await? {
        println_warning(&format!(
            "A network configuration update detected for {conf}, this might create problems while syncing with rest of the network"
        ));
        let non_interactive = fetcher.non_interactive();
        if non_interactive {
            println_action_green(
                "Updating",
                &format!("configuration files for {conf} (non-interactive mode)",),
            );
        }
        let should_update = if non_interactive {
            true
        } else {
            ask_user_yes_no_question("Would you like to update network configuration?")?
        };
        if should_update {
            if !non_interactive {
                println_action_green("Updating", &format!("configuration files for {conf}",));
            }
            fetcher.download_config(conf).await?;
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

/// Check local state of the configuration file in the vault (if they exists)
/// and compare them to the remote one in github. If a change is detected asks
/// user if they want to update, and does the update for them.
pub async fn check_and_update_chain_config(
    conf: ChainConfig,
    non_interactive: bool,
) -> anyhow::Result<()> {
    let fetcher = ConfigFetcher::new(non_interactive);
    match conf {
        ChainConfig::Local => validate_local_chainconfig(&fetcher).await?,
        remote_config => validate_remote_chainconfig(&fetcher, &remote_config).await?,
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    // Helper function to create dummy github response
    fn create_github_response(files: &[(&str, &str)]) -> Vec<GithubContentDetails> {
        files
            .iter()
            .map(|(name, content)| {
                let mut hasher = Sha1::new();
                hasher.update(b"blob ");
                hasher.update(content.len().to_string().as_bytes());
                hasher.update([0]);
                hasher.update(content.as_bytes());
                let sha = format!("{:x}", hasher.finalize());

                GithubContentDetails {
                    name: name.to_string(),
                    sha,
                    download_url: Some(format!("https://raw.githubusercontent.com/test/{name}")),
                    content_type: "file".to_string(),
                }
            })
            .collect()
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
                "/repos/FuelLabs/{CHAIN_CONFIG_REPO_NAME}/contents/{TESTNET_CONFIG_FOLDER_NAME}"
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
                "/repos/FuelLabs/{CHAIN_CONFIG_REPO_NAME}/contents/{TESTNET_CONFIG_FOLDER_NAME}"
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
                "/repos/FuelLabs/{CHAIN_CONFIG_REPO_NAME}/contents/{TESTNET_CONFIG_FOLDER_NAME}"
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
        let fetcher = ConfigFetcher::default();
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
                "/repos/FuelLabs/{CHAIN_CONFIG_REPO_NAME}/contents/{TESTNET_CONFIG_FOLDER_NAME}"
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
}

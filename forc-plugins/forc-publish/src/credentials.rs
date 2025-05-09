use crate::error::Result;
use forc_util::user_forc_directory;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self};
use std::path::PathBuf;
use toml;

const CREDENTIALS_FILE: &str = "credentials.toml";

#[derive(Serialize, Deserialize)]
struct Registry {
    token: String,
}

#[derive(Serialize, Deserialize)]
struct Credentials {
    registry: Registry,
}

/// Gets the user's auth token.
/// - First checks CLI arguments.
/// - Then checks `~/.forc/credentials.toml` inside the `[registry]` section.
/// - If neither are found, prompts the user and saves it to `credentials.toml`.
pub fn get_auth_token(
    opt_token: Option<String>,
    credentials_dir: Option<PathBuf>,
) -> Result<String> {
    if let Some(token) = opt_token {
        return Ok(token);
    }

    if let Some(token) = std::env::var("FORC_PUB_TOKEN").ok() {
        return Ok(token);
    }

    let credentials_path = credentials_dir
        .unwrap_or(user_forc_directory())
        .join(CREDENTIALS_FILE);
    if let Some(token) = get_auth_token_from_file(&credentials_path)? {
        return Ok(token);
    }

    let auth_token =
        get_auth_token_from_user_input(&credentials_path, io::stdin().lock(), io::stdout())?;

    Ok(auth_token)
}

// Check if credentials file exists and read from it
fn get_auth_token_from_file(path: &PathBuf) -> Result<Option<String>> {
    if path.exists() {
        let content = fs::read_to_string(path)?;
        if let Ok(credentials) = toml::from_str::<Credentials>(&content) {
            return Ok(Some(credentials.registry.token));
        }
    }
    Ok(None)
}

// Prompt user for input and save to credentials file
fn get_auth_token_from_user_input<R, W>(
    credentials_path: &PathBuf,
    mut reader: R,
    mut writer: W,
) -> Result<String>
where
    R: io::BufRead,
    W: io::Write,
{
    tracing::info!("Paste your auth token found on https://forc.pub/tokens below: ");
    writer.flush()?;
    let mut auth_token = String::new();
    reader.read_line(&mut auth_token)?;
    let auth_token = auth_token.trim().to_string();

    // Save the token to the credentials file
    if let Some(parent_path) = credentials_path.parent() {
        fs::create_dir_all(parent_path)?;
        let credentials = Credentials {
            registry: Registry {
                token: auth_token.clone(),
            },
        };
        fs::write(credentials_path, toml::to_string(&credentials)?)?;
        tracing::info!("Auth token saved to {}", credentials_path.display());
    }
    Ok(auth_token)
}

#[cfg(test)]
mod test {
    use super::*;
    use serial_test::serial;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_get_auth_token_from_cli_arg() {
        let token = Some("cli_token".to_string());
        let result = get_auth_token(token, None).unwrap();
        assert_eq!(result, "cli_token");
    }

    #[test]
    #[serial]
    fn test_get_auth_token_from_env() {
        std::env::set_var("FORC_PUB_TOKEN", "env_token");
        let result = get_auth_token(None, None).unwrap();
        std::env::remove_var("FORC_PUB_TOKEN");
        assert_eq!(result, "env_token");
    }

    #[test]
    fn test_get_auth_token_from_file() {
        let temp_dir = tempdir().unwrap();
        let cred_path = temp_dir.path().join("credentials.toml");

        let credentials = r#"
            [registry]
            token = "file_token"
        "#;
        fs::write(&cred_path, credentials).unwrap();

        let result = get_auth_token(None, Some(temp_dir.path().into())).unwrap();
        assert_eq!(result, "file_token".to_string());
    }

    #[test]
    fn test_get_auth_token_from_user_input() {
        let temp_dir = tempdir().unwrap();
        let cred_path = temp_dir.path().join("credentials.toml");

        let reader = io::Cursor::new(b"user_token");

        let result =
            get_auth_token_from_user_input(&cred_path.clone(), reader, io::sink()).unwrap();

        assert_eq!(result, "user_token");

        // Ensure the token is saved in the credentials file
        let saved_content = fs::read_to_string(&cred_path).unwrap();
        assert!(saved_content.contains("token = \"user_token\""));
    }
}

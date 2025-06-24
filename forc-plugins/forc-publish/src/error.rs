use reqwest::StatusCode;
use serde::Deserialize;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Json error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("TOML error: {0}")]
    TomlError(#[from] toml::ser::Error),

    #[error("URL error: {0}")]
    UrlError(#[from] url::ParseError),

    #[error("Failed to get relative path")]
    RelativePathError(#[from] std::path::StripPrefixError),

    #[error("{error}")]
    ApiResponseError { status: StatusCode, error: String },

    #[error("Forc.toml not found in the current directory")]
    ForcTomlNotFound,

    #[error("Invalid forc.toml: {0}")]
    InvalidForcToml(#[from] anyhow::Error),

    #[error("Project is missing a version field, add one under [project]")]
    MissingVersionField,

    #[error("Workspace is not supported yet, deploy each member seperately")]
    WorkspaceNotSupported,

    #[error("{0} is not a forc.pub dependency, depend on it using version.")]
    DependencyMissingVersion(String),

    #[error("Server error")]
    ServerError,

    #[error("Readme pre-process error: {0}")]
    MDPreProcessError(#[from] crate::md_pre_process::error::MDPreProcessError),
}

#[derive(Deserialize)]
pub struct ApiErrorResponse {
    error: String,
}

impl Error {
    /// Converts a `reqwest::Response` into an `ApiError`
    pub async fn from_response(response: reqwest::Response) -> Self {
        let status = response.status();
        match response.json::<ApiErrorResponse>().await {
            Ok(parsed_error) => Error::ApiResponseError {
                status,
                error: parsed_error.error,
            },
            Err(err) => Error::ApiResponseError {
                status,
                error: format!("Unexpected API error: {}", err),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use reqwest::StatusCode;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_error_from_response_with_valid_json() {
        let mock_server = MockServer::start().await;

        // Simulated JSON API error response
        let error_json = json!({
            "error": "Invalid request data"
        });

        Mock::given(method("POST"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(400).set_body_json(&error_json))
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/test", mock_server.uri()))
            .send()
            .await
            .unwrap();

        let error = Error::from_response(response).await;

        match error {
            Error::ApiResponseError { status, error } => {
                assert_eq!(status, StatusCode::BAD_REQUEST);
                assert_eq!(error, "Invalid request data");
            }
            _ => panic!("Expected ApiResponseError"),
        }
    }

    #[tokio::test]
    async fn test_error_from_response_with_invalid_json() {
        let mock_server = MockServer::start().await;

        // Simulated invalid JSON response (causing deserialization failure)
        let invalid_json = "not a json object";

        Mock::given(method("POST"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(500).set_body_string(invalid_json))
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/test", mock_server.uri()))
            .send()
            .await
            .unwrap();

        let error = Error::from_response(response).await;

        match error {
            Error::ApiResponseError { status, error } => {
                assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
                assert_eq!(error, "Unexpected API error: error decoding response body");
            }
            _ => panic!("Expected ApiResponseError"),
        }
    }
}

use reqwest::StatusCode;
use serde::Deserialize;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error")]
    IoError(#[from] std::io::Error),

    #[error("Json error")]
    JsonError(#[from] serde_json::Error),

    #[error("HTTP error")]
    HttpError(#[from] reqwest::Error),

    #[error("TOML error")]
    TomlError(#[from] toml::ser::Error),

    #[error("URL error")]
    UrlError(#[from] url::ParseError),

    #[error("Failed to get relative path")]
    RelativePathError(#[from] std::path::StripPrefixError),

    #[error("{error}")]
    ApiResponseError { status: StatusCode, error: String },

    #[error("Forc.toml not found in the current directory")]
    ForcTomlNotFound,
}

#[derive(Deserialize)]
pub struct ApiErrorResponse {
    error: String,
}

impl Error {
    /// Converts a `reqwest::Response` into an `ApiError`
    pub async fn from_response(response: reqwest::Response) -> Self {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        
        match serde_json::from_str::<ApiErrorResponse>(&body) {
            Ok(parsed_error) => Error::ApiResponseError {
                status,
                error: parsed_error.error,
            },
            Err(_) => Error::ApiResponseError {
                status,
                error: format!("Unexpected API error: {}", body),
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
                assert_eq!(error, "Unexpected API error: not a json object");
            }
            _ => panic!("Expected ApiResponseError"),
        }
    }
}

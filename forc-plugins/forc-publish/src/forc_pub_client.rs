use crate::error::Error;
use crate::error::Result;
use reqwest::StatusCode;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use url::Url;
use uuid::Uuid;

/// The publish request.
#[derive(Serialize, Debug)]
pub struct PublishRequest {
    pub upload_id: Uuid,
}

/// The publish response.
#[derive(Serialize, Deserialize, Debug)]
pub struct PublishResponse {
    pub name: String,
    pub version: Version,
}

/// The response to an upload_project request.
#[derive(Deserialize, Debug)]
pub struct UploadResponse {
    pub upload_id: Uuid,
}

pub struct ForcPubClient {
    client: reqwest::Client,
    uri: Url,
}

impl ForcPubClient {
    pub fn new(uri: Url) -> Self {
        let client = reqwest::Client::new();
        Self { client, uri }
    }

    /// Uploads the given file to the server
    pub async fn upload<P: AsRef<Path>>(&self, file_path: P, forc_version: &str) -> Result<Uuid> {
        use futures_util::StreamExt;
        let url = self
            .uri
            .join(&format!("upload_project?forc_version={}", forc_version))?;
        let file_bytes = fs::read(file_path)?;

        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/gzip")
            .body(file_bytes)
            .send()
            .await;

        if let Ok(response) = response {
            let mut stream = response.bytes_stream();

            // TODO: Close stream
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        let event_str = String::from_utf8_lossy(&bytes);
                        for event in event_str.split("\n\n") {
                            if event.starts_with("data:") {
                                let data = &event[5..].trim();
                                if let Ok(upload_response) =
                                    serde_json::from_str::<UploadResponse>(data)
                                {
                                    return Ok(upload_response.upload_id);
                                } else if data.starts_with("{") {
                                    // Attempt to parse error from JSON
                                    return Err(Error::ApiResponseError {
                                        status: StatusCode::INTERNAL_SERVER_ERROR,
                                        error: data.to_string(),
                                    });
                                } else {
                                    // Print the event data, replacing the previous message.
                                    print!("\r\x1b[2K  =>  {}", data);
                                    use std::io::{stdout, Write};
                                    stdout().flush().unwrap();
                                }
                            } else if event.starts_with(":") {
                                // Do nothing. These are keep-alive events.
                            }
                        }
                    }
                    Err(e) => {
                        return Err(Error::HttpError(e));
                    }
                }
            }
            Err(Error::ServerError)
        } else {
            eprintln!("Error during upload initiation: {:?}", response);
            Err(Error::ServerError)
        }
    }

    /// Publishes the given upload_id to the registry
    pub async fn publish(&self, upload_id: Uuid, auth_token: &str) -> Result<PublishResponse> {
        let url = self.uri.join("publish")?;
        let publish_request = PublishRequest { upload_id };

        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", auth_token))
            .json(&publish_request)
            .send()
            .await?;

        let status = response.status();

        if status.is_success() {
            let publish_response: PublishResponse = response.json().await?;
            Ok(publish_response)
        } else {
            Err(Error::from_response(response).await)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use reqwest::StatusCode;
    use serde_json::json;
    use std::fs;
    use tempfile::NamedTempFile;
    use uuid::Uuid;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn get_mock_client_server() -> (ForcPubClient, MockServer) {
        let mock_server = MockServer::start().await;
        let url = Url::parse(&mock_server.uri()).expect("url");
        let mock_client = ForcPubClient::new(url);
        (mock_client, mock_server)
    }

    #[tokio::test]
    async fn test_upload_success() {
        let (client, mock_server) = get_mock_client_server().await;
        let upload_id = Uuid::new_v4();
        let success_response = serde_json::json!({ "upload_id": upload_id });

        Mock::given(method("POST"))
            .and(path("/upload_project"))
            .and(query_param("forc_version", "0.66.5"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&success_response))
            .mount(&mock_server)
            .await;

        // Create a temporary gzip file
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), b"test content").unwrap();

        let result = client.upload(temp_file.path(), "0.66.5").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), upload_id);
    }

    #[tokio::test]
    async fn test_upload_server_error() {
        let (client, mock_server) = get_mock_client_server().await;

        Mock::given(method("POST"))
            .and(path("/upload_project"))
            .respond_with(
                ResponseTemplate::new(500)
                    .set_body_json(serde_json::json!({ "error": "Internal Server Error" })),
            )
            .mount(&mock_server)
            .await;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), b"test content").unwrap();

        let result = client.upload(temp_file.path(), "0.66.5").await;

        assert!(result.is_err());
        match result {
            Err(Error::ApiResponseError { status, error }) => {
                assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
                assert_eq!(error, "Internal Server Error");
            }
            _ => panic!("Expected ApiResponseError"),
        }
    }

    #[tokio::test]
    async fn test_publish_success() {
        let (client, mock_server) = get_mock_client_server().await;

        let publish_response = json!({
            "name": "test_project",
            "version": "1.0.0"
        });

        Mock::given(method("POST"))
            .and(path("/publish"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&publish_response))
            .mount(&mock_server)
            .await;

        let upload_id = Uuid::new_v4();

        let result = client.publish(upload_id, "valid_auth_token").await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.name, "test_project");
        assert_eq!(response.version.to_string(), "1.0.0");
    }

    #[tokio::test]
    async fn test_publish_unauthorized() {
        let (client, mock_server) = get_mock_client_server().await;

        Mock::given(method("POST"))
            .and(path("/publish"))
            .respond_with(ResponseTemplate::new(401).set_body_json(json!({
                "error": "Unauthorized"
            })))
            .mount(&mock_server)
            .await;

        let upload_id = Uuid::new_v4();

        let result = client.publish(upload_id, "invalid_token").await;

        assert!(result.is_err());
        match result {
            Err(Error::ApiResponseError { status, error }) => {
                assert_eq!(status, StatusCode::UNAUTHORIZED);
                assert_eq!(error, "Unauthorized");
            }
            _ => panic!("Expected ApiResponseError"),
        }
    }

    #[tokio::test]
    async fn test_publish_server_error() {
        let (client, mock_server) = get_mock_client_server().await;

        Mock::given(method("POST"))
            .and(path("/publish"))
            .respond_with(ResponseTemplate::new(500).set_body_json(json!({
                "error": "Internal Server Error"
            })))
            .mount(&mock_server)
            .await;

        let upload_id = Uuid::new_v4();

        let result = client.publish(upload_id, "valid_token").await;

        assert!(result.is_err());
        match result {
            Err(Error::ApiResponseError { status, error }) => {
                assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
                assert_eq!(error, "Internal Server Error");
            }
            _ => panic!("Expected ApiResponseError"),
        }
    }
}

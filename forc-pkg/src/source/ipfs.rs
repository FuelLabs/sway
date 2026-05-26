use crate::manifest::GenericManifestFile;
use crate::{
    manifest::{self, PackageManifestFile},
    source,
};
use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use forc_tracing::println_action_green;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    fmt,
    path::{Path, PathBuf},
    str::FromStr,
};
use tar::Archive;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Cid(pub(crate) cid::Cid);

/// Package source at a specific content address.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Source(pub Cid);

/// A pinned instance of an ipfs source
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Pinned(pub Cid);

impl Pinned {
    pub const PREFIX: &'static str = "ipfs";
}

const IPFS_DIR_NAME: &str = "ipfs";
const IPFS_CACHE_DIR_NAME: &str = "cache";
const DEFAULT_LOCAL_IPFS_API: &str = "http://127.0.0.1:5001";

impl FromStr for Cid {
    type Err = <cid::Cid as FromStr>::Err;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let cid = s.parse()?;
        Ok(Self(cid))
    }
}

impl source::Pin for Source {
    type Pinned = Pinned;
    fn pin(&self, _ctx: source::PinCtx) -> Result<(Self::Pinned, PathBuf)> {
        let cid = &self.0;
        let pinned = Pinned(cid.clone());
        let path = pkg_cache_dir(cid);
        Ok((pinned, path))
    }
}

impl source::Fetch for Pinned {
    fn fetch(&self, ctx: source::PinCtx, repo_path: &Path) -> Result<PackageManifestFile> {
        // TODO: implement local cache search for ipfs sources.
        if ctx.offline {
            anyhow::bail!("offline fetching for IPFS sources is not supported")
        }

        let mut lock = forc_util::path_lock(repo_path)?;
        // TODO: Here we assume that if the local path already exists, that it contains the
        // full and correct source for that registry entry and hasn't been tampered with. This is
        // probably fine for most cases as users should never be touching these
        // directories, however we should add some code to validate this. E.g. can we
        // recreate the ipfs cid by hashing the directory or something along these lines?
        // https://github.com/FuelLabs/sway/issues/7075
        {
            let _guard = lock.write()?;
            if !repo_path.exists() {
                println_action_green(
                    "Fetching",
                    &format!(
                        "{} {}",
                        ansiterm::Style::new().bold().paint(ctx.name()),
                        self
                    ),
                );
                let cid = self.0.clone();
                let ipfs_node = ctx.ipfs_node().clone();
                let dest = cache_dir();

                crate::source::reg::block_on_any_runtime(async move {
                    match ipfs_node {
                        source::IPFSNode::Local => {
                            println_action_green("Fetching", "with local IPFS node");
                            cid.fetch_with_local_node(&dest).await
                        }
                        source::IPFSNode::WithUrl(ipfs_node_gateway_url) => {
                            println_action_green(
                                "Fetching",
                                &format!(
                                    "from {ipfs_node_gateway_url}. Note: This can take several minutes."
                                ),
                            );
                            cid.fetch_with_gateway_url(&ipfs_node_gateway_url, &dest)
                                .await
                        }
                    }
                })?;
            }
        }
        let path = {
            let _guard = lock.read()?;
            manifest::find_within(repo_path, ctx.name()).ok_or_else(|| {
                anyhow::anyhow!("failed to find package `{}` in {}", ctx.name(), self)
            })?
        };
        PackageManifestFile::from_file(path)
    }
}

impl source::DepPath for Pinned {
    fn dep_path(&self, name: &str) -> anyhow::Result<source::DependencyPath> {
        let repo_path = pkg_cache_dir(&self.0);
        // Co-ordinate access to the ipfs checkout directory using an advisory file lock.
        let lock = forc_util::path_lock(&repo_path)?;
        let _guard = lock.read()?;
        let path = manifest::find_within(&repo_path, name)
            .ok_or_else(|| anyhow::anyhow!("failed to find package `{}` in {}", name, self))?;
        Ok(source::DependencyPath::ManifestPath(path))
    }
}

impl From<Pinned> for source::Pinned {
    fn from(p: Pinned) -> Self {
        Self::Ipfs(p)
    }
}

impl fmt::Display for Pinned {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}+{}", Self::PREFIX, self.0 .0)
    }
}

impl Cid {
    fn extract_archive<R: std::io::Read>(&self, reader: R, dst: &Path) -> Result<()> {
        let dst_dir = dst.join(self.0.to_string());
        std::fs::create_dir_all(&dst_dir)?;
        let mut archive = Archive::new(reader);

        for entry in archive.entries()? {
            let mut entry = entry?;
            entry.unpack_in(&dst_dir)?;
        }

        Ok(())
    }

    /// Using a local IPFS node, fetches the content described by this cid.
    pub(crate) async fn fetch_with_local_node(&self, dst: &Path) -> Result<()> {
        let cid_path = format!("/ipfs/{}", self.0);
        let api_base = local_ipfs_api_base_url();
        // Kubo RPC endpoints require POST. Published forc packages are stored as a single
        // gzip-compressed tar blob; `/cat` returns those bytes (same payload gateways serve).
        let url = format!(
            "{}/api/v0/cat?arg={}",
            api_base.trim_end_matches('/'),
            urlencoding::encode(&cid_path)
        );

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .send()
            .await
            .with_context(|| format!("failed to request IPFS content from {url}"))?;
        if !response.status().is_success() {
            anyhow::bail!(
                "IPFS API request to {url} failed with status {}",
                response.status()
            );
        }

        let bytes = response
            .bytes()
            .await
            .context("failed to read IPFS API response body")?;
        let tar = GzDecoder::new(bytes.as_ref());
        self.extract_archive(tar, dst)?;
        Ok(())
    }

    /// Using the provided gateway url, fetches the content described by this cid.
    pub(crate) async fn fetch_with_gateway_url(&self, gateway_url: &str, dst: &Path) -> Result<()> {
        let client = reqwest::Client::new();
        // We request the content to be served to us in tar format by the public gateway.
        let fetch_url = format!(
            "{}/ipfs/{}?download=true&filename={}.tar.gz",
            gateway_url, self.0, self.0
        );
        let req = client.get(&fetch_url);
        let res = req.send().await?;
        if !res.status().is_success() {
            anyhow::bail!("Failed to fetch from {fetch_url:?}");
        }
        let bytes: Vec<_> = res.bytes().await?.into_iter().collect();
        let tar = GzDecoder::new(bytes.as_slice());
        // After collecting and decoding bytes of the archive, we unpack it to the dst.
        self.extract_archive(tar, dst)?;
        Ok(())
    }
}

/// Returns the local IPFS HTTP API base URL, matching `IpfsClient::default()` behavior.
fn local_ipfs_api_base_url() -> String {
    read_ipfs_api_base_url_from_config().unwrap_or_else(|| DEFAULT_LOCAL_IPFS_API.to_string())
}

fn read_ipfs_api_base_url_from_config() -> Option<String> {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    let api_path = Path::new(&home).join(".ipfs").join("api");
    let contents = std::fs::read_to_string(api_path).ok()?;
    parse_ipfs_api_multiaddr(contents.trim())
}

fn parse_ipfs_api_multiaddr(addr: &str) -> Option<String> {
    let parts: Vec<&str> = addr.split('/').filter(|part| !part.is_empty()).collect();
    if parts.len() < 4 {
        return None;
    }

    let (protocol, host, transport, port) = (parts[0], parts[1], parts[2], parts[3]);
    if transport != "tcp" {
        return None;
    }

    match protocol {
        "ip4" | "ip6" | "dns4" | "dns6" => Some(format!("http://{host}:{port}")),
        _ => None,
    }
}

#[derive(Debug)]
pub enum PinnedParseError {
    Prefix,
    Cid(<cid::Cid as FromStr>::Err),
}

impl FromStr for Pinned {
    type Err = PinnedParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // ipfs+<cid>
        let s = s.trim();
        // Parse the prefix.
        let prefix_plus = format!("{}+", Self::PREFIX);
        if s.find(&prefix_plus) != Some(0) {
            return Err(PinnedParseError::Prefix);
        }
        let s = &s[prefix_plus.len()..];
        // Then the CID.
        let cid: cid::Cid = s.parse().map_err(PinnedParseError::Cid)?;
        Ok(Self(Cid(cid)))
    }
}

impl Serialize for Cid {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let cid_string: String = format!("{}", self.0);
        cid_string.serialize(s)
    }
}

impl<'de> Deserialize<'de> for Cid {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let cid_string = String::deserialize(d)?;
        let cid: cid::Cid = cid_string.parse().map_err(|e| {
            let msg = format!("failed to parse CID from {cid_string:?}: {e}");
            D::Error::custom(msg)
        })?;
        Ok(Self(cid))
    }
}

fn ipfs_dir() -> PathBuf {
    forc_util::user_forc_directory().join(IPFS_DIR_NAME)
}

fn cache_dir() -> PathBuf {
    ipfs_dir().join(IPFS_CACHE_DIR_NAME)
}

fn pkg_cache_dir(cid: &Cid) -> PathBuf {
    cache_dir().join(format!("{}", cid.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::io::Cursor;
    use tar::Header;
    use tempfile::TempDir;

    fn create_header(path: &str, size: u64) -> Header {
        let mut header = Header::new_gnu();
        header.set_path(path).unwrap();
        header.set_size(size);
        header.set_mode(0o755);
        header.set_cksum();
        header
    }

    fn create_test_tar(files: &[(&str, &str)]) -> Vec<u8> {
        let mut ar = tar::Builder::new(Vec::new());

        // Add root project directory
        let header = create_header("test-project/", 0);
        ar.append(&header, &mut std::io::empty()).unwrap();

        // Add files
        for (path, content) in files {
            let full_path = format!("test-project/{path}");
            let header = create_header(&full_path, content.len() as u64);
            ar.append(&header, content.as_bytes()).unwrap();
        }

        ar.into_inner().unwrap()
    }

    fn create_test_cid() -> Cid {
        let cid = cid::Cid::from_str("QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG").unwrap();
        Cid(cid)
    }

    #[test]
    fn test_basic_extraction() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cid = create_test_cid();

        let tar_content = create_test_tar(&[("test.txt", "hello world")]);

        cid.extract_archive(Cursor::new(tar_content), temp_dir.path())?;

        let extracted_path = temp_dir
            .path()
            .join(cid.0.to_string())
            .join("test-project")
            .join("test.txt");

        assert!(extracted_path.exists());
        assert_eq!(std::fs::read_to_string(extracted_path)?, "hello world");

        Ok(())
    }

    #[test]
    fn test_nested_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cid = create_test_cid();

        let tar_content =
            create_test_tar(&[("src/main.sw", "contract {};"), ("README.md", "# Test")]);

        cid.extract_archive(Cursor::new(tar_content), temp_dir.path())?;

        let base = temp_dir.path().join(cid.0.to_string()).join("test-project");
        assert_eq!(
            std::fs::read_to_string(base.join("src/main.sw"))?,
            "contract {};"
        );
        assert_eq!(std::fs::read_to_string(base.join("README.md"))?, "# Test");

        Ok(())
    }

    #[test]
    fn test_invalid_tar() {
        let temp_dir = TempDir::new().unwrap();
        let cid = create_test_cid();

        let result = cid.extract_archive(Cursor::new(b"not a tar file"), temp_dir.path());

        assert!(result.is_err());
    }

    #[test]
    fn test_source_ipfs_pinned_parsing() {
        let string = "ipfs+QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG";
        let expected = Pinned(Cid(cid::Cid::from_str(
            "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG",
        )
        .unwrap()));
        let parsed = Pinned::from_str(string).unwrap();
        assert_eq!(parsed, expected);
        let serialized = expected.to_string();
        assert_eq!(&serialized, string);
    }

    #[test]
    fn test_parse_ipfs_api_multiaddr() {
        assert_eq!(
            parse_ipfs_api_multiaddr("/ip4/127.0.0.1/tcp/5001").as_deref(),
            Some("http://127.0.0.1:5001")
        );
        assert_eq!(
            parse_ipfs_api_multiaddr("/dns4/localhost/tcp/5001").as_deref(),
            Some("http://localhost:5001")
        );
        assert!(parse_ipfs_api_multiaddr("/unix/path").is_none());
    }

    /// Exercises the Kubo RPC path used in production: POST `/api/v0/cat` (no daemon required).
    #[tokio::test]
    async fn test_fetch_with_local_node_post_cat() -> Result<()> {
        use std::io::Write;
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        let cid = create_test_cid();
        let tar_content =
            create_test_tar(&[("Forc.toml", "[project]\nname = \"mock-ipfs-pkg\"\n")]);
        let mut gz = Vec::new();
        {
            let mut encoder =
                flate2::write::GzEncoder::new(&mut gz, flate2::Compression::default());
            encoder.write_all(&tar_content)?;
            encoder.finish()?;
        }

        Mock::given(method("POST"))
            .and(path("/api/v0/cat"))
            .and(query_param("arg", format!("/ipfs/{}", cid.0)))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(gz))
            .expect(1)
            .mount(&mock_server)
            .await;

        let port = mock_server
            .uri()
            .rsplit(':')
            .next()
            .and_then(|p| p.parse::<u16>().ok())
            .expect("mock server port");

        let temp_home = TempDir::new()?;
        let api_file = temp_home.path().join(".ipfs").join("api");
        std::fs::create_dir_all(api_file.parent().unwrap())?;
        std::fs::write(api_file, format!("/ip4/127.0.0.1/tcp/{port}"))?;

        // Wiremock binds a random port; point `~/.ipfs/api` at it for this test only.
        unsafe { std::env::set_var("HOME", temp_home.path()) };

        let dest = TempDir::new()?;
        cid.fetch_with_local_node(dest.path()).await?;

        let forc_toml = dest
            .path()
            .join(cid.0.to_string())
            .join("test-project")
            .join("Forc.toml");
        assert!(forc_toml.exists());
        assert!(std::fs::read_to_string(forc_toml)?.contains("mock-ipfs-pkg"));

        mock_server.verify().await;
        Ok(())
    }

    #[test]
    fn test_path_traversal_prevention() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cid = create_test_cid();

        // Create a known directory structure
        let target_dir = temp_dir.path().join("target");
        std::fs::create_dir(&target_dir)?;

        // Create our canary file in a known location
        let canary_content = "sensitive content";
        let canary_path = target_dir.join("canary.txt");
        std::fs::write(&canary_path, canary_content)?;

        // Create tar with malicious path targeting our specific canary file
        let mut header = tar::Header::new_gnu();
        let malicious_path = b"../../target/canary.txt";
        header.as_gnu_mut().unwrap().name[..malicious_path.len()].copy_from_slice(malicious_path);
        header.set_size(17);
        header.set_mode(0o644);
        header.set_cksum();

        let mut ar = tar::Builder::new(Vec::new());
        ar.append(&header, b"malicious content".as_slice())?;

        // Add safe file
        let mut safe_header = tar::Header::new_gnu();
        safe_header.set_path("safe.txt")?;
        safe_header.set_size(12);
        safe_header.set_mode(0o644);
        safe_header.set_cksum();
        ar.append(&safe_header, b"safe content".as_slice())?;

        // Extract to a subdirectory of temp_dir
        let tar_content = ar.into_inner()?;
        let extract_dir = temp_dir.path().join("extract");
        std::fs::create_dir(&extract_dir)?;
        cid.extract_archive(Cursor::new(tar_content), &extract_dir)?;

        // Verify canary file was not modified
        assert_eq!(
            std::fs::read_to_string(&canary_path)?,
            canary_content,
            "Canary file was modified - path traversal protection failed!"
        );
        Ok(())
    }
}

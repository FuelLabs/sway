use crate::{
    manifest::{self, PackageManifestFile},
    source,
};
use anyhow::Result;
use futures::TryStreamExt;
use ipfs_api::IpfsApi;
use ipfs_api_backend_hyper as ipfs_api;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    fmt,
    path::{Path, PathBuf},
    str::FromStr,
};
use tar::Archive;
use tracing::info;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Cid(cid::Cid);

/// A client that can interact with local ipfs daemon.
pub type IpfsClient = ipfs_api::IpfsClient;

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

        let mut lock = crate::pkg::path_lock(repo_path)?;
        {
            let _guard = lock.write()?;
            if !repo_path.exists() {
                info!(
                    "  {} {} {}",
                    ansi_term::Color::Green.bold().paint("Fetching"),
                    ansi_term::Style::new().bold().paint(ctx.name),
                    self
                );
                let cid = &self.0;
                let ipfs_client = ipfs_client();
                let dest = cache_dir();
                futures::executor::block_on(async {
                    match ctx.ipfs_node() {
                        source::IPFSNode::Local => {
                            info!(
                                "   {} with local IPFS node",
                                ansi_term::Color::Green.bold().paint("Fetching")
                            );
                            cid.fetch_with_client(&ipfs_client, &dest).await
                        }
                        source::IPFSNode::WithUrl(ipfs_node_gateway_url) => {
                            info!(
                                "   {} from {}. Note: This can take several minutes.",
                                ansi_term::Color::Green.bold().paint("Fetching"),
                                ipfs_node_gateway_url
                            );
                            cid.fetch_with_gateway_url(ipfs_node_gateway_url, &dest)
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
        let lock = crate::pkg::path_lock(&repo_path)?;
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
    /// Using local node, fetches the content described by this cid.
    async fn fetch_with_client(&self, ipfs_client: &IpfsClient, dst: &Path) -> Result<()> {
        let cid_path = format!("/ipfs/{}", self.0);
        // Since we are fetching packages as a fodler, they are returned as a tar archive.
        let bytes = ipfs_client
            .get(&cid_path)
            .map_ok(|chunk| chunk.to_vec())
            .try_concat()
            .await?;
        // After collecting bytes of the archive, we unpack it to the dst.
        let mut archive = Archive::new(bytes.as_slice());
        archive.unpack(dst)?;
        Ok(())
    }

    /// Using the provided gateway url, fetches the content described by this cid.
    async fn fetch_with_gateway_url(&self, gateway_url: &str, dst: &Path) -> Result<()> {
        let client = reqwest::Client::new();
        // We request the content to be served to us in tar format by the public gateway.
        let fetch_url = format!(
            "{}/ipfs/{}?download=true&format=tar&filename={}.tar",
            gateway_url, self.0, self.0
        );
        let req = client.get(&fetch_url);
        let res = req.send().await?;
        if !res.status().is_success() {
            anyhow::bail!("Failed to fetch from {fetch_url:?}");
        }
        let bytes: Vec<_> = res.text().await?.bytes().collect();

        // After collecting bytes of the archive, we unpack it to the dst.
        let mut archive = Archive::new(bytes.as_slice());
        archive.unpack(dst)?;
        Ok(())
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

/// Returns a `IpfsClient` instance ready to be used to make requests to local ipfs node.
fn ipfs_client() -> IpfsClient {
    IpfsClient::default()
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

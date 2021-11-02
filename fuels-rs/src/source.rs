use std::{
    borrow::Cow,
    env, fs,
    path::{Path, PathBuf},
    str::FromStr,
};
use url::Url;

use anyhow::{anyhow, Context, Error, Result};

/// A source of a Truffle artifact JSON.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Source {
    /// A raw ABI string
    String(String),

    /// An ABI located on the local file system.
    Local(PathBuf),
    // In the future we can have an ABI to be retrieved over HTTP(S) or block explorer
    // Http(Url),
}

impl Source {
    /// Parses an ABI from a source
    ///
    /// Contract ABIs can be retrieved from the local filesystem or it can
    /// be provided in-line. It accepts:
    ///
    /// - raw ABI JSON
    ///
    /// - `relative/path/to/Contract.json`: a relative path to an ABI JSON file.
    /// This relative path is rooted in the current working directory.
    /// To specify the root for relative paths, use `Source::with_root`.
    ///
    /// - `/absolute/path/to/Contract.json` or
    ///   `file:///absolute/path/to/Contract.json`: an absolute path or file URL
    ///   to an ABI JSON file.
    pub fn parse<S>(source: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        let source = source.as_ref().trim();

        if source.starts_with('[') || source.starts_with("\n") {
            return Ok(Source::String(source.to_owned()));
        }
        let root = env::current_dir()?.canonicalize()?;
        Source::with_root(root, source)
    }

    /// Parses an artifact source from a string and a specified root directory
    /// for resolving relative paths. See `Source::with_root` for more details
    /// on supported source strings.
    fn with_root<P, S>(root: P, source: S) -> Result<Self, Error>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        let base = Url::from_directory_path(root)
            .map_err(|_| anyhow!("root path '{}' is not absolute"))?;
        let url = base.join(source.as_ref())?;

        match url.scheme() {
            "file" => Ok(Source::local(url.path())),
            // TODO: support other URL schemes (http, etc)
            _ => Err(anyhow!("unsupported URL '{}'", url)),
        }
    }

    /// Creates a local filesystem source from a path string.
    fn local<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Source::Local(path.as_ref().into())
    }

    /// Retrieves the source JSON of the artifact this will either read the JSON
    /// from the file system or retrieve a contract ABI from the network
    /// dependending on the source type.
    pub fn get(&self) -> Result<String> {
        match self {
            Source::Local(path) => get_local_contract(path),
            Source::String(abi) => Ok(abi.clone()),
        }
    }
}

fn get_local_contract(path: &Path) -> Result<String> {
    let path = if path.is_relative() {
        let absolute_path = path.canonicalize().with_context(|| {
            format!(
                "unable to canonicalize file from working dir {} with path {}",
                env::current_dir()
                    .map(|cwd| cwd.display().to_string())
                    .unwrap_or_else(|err| format!("??? ({})", err)),
                path.display(),
            )
        })?;
        Cow::Owned(absolute_path)
    } else {
        Cow::Borrowed(path)
    };

    let json = fs::read_to_string(&path).context(format!(
        "failed to read artifact JSON file with path {}",
        &path.display()
    ))?;
    Ok(json)
}

impl FromStr for Source {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Source::parse(s)
    }
}

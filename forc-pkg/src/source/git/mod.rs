mod auth;

use crate::manifest::GenericManifestFile;
use crate::{
    manifest::{self, PackageManifestFile},
    path_utils::{git_checkouts_directory, path_lock},
    source,
};
use anyhow::{anyhow, bail, Context, Result};
use forc_diagnostic::println_action_green;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::{
    collections::hash_map,
    fmt, fs,
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Url {
    url: gix_url::Url,
}

/// A git repo with a `Forc.toml` manifest at its root.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Source {
    /// The URL at which the repository is located.
    pub repo: Url,
    /// A git reference, e.g. a branch or tag.
    pub reference: Reference,
}

impl Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.repo, self.reference)
    }
}

/// Used to distinguish between types of git references.
///
/// For the most part, `Reference` is useful to refine the `refspecs` used to fetch remote
/// repositories.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum Reference {
    Branch(String),
    Tag(String),
    Rev(String),
    DefaultBranch,
}

/// A pinned instance of a git source.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct Pinned {
    /// The git source that is being pinned.
    pub source: Source,
    /// The hash to which we have pinned the source.
    pub commit_hash: String,
}

/// Error returned upon failed parsing of `Pinned::from_str`.
#[derive(Clone, Debug)]
pub enum PinnedParseError {
    Prefix,
    Url,
    Reference,
    CommitHash,
}

/// Represents the Head's commit hash and time (in seconds) from epoch
type HeadWithTime = (String, i64);

const DEFAULT_REMOTE_NAME: &str = "origin";

/// Everything needed to recognize a checkout in offline mode
///
/// Since we are omitting `.git` folder to save disk space, we need an indexing file
/// to recognize a checkout while searching local checkouts in offline mode
#[derive(Serialize, Deserialize)]
pub struct SourceIndex {
    /// Type of the git reference
    pub git_reference: Reference,
    pub head_with_time: HeadWithTime,
}

impl SourceIndex {
    pub fn new(time: i64, git_reference: Reference, commit_hash: String) -> SourceIndex {
        SourceIndex {
            git_reference,
            head_with_time: (commit_hash, time),
        }
    }
}

impl Reference {
    /// Resolves the parsed forc git reference to the associated git ID.
    pub fn resolve(&self, repo: &git2::Repository) -> Result<git2::Oid> {
        // Find the commit associated with this tag.
        fn resolve_tag(repo: &git2::Repository, tag: &str) -> Result<git2::Oid> {
            let refname = format!("refs/remotes/{DEFAULT_REMOTE_NAME}/tags/{tag}");
            let id = repo.refname_to_id(&refname)?;
            let obj = repo.find_object(id, None)?;
            let obj = obj.peel(git2::ObjectType::Commit)?;
            Ok(obj.id())
        }

        // Resolve to the target for the given branch.
        fn resolve_branch(repo: &git2::Repository, branch: &str) -> Result<git2::Oid> {
            let name = format!("{DEFAULT_REMOTE_NAME}/{branch}");
            let b = repo
                .find_branch(&name, git2::BranchType::Remote)
                .with_context(|| format!("failed to find branch `{branch}`"))?;
            b.get()
                .target()
                .ok_or_else(|| anyhow::format_err!("branch `{}` did not have a target", branch))
        }

        // Use the HEAD commit when default branch is specified.
        fn resolve_default_branch(repo: &git2::Repository) -> Result<git2::Oid> {
            let head_id =
                repo.refname_to_id(&format!("refs/remotes/{DEFAULT_REMOTE_NAME}/HEAD"))?;
            let head = repo.find_object(head_id, None)?;
            Ok(head.peel(git2::ObjectType::Commit)?.id())
        }

        // Find the commit for the given revision.
        fn resolve_rev(repo: &git2::Repository, rev: &str) -> Result<git2::Oid> {
            let obj = repo.revparse_single(rev)?;
            match obj.as_tag() {
                Some(tag) => Ok(tag.target_id()),
                None => Ok(obj.id()),
            }
        }

        match self {
            Reference::Tag(s) => {
                resolve_tag(repo, s).with_context(|| format!("failed to find tag `{s}`"))
            }
            Reference::Branch(s) => resolve_branch(repo, s),
            Reference::DefaultBranch => resolve_default_branch(repo),
            Reference::Rev(s) => resolve_rev(repo, s),
        }
    }
}

impl Pinned {
    pub const PREFIX: &'static str = "git";
}

impl source::Pin for Source {
    type Pinned = Pinned;
    fn pin(&self, ctx: source::PinCtx) -> Result<(Self::Pinned, PathBuf)> {
        // If the git source directly specifies a full commit hash, we should check
        // to see if we have a local copy. Otherwise we cannot know what commit we should pin
        // to without fetching the repo into a temporary directory.
        let pinned = if ctx.offline() {
            let (_local_path, commit_hash) =
                search_source_locally(ctx.name(), self)?.ok_or_else(|| {
                    anyhow!(
                        "Unable to fetch pkg {:?} from  {:?} in offline mode",
                        ctx.name(),
                        self.repo
                    )
                })?;
            Pinned {
                source: self.clone(),
                commit_hash,
            }
        } else if let Reference::DefaultBranch | Reference::Branch(_) = self.reference {
            // If the reference is to a branch or to the default branch we need to fetch
            // from remote even though we may have it locally. Because remote may contain a
            // newer commit.
            pin(ctx.fetch_id(), ctx.name(), self.clone())?
        } else {
            // If we are in online mode and the reference is to a specific commit (tag or
            // rev) we can first search it locally and re-use it.
            match search_source_locally(ctx.name(), self) {
                Ok(Some((_local_path, commit_hash))) => Pinned {
                    source: self.clone(),
                    commit_hash,
                },
                _ => {
                    // If the checkout we are looking for does not exists locally or an
                    // error happened during the search fetch it
                    pin(ctx.fetch_id(), ctx.name(), self.clone())?
                }
            }
        };
        let repo_path = commit_path(ctx.name(), &pinned.source.repo, &pinned.commit_hash);
        Ok((pinned, repo_path))
    }
}

impl source::Fetch for Pinned {
    fn fetch(&self, ctx: source::PinCtx, repo_path: &Path) -> Result<PackageManifestFile> {
        // Co-ordinate access to the git checkout directory using an advisory file lock.
        let mut lock = path_lock(repo_path)?;
        // TODO: Here we assume that if the local path already exists, that it contains the
        // full and correct source for that commit and hasn't been tampered with. This is
        // probably fine for most cases as users should never be touching these
        // directories, however we should add some code to validate this. E.g. can we
        // recreate the git hash by hashing the directory or something along these lines
        // using git?
        // https://github.com/FuelLabs/sway/issues/7075
        {
            let _guard = lock.write()?;
            if !repo_path.exists() {
                println_action_green(
                    "Fetching",
                    &format!("{} {}", ansiterm::Style::new().bold().paint(ctx.name), self),
                );
                fetch(ctx.fetch_id(), ctx.name(), self)?;
            }
        }
        let path = {
            let _guard = lock.read()?;
            manifest::find_within(repo_path, ctx.name())
                .ok_or_else(|| anyhow!("failed to find package `{}` in {}", ctx.name(), self))?
        };
        PackageManifestFile::from_file(path)
    }
}

impl source::DepPath for Pinned {
    fn dep_path(&self, name: &str) -> anyhow::Result<source::DependencyPath> {
        let repo_path = commit_path(name, &self.source.repo, &self.commit_hash);
        // Co-ordinate access to the git checkout directory using an advisory file lock.
        let lock = path_lock(&repo_path)?;
        let _guard = lock.read()?;
        let path = manifest::find_within(&repo_path, name)
            .ok_or_else(|| anyhow!("failed to find package `{}` in {}", name, self))?;
        Ok(source::DependencyPath::ManifestPath(path))
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let url_string = self.url.to_bstring().to_string();
        write!(f, "{url_string}")
    }
}

impl fmt::Display for Pinned {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // git+<url/to/repo>?<ref_kind>=<ref_string>#<commit>
        write!(
            f,
            "{}+{}?{}#{}",
            Self::PREFIX,
            self.source.repo,
            self.source.reference,
            self.commit_hash
        )
    }
}

impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Reference::Branch(ref s) => write!(f, "branch={s}"),
            Reference::Tag(ref s) => write!(f, "tag={s}"),
            Reference::Rev(ref _s) => write!(f, "rev"),
            Reference::DefaultBranch => write!(f, "default-branch"),
        }
    }
}

impl FromStr for Url {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let url = gix_url::Url::from_bytes(s.as_bytes().into()).map_err(|e| anyhow!("{}", e))?;
        Ok(Self { url })
    }
}

impl FromStr for Pinned {
    type Err = PinnedParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // git+<url/to/repo>?<reference>#<commit>
        let s = s.trim();

        // Check for "git+" at the start.
        let prefix_plus = format!("{}+", Self::PREFIX);
        if s.find(&prefix_plus) != Some(0) {
            return Err(PinnedParseError::Prefix);
        }
        let s = &s[prefix_plus.len()..];

        // Parse the `repo` URL.
        let repo_str = s.split('?').next().ok_or(PinnedParseError::Url)?;
        let repo = Url::from_str(repo_str).map_err(|_| PinnedParseError::Url)?;
        let s = &s[repo_str.len() + "?".len()..];

        // Parse the git reference and commit hash. This can be any of either:
        // - `branch=<branch-name>#<commit-hash>`
        // - `tag=<tag-name>#<commit-hash>`
        // - `rev#<commit-hash>`
        // - `default#<commit-hash>`
        let mut s_iter = s.split('#');
        let reference = s_iter.next().ok_or(PinnedParseError::Reference)?;
        let commit_hash = s_iter
            .next()
            .ok_or(PinnedParseError::CommitHash)?
            .to_string();
        validate_git_commit_hash(&commit_hash).map_err(|_| PinnedParseError::CommitHash)?;

        const BRANCH: &str = "branch=";
        const TAG: &str = "tag=";
        let reference = if reference.find(BRANCH) == Some(0) {
            Reference::Branch(reference[BRANCH.len()..].to_string())
        } else if reference.find(TAG) == Some(0) {
            Reference::Tag(reference[TAG.len()..].to_string())
        } else if reference == "rev" {
            Reference::Rev(commit_hash.to_string())
        } else if reference == "default-branch" {
            Reference::DefaultBranch
        } else {
            return Err(PinnedParseError::Reference);
        };

        let source = Source { repo, reference };
        Ok(Self {
            source,
            commit_hash,
        })
    }
}

impl Default for Reference {
    fn default() -> Self {
        Self::DefaultBranch
    }
}

impl From<Pinned> for source::Pinned {
    fn from(p: Pinned) -> Self {
        Self::Git(p)
    }
}

/// The name to use for a package's git repository under the user's forc directory.
fn git_repo_dir_name(name: &str, repo: &Url) -> String {
    use std::hash::{Hash, Hasher};
    fn hash_url(url: &Url) -> u64 {
        let mut hasher = hash_map::DefaultHasher::new();
        url.hash(&mut hasher);
        hasher.finish()
    }
    let repo_url_hash = hash_url(repo);
    format!("{name}-{repo_url_hash:x}")
}

fn validate_git_commit_hash(commit_hash: &str) -> Result<()> {
    const LEN: usize = 40;
    if commit_hash.len() != LEN {
        bail!(
            "invalid hash length: expected {}, found {}",
            LEN,
            commit_hash.len()
        );
    }
    if !commit_hash.chars().all(|c| c.is_ascii_alphanumeric()) {
        bail!("hash contains one or more non-ascii-alphanumeric characters");
    }
    Ok(())
}

/// A temporary directory that we can use for cloning a git-sourced package's repo and discovering
/// the current HEAD for the given git reference.
///
/// The resulting directory is:
///
/// ```ignore
/// $HOME/.forc/git/checkouts/tmp/<fetch_id>-name-<repo_url_hash>
/// ```
///
/// A unique `fetch_id` may be specified to avoid contention over the git repo directory in the
/// case that multiple processes or threads may be building different projects that may require
/// fetching the same dependency.
fn tmp_git_repo_dir(fetch_id: u64, name: &str, repo: &Url) -> PathBuf {
    let repo_dir_name = format!("{:x}-{}", fetch_id, git_repo_dir_name(name, repo));
    git_checkouts_directory().join("tmp").join(repo_dir_name)
}

/// Given a git reference, build a list of `refspecs` required for the fetch operation.
///
/// Also returns whether or not our reference implies we require fetching tags.
fn git_ref_to_refspecs(reference: &Reference) -> (Vec<String>, bool) {
    let mut refspecs = vec![];
    let mut tags = false;
    match reference {
        Reference::Branch(s) => {
            refspecs.push(format!(
                "+refs/heads/{s}:refs/remotes/{DEFAULT_REMOTE_NAME}/{s}"
            ));
        }
        Reference::Tag(s) => {
            refspecs.push(format!(
                "+refs/tags/{s}:refs/remotes/{DEFAULT_REMOTE_NAME}/tags/{s}"
            ));
        }
        Reference::Rev(s) => {
            if s.starts_with("refs/") {
                refspecs.push(format!("+{s}:{s}"));
            } else {
                // We can't fetch the commit directly, so we fetch all branches and tags in order
                // to find it.
                refspecs.push(format!(
                    "+refs/heads/*:refs/remotes/{DEFAULT_REMOTE_NAME}/*"
                ));
                refspecs.push(format!("+HEAD:refs/remotes/{DEFAULT_REMOTE_NAME}/HEAD"));
                tags = true;
            }
        }
        Reference::DefaultBranch => {
            refspecs.push(format!("+HEAD:refs/remotes/{DEFAULT_REMOTE_NAME}/HEAD"));
        }
    }
    (refspecs, tags)
}

/// Initializes a temporary git repo for the package and fetches only the reference associated with
/// the given source.
fn with_tmp_git_repo<F, O>(fetch_id: u64, name: &str, source: &Source, f: F) -> Result<O>
where
    F: FnOnce(git2::Repository) -> Result<O>,
{
    // Clear existing temporary directory if it exists.
    let repo_dir = tmp_git_repo_dir(fetch_id, name, &source.repo);
    if repo_dir.exists() {
        let _ = std::fs::remove_dir_all(&repo_dir);
    }

    // Add a guard to ensure cleanup happens if we got out of scope whether by
    // returning or panicking.
    let _cleanup_guard = scopeguard::guard(&repo_dir, |dir| {
        let _ = std::fs::remove_dir_all(dir);
    });

    let config = git2::Config::open_default().unwrap();

    // Init auth manager
    let mut auth_handler = auth::AuthHandler::default_with_config(config);

    // Setup remote callbacks
    let mut callback = git2::RemoteCallbacks::new();
    callback.credentials(move |url, username, allowed| {
        auth_handler.handle_callback(url, username, allowed)
    });

    // Initialise the repository.
    let repo = git2::Repository::init(&repo_dir)
        .map_err(|e| anyhow!("failed to init repo at \"{}\": {}", repo_dir.display(), e))?;

    // Fetch the necessary references.
    let (refspecs, tags) = git_ref_to_refspecs(&source.reference);

    // Fetch the refspecs.
    let mut fetch_opts = git2::FetchOptions::new();
    fetch_opts.remote_callbacks(callback);

    if tags {
        fetch_opts.download_tags(git2::AutotagOption::All);
    }
    let repo_url_string = source.repo.to_string();
    repo.remote_anonymous(&repo_url_string)?
        .fetch(&refspecs, Some(&mut fetch_opts), None)
        .with_context(|| {
            format!(
                "failed to fetch `{}`. Check your connection or run in `--offline` mode",
                &repo_url_string
            )
        })?;

    // Call the user function.
    let output = f(repo)?;
    Ok(output)
}

/// Pin the given git-sourced package.
///
/// This clones the repository to a temporary directory in order to determine the commit at the
/// HEAD of the given git reference.
pub fn pin(fetch_id: u64, name: &str, source: Source) -> Result<Pinned> {
    let commit_hash = with_tmp_git_repo(fetch_id, name, &source, |repo| {
        // Resolve the reference to the commit ID.
        let commit_id = source
            .reference
            .resolve(&repo)
            .with_context(|| format!("Failed to resolve manifest reference: {source}"))?;
        Ok(format!("{commit_id}"))
    })?;
    Ok(Pinned {
        source,
        commit_hash,
    })
}

/// The path to which a git package commit should be checked out.
///
/// The resulting directory is:
///
/// ```ignore
/// $HOME/.forc/git/checkouts/name-<repo_url_hash>/<commit_hash>
/// ```
///
/// where `<repo_url_hash>` is a hash of the source repository URL.
pub fn commit_path(name: &str, repo: &Url, commit_hash: &str) -> PathBuf {
    let repo_dir_name = git_repo_dir_name(name, repo);
    git_checkouts_directory()
        .join(repo_dir_name)
        .join(commit_hash)
}

/// Fetch the repo at the given git package's URL and checkout the pinned commit.
///
/// Returns the location of the checked out commit.
///
/// NOTE: This function assumes that the caller has acquired an advisory lock to co-ordinate access
/// to the git repository checkout path.
pub fn fetch(fetch_id: u64, name: &str, pinned: &Pinned) -> Result<PathBuf> {
    let path = commit_path(name, &pinned.source.repo, &pinned.commit_hash);
    // Checkout the pinned hash to the path.
    with_tmp_git_repo(fetch_id, name, &pinned.source, |repo| {
        // Change HEAD to point to the pinned commit.
        let id = git2::Oid::from_str(&pinned.commit_hash)?;
        repo.set_head_detached(id)?;

        // If the directory exists, remove it. Note that we already check for an existing,
        // cached checkout directory for re-use prior to reaching the `fetch` function.
        if path.exists() {
            let _ = fs::remove_dir_all(&path);
        }
        fs::create_dir_all(&path)?;

        // Checkout HEAD to the target directory.
        let mut checkout = git2::build::CheckoutBuilder::new();
        checkout.force().target_dir(&path);
        repo.checkout_head(Some(&mut checkout))?;

        // Fetch HEAD time and create an index
        let current_head = repo.revparse_single("HEAD")?;
        let head_commit = current_head
            .as_commit()
            .ok_or_else(|| anyhow!("Cannot get commit from {}", current_head.id()))?;
        let head_time = head_commit.time().seconds();
        let source_index = SourceIndex::new(
            head_time,
            pinned.source.reference.clone(),
            pinned.commit_hash.clone(),
        );

        // Write the index file
        fs::write(
            path.join(".forc_index"),
            serde_json::to_string(&source_index)?,
        )?;
        Ok(())
    })?;
    Ok(path)
}

/// Search local checkout dir for git sources, for non-branch git references tries to find the
/// exact match. For branch references, tries to find the most recent repo present locally with the given repo
pub(crate) fn search_source_locally(
    name: &str,
    git_source: &Source,
) -> Result<Option<(PathBuf, String)>> {
    // In the checkouts dir iterate over dirs whose name starts with `name`
    let checkouts_dir = git_checkouts_directory();
    match &git_source.reference {
        Reference::Branch(branch) => {
            // Collect repos from this branch with their HEAD time
            let repos_from_branch = collect_local_repos_with_branch(checkouts_dir, name, branch)?;
            // Get the newest repo by their HEAD commit times
            let newest_branch_repo = repos_from_branch
                .into_iter()
                .max_by_key(|&(_, (_, time))| time)
                .map(|(repo_path, (hash, _))| (repo_path, hash));
            Ok(newest_branch_repo)
        }
        _ => find_exact_local_repo_with_reference(checkouts_dir, name, &git_source.reference),
    }
}

/// Search and collect repos from checkouts_dir that are from given branch and for the given package
fn collect_local_repos_with_branch(
    checkouts_dir: PathBuf,
    package_name: &str,
    branch_name: &str,
) -> Result<Vec<(PathBuf, HeadWithTime)>> {
    let mut list_of_repos = Vec::new();
    with_search_checkouts(checkouts_dir, package_name, |repo_index, repo_dir_path| {
        // Check if the repo's HEAD commit to verify it is from desired branch
        if let Reference::Branch(branch) = repo_index.git_reference {
            if branch == branch_name {
                list_of_repos.push((repo_dir_path, repo_index.head_with_time));
            }
        }
        Ok(())
    })?;
    Ok(list_of_repos)
}

/// Search an exact reference in locally available repos
fn find_exact_local_repo_with_reference(
    checkouts_dir: PathBuf,
    package_name: &str,
    git_reference: &Reference,
) -> Result<Option<(PathBuf, String)>> {
    let mut found_local_repo = None;
    if let Reference::Tag(tag) = git_reference {
        found_local_repo = find_repo_with_tag(tag, package_name, checkouts_dir)?;
    } else if let Reference::Rev(rev) = git_reference {
        found_local_repo = find_repo_with_rev(rev, package_name, checkouts_dir)?;
    }
    Ok(found_local_repo)
}

/// Search and find the match repo between the given tag and locally available options
fn find_repo_with_tag(
    tag: &str,
    package_name: &str,
    checkouts_dir: PathBuf,
) -> Result<Option<(PathBuf, String)>> {
    let mut found_local_repo = None;
    with_search_checkouts(checkouts_dir, package_name, |repo_index, repo_dir_path| {
        // Get current head of the repo
        let current_head = repo_index.head_with_time.0;
        if let Reference::Tag(curr_repo_tag) = repo_index.git_reference {
            if curr_repo_tag == tag {
                found_local_repo = Some((repo_dir_path, current_head));
            }
        }
        Ok(())
    })?;
    Ok(found_local_repo)
}

/// Search and find the match repo between the given rev and locally available options
fn find_repo_with_rev(
    rev: &str,
    package_name: &str,
    checkouts_dir: PathBuf,
) -> Result<Option<(PathBuf, String)>> {
    let mut found_local_repo = None;
    with_search_checkouts(checkouts_dir, package_name, |repo_index, repo_dir_path| {
        // Get current head of the repo
        let current_head = repo_index.head_with_time.0;
        if let Reference::Rev(curr_repo_rev) = repo_index.git_reference {
            if curr_repo_rev == rev {
                found_local_repo = Some((repo_dir_path, current_head));
            }
        }
        Ok(())
    })?;
    Ok(found_local_repo)
}

/// Search local checkouts directory and apply the given function. This is used for iterating over
/// possible options of a given package.
fn with_search_checkouts<F>(checkouts_dir: PathBuf, package_name: &str, mut f: F) -> Result<()>
where
    F: FnMut(SourceIndex, PathBuf) -> Result<()>,
{
    for entry in fs::read_dir(checkouts_dir)? {
        let entry = entry?;
        let folder_name = entry
            .file_name()
            .into_string()
            .map_err(|_| anyhow!("invalid folder name"))?;
        if folder_name.starts_with(package_name) {
            // Search if the dir we are looking starts with the name of our package
            for repo_dir in fs::read_dir(entry.path())? {
                // Iterate over all dirs inside the `name-***` directory and try to open repo from
                // each dirs inside this one
                let repo_dir = repo_dir
                    .map_err(|e| anyhow!("Cannot find local repo at checkouts dir {}", e))?;
                if repo_dir.file_type()?.is_dir() {
                    // Get the path of the current repo
                    let repo_dir_path = repo_dir.path();
                    // Get the index file from the found path
                    if let Ok(index_file) = fs::read_to_string(repo_dir_path.join(".forc_index")) {
                        let index = serde_json::from_str(&index_file)?;
                        f(index, repo_dir_path)?;
                    }
                }
            }
        }
    }
    Ok(())
}

#[test]
fn test_source_git_pinned_parsing() {
    let strings = [
        "git+https://github.com/foo/bar?branch=baz#64092602dd6158f3e41d775ed889389440a2cd86",
        "git+https://github.com/fuellabs/sway-lib-std?tag=v0.1.0#0000000000000000000000000000000000000000",
        "git+https://some-git-host.com/owner/repo?rev#FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        "git+https://some-git-host.com/owner/repo?default-branch#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    ];

    let expected = [
        Pinned {
            source: Source {
                repo: Url::from_str("https://github.com/foo/bar").unwrap(),
                reference: Reference::Branch("baz".to_string()),
            },
            commit_hash: "64092602dd6158f3e41d775ed889389440a2cd86".to_string(),
        },
        Pinned {
            source: Source {
                repo: Url::from_str("https://github.com/fuellabs/sway-lib-std").unwrap(),
                reference: Reference::Tag("v0.1.0".to_string()),
            },
            commit_hash: "0000000000000000000000000000000000000000".to_string(),
        },
        Pinned {
            source: Source {
                repo: Url::from_str("https://some-git-host.com/owner/repo").unwrap(),
                reference: Reference::Rev("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF".to_string()),
            },
            commit_hash: "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF".to_string(),
        },
        Pinned {
            source: Source {
                repo: Url::from_str("https://some-git-host.com/owner/repo").unwrap(),
                reference: Reference::DefaultBranch,
            },
            commit_hash: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
        },
    ];

    for (&string, expected) in strings.iter().zip(&expected) {
        let parsed = Pinned::from_str(string).unwrap();
        assert_eq!(&parsed, expected);
        let serialized = expected.to_string();
        assert_eq!(&serialized, string);
    }
}

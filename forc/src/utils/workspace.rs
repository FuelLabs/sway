use std::{
    env,
    ffi::OsStr,
    path::{Component, Path, PathBuf},
};

pub fn check_workspace_membership(path: &str, name: &str) {}

pub fn root_manifest(manifest_path: Option<&Path>, cwd: &Path) -> Result<PathBuf, anyhow::Error> {
    if let Some(manifest_path) = manifest_path {
        let path = cwd.join(manifest_path);
        // In general, we try to avoid normalizing paths in Cargo,
        // but in this particular case we need it to fix #3586.
        let path = normalize_path(&path);
        if !path.ends_with("Cargo.toml") && !is_embedded(&path) {
            anyhow::bail!("the manifest-path must be a path to a Cargo.toml file")
        }
        if !path.exists() {
            anyhow::bail!("manifest path `{}` does not exist", manifest_path.display())
        }
        if path.is_dir() {
            anyhow::bail!(
                "manifest path `{}` is a directory but expected a file",
                manifest_path.display()
            )
        }
        if is_embedded(&path) {
            anyhow::bail!("embedded manifest `{}` requires `-Zscript`", path.display())
        }
        Ok(path)
    } else {
        find_root_manifest_for_wd(cwd)
    }
}

pub fn find_root_manifest_for_wd(cwd: &Path) -> Result<PathBuf, anyhow::Error> {
    let valid_forc_toml_file_name = "Forc.toml";
    let invalid_forc_toml_file_name = "forc.toml";
    let mut invalid_cargo_toml_path_exists = false;

    for current in ancestors(cwd, None) {
        let manifest = current.join(valid_forc_toml_file_name);
        if manifest.exists() {
            return Ok(manifest);
        }
        if current.join(invalid_forc_toml_file_name).exists() {
            invalid_cargo_toml_path_exists = true;
        }
    }

    if invalid_cargo_toml_path_exists {
        anyhow::bail!(
        "could not find `{}` in `{}` or any parent directory, but found cargo.toml please try to rename it to Cargo.toml",
        valid_forc_toml_file_name,
        cwd.display()
    )
    } else {
        anyhow::bail!(
            "could not find `{}` in `{}` or any parent directory",
            valid_forc_toml_file_name,
            cwd.display()
        )
    }
}
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(Component::RootDir);
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if ret.ends_with(Component::ParentDir) {
                    ret.push(Component::ParentDir);
                } else {
                    let popped = ret.pop();
                    if !popped && !ret.has_root() {
                        ret.push(Component::ParentDir);
                    }
                }
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}

pub fn is_embedded(path: &Path) -> bool {
    let ext = path.extension();
    (ext == Some(OsStr::new("rs")) ||
        // Provide better errors by not considering directories to be embedded manifests
        ext.is_none())
        && path.is_file()
}

pub fn ancestors<'a>(path: &'a Path, stop_root_at: Option<&Path>) -> PathAncestors<'a> {
    PathAncestors::new(path, stop_root_at)
}

pub struct PathAncestors<'a> {
    current: Option<&'a Path>,
    stop_at: Option<PathBuf>,
}

impl<'a> PathAncestors<'a> {
    fn new(path: &'a Path, stop_root_at: Option<&Path>) -> PathAncestors<'a> {
        // FIXME: This should be checked
        let stop_at = env::var("")
            .ok()
            .map(PathBuf::from)
            .or_else(|| stop_root_at.map(|p| p.to_path_buf()));
        PathAncestors {
            current: Some(path),
            //HACK: avoid reading `~/.cargo/config` when testing Cargo itself.
            stop_at,
        }
    }
}

impl<'a> Iterator for PathAncestors<'a> {
    type Item = &'a Path;

    fn next(&mut self) -> Option<&'a Path> {
        if let Some(path) = self.current {
            self.current = path.parent();

            if let Some(ref stop_at) = self.stop_at {
                if path == stop_at {
                    self.current = None;
                }
            }

            Some(path)
        } else {
            None
        }
    }
}

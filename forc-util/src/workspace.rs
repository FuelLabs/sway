use std::{
    env,
    path::{Path, PathBuf},
};

pub fn check_workspace_membership(path: &str, name: &str) {}

pub fn root_manifest(manifest_path: Option<&Path>, cwd: &Path) -> Result<PathBuf, anyhow::Error> {
    if let Some(manifest_path) = manifest_path {
        let path = cwd.join(manifest_path);

        if !path.ends_with("Forc.toml") && !path.is_file() {
            anyhow::bail!("the manifest-path must be a path to a Forc.toml file")
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

        Ok(path)
    } else {
        find_root_manifest_for_wd(cwd)
    }
}

pub fn find_root_manifest_for_wd(cwd: &Path) -> Result<PathBuf, anyhow::Error> {
    let valid_forc_toml_file_name = "Forc.toml";
    let invalid_forc_toml_file_name = "forc.toml";
    let mut invalid_forc_toml_path_exists = false;

    for current in ancestors(cwd, None) {
        let manifest = current.join(valid_forc_toml_file_name);
        if manifest.exists() {
            return Ok(manifest);
        }
        if current.join(invalid_forc_toml_file_name).exists() {
            invalid_forc_toml_path_exists = true;
        }
    }

    if invalid_forc_toml_path_exists {
        anyhow::bail!(
        "could not find `{}` in `{}` or any parent directory, but found forc.toml please try to rename it to Forc.toml",
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

pub fn ancestors<'a>(path: &'a Path, stop_root_at: Option<&Path>) -> PathAncestors<'a> {
    PathAncestors::new(path, stop_root_at)
}

pub struct PathAncestors<'a> {
    current: Option<&'a Path>,
    stop_at: Option<PathBuf>,
}

impl<'a> PathAncestors<'a> {
    fn new(path: &'a Path, stop_at: Option<&Path>) -> PathAncestors<'a> {
        let stop_at = stop_at.and_then(|p| {
            if p.is_absolute() {
                Some(p.to_path_buf())
            } else {
                env::current_dir().ok().map(|cwd| cwd.join(p))
            }
        });
        PathAncestors {
            current: Some(path),
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

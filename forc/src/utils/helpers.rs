use std::path::PathBuf;

// Continually go up in the file tree until a manifest (Forc.toml) is found.
pub fn find_manifest_dir(starter_path: &PathBuf) -> Option<PathBuf> {
    let mut path = std::fs::canonicalize(starter_path.clone()).ok()?;
    let empty_path = PathBuf::from("/");
    while path != empty_path {
        path.push(crate::utils::constants::MANIFEST_FILE_NAME);
        if path.exists() {
            path.pop();
            return Some(path);
        } else {
            path.pop();
            path.pop();
        }
    }
    None
}

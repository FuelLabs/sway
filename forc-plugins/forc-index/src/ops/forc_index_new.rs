use crate::cli::{InitCommand, NewCommand};
use crate::commands::init;
use std::path::Path;

pub fn init(command: NewCommand) -> anyhow::Result<()> {
    let NewCommand {
        name,
        namespace,
        path,
    } = command;

    let dir_path = Path::new(&path);
    if dir_path.exists() {
        anyhow::bail!(
            "❌ Directory \"{}\" already exists.\nIf you wish to initialise an index project inside \
            this directory, consider using `forc index init --path {}`",
            dir_path.canonicalize()?.display(),
            dir_path.display(),
        );
    } else {
        std::fs::create_dir_all(dir_path)?;
    }

    let _ = init::exec(InitCommand {
        name,
        namespace,
        path: Some(dir_path.to_path_buf()),
    });

    Ok(())
}

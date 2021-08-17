use std::{env, path::PathBuf};
mod html;

use crate::{
    cli::{BuildCommand, DocCommand},
    utils::{
        cli_error::CliError,
        helpers::{find_manifest_dir, get_sway_files, read_manifest},
    },
};

use super::forc_build;

pub fn doc(command: DocCommand) -> Result<(), CliError> {
    let build_command = BuildCommand {
        path: command.path.clone(),
        print_asm: false,
        binary_outfile: None,
        offline_mode: false,
    };

    match forc_build::build(build_command) {
        Ok(_) => {
            let project_dir = if let Some(path) = &command.path {
                PathBuf::from(path)
            } else {
                env::current_dir()?
            };

            match find_manifest_dir(&project_dir) {
                Some(manifest_dir) => {
                    let manifest = read_manifest(&manifest_dir)?;
                    let project_name = manifest.project.name;
                    let project_name = html::build_static_files(&project_name)?;
                    let files = get_sway_files(manifest_dir)?;

                    env::set_current_dir(project_name)?;

                    for file in files {
                        if let Ok(file_content) = std::fs::read_to_string(&file) {
                            if let core_lang::CompileResult::Ok {
                                value,
                                warnings: _,
                                errors: _,
                            } = core_lang::parse(&file_content)
                            {
                                html::build_from_tree(value)?;
                            }
                        }
                    }

                    Ok(())
                }
                None => Err(CliError::manifest_file_missing(project_dir)),
            }
        }
        Err(err) => Err(err.into()),
    }
}

use crate::cli::DocCommand;
use anyhow::{bail, Result};
use forc_util::find_manifest_dir;
use std::{fs, sync::Arc};
use sway_core::BuildConfig;
use sway_utils::get_sway_files;

/// Main method for `forc doc`.
pub fn doc(command: DocCommand) -> Result<()> {
    // compile the workspace documentation and store it in `target/doc`
    match find_manifest_dir(&command.manifest_path) {
        Some(path) => {
            let manifest_path = path.clone();
            let files = get_sway_files(path);

            for file in files {
                if let Ok(file_content) = fs::read_to_string(&file) {
                    let _file_content: Arc<str> = Arc::from(file_content);
                    let _build_config = BuildConfig::root_from_file_name_and_manifest_path(
                        file.clone(),
                        manifest_path.clone(),
                    );
                    // make function to retreive doc comment spans
                }
            }
            // check if a `target/doc` folder exists, if not then create one
            // organize doc comment information into `html` files

            // check if the user wants to open the doc in the browser
            if command.open {}
        }
        None => bail!("failed to locate manifest file"),
    };

    Ok(())
}

// From Cargo Doc:
//
// pub fn doc(ws: &Workspace<'_>, options: &DocOptions) -> CargoResult<()> {
//     // compile the workspace documentation and store it in `target/doc`
//     let compilation = ops::compile(ws, &options.compile_opts)?;

//     // check if the user wants to open the doc in the browser
//     if options.open_result {
//         let name = &compilation
//             .root_crate_names
//             .get(0)
//             .ok_or_else(|| anyhow::anyhow!("no crates with documentation"))?;
//         let kind = options.compile_opts.build_config.single_requested_kind()?;
//         let path = compilation.root_output[&kind]
//             .with_file_name("doc")
//             .join(&name)
//             .join("index.html");
//         if path.exists() {
//             let config_browser = {
//                 let cfg: Option<PathAndArgs> = ws.config().get("doc.browser")?;
//                 cfg.map(|path_args| (path_args.path.resolve_program(ws.config()), path_args.args))
//             };

//             let mut shell = ws.config().shell();
//             shell.status("Opening", path.display())?;
//             open_docs(&path, &mut shell, config_browser)?;
//         }
//     }

//     Ok(())
// }

// fn open_docs(
//     path: &Path,
//     shell: &mut Shell,
//     config_browser: Option<(PathBuf, Vec<String>)>,
// ) -> Result<()> {
//     let browser =
//         config_browser.or_else(|| Some((PathBuf::from(std::env::var_os("BROWSER")?), Vec::new())));

//     match browser {
//         Some((browser, initial_args)) => {
//             if let Err(e) = Command::new(&browser).args(initial_args).arg(path).status() {
//                 shell.warn(format!(
//                     "Couldn't open docs with {}: {}",
//                     browser.to_string_lossy(),
//                     e
//                 ))?;
//             }
//         }
//         None => {
//             if let Err(e) = opener::open(&path) {
//                 bail!("couldn't open docs: {e}");
//             }
//         }
//     };

//     Ok(())
// }

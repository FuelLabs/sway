use crate::cli::DocCommand;
use anyhow::Result;
use forc_pkg::{self as pkg, ManifestFile};
use std::path::PathBuf;

/// Main method for `forc doc`.
pub fn doc(command: DocCommand) -> Result<()> {
    let DocCommand {
        manifest_path,
        open: open_result,
        offline_mode: offline,
        silent_mode,
        locked,
    } = command;

    let dir = if let Some(ref path) = manifest_path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };
    let manifest = ManifestFile::from_dir(&dir)?;
    let plan = pkg::BuildPlan::from_lock_and_manifest(&manifest, locked, offline)?;

    let compilation = pkg::check(&plan, silent_mode)?;
    match compilation.value {
        Some((_parse_program, _typed_program_opt)) => {}
        None => {}
    }

    // check if the user wants to open the doc in the browser
    if open_result {}

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

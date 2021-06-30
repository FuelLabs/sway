use crate::{cli::BuildCommand, utils::helpers::find_manifest_dir};
use line_col::LineColLookup;
use source_span::{
    fmt::{Color, Formatter, Style},
    Position, Span,
};
use std::fs::File;
use std::io::{self, Write};
use termcolor::{BufferWriter, Color as TermColor, ColorChoice, ColorSpec, WriteColor};

use crate::utils::constants;
use crate::utils::manifest::{Dependency, DependencyDetails, Manifest};
use core_lang::{
    BuildConfig, BytecodeCompilationResult, CompilationResult, FinalizedAsm, LibraryExports,
    Namespace,
};

use anyhow::{anyhow, Context, Result};
use curl::easy::Easy;
use dirs::home_dir;
use flate2::read::GzDecoder;
use std::{fs, io::Cursor, path::Path, path::PathBuf, str};
use tar::Archive;

pub fn build(command: BuildCommand) -> Result<Vec<u8>, String> {
    let BuildCommand {
        path,
        binary_outfile,
        print_asm,
        offline_mode,
    } = command;
    // find manifest directory, even if in subdirectory
    let this_dir = if let Some(path) = path {
        PathBuf::from(path)
    } else {
        std::env::current_dir().unwrap()
    };
    let manifest_dir = match find_manifest_dir(&this_dir) {
        Some(dir) => dir,
        None => {
            return Err(format!(
                "No manifest file found in this directory or any parent directories of it: {:?}",
                this_dir
            ))
        }
    };
    let build_config = BuildConfig::root_from_manifest_path(manifest_dir.clone());
    let mut manifest = read_manifest(&manifest_dir)?;

    let mut namespace: Namespace = Default::default();
    if let Some(ref mut deps) = manifest.dependencies {
        for (dependency_name, dependency_details) in deps.iter_mut() {
            // Check if dependency is a git-based dependency.
            let dep = match dependency_details {
                Dependency::Simple(..) => {
                    return Err(
                        "Not yet implemented: Simple version-spec dependencies require a registry."
                            .into(),
                    );
                }
                Dependency::Detailed(dep_details) => dep_details,
            };

            // Download a non-local dependency if the `git` property is set in this dependency.
            if let Some(_) = dep.git {
                let downloaded_dep_path = match download_github_dep(
                    dependency_name,
                    dep.git.as_ref().unwrap(),
                    &dep.branch,
                    &dep.version,
                    offline_mode,
                ) {
                    Ok(path) => path,
                    Err(e) => {
                        return Err(format!(
                            "Couldn't download dependency ({:?}): {:?}",
                            dependency_name, e
                        ))
                    }
                };

                // Mutate this dependency's path to hold the newly downloaded dependency's path.
                dep.path = Some(downloaded_dep_path);
            }

            compile_dependency_lib(
                &this_dir,
                &dependency_name,
                &dependency_details,
                &mut namespace,
            )?;
        }
    }

    // now, compile this program with all of its dependencies
    let main_file = get_main_file(&manifest, &manifest_dir)?;
    if print_asm {
        let main = compile_to_asm(
            main_file,
            &manifest.project.name,
            &namespace,
            build_config.clone(),
        )?;
        println!("{}", main);
    }

    let main = compile(main_file, &manifest.project.name, &namespace, build_config)?;
    if let Some(outfile) = binary_outfile {
        let mut file = File::create(outfile).map_err(|e| e.to_string())?;
        file.write_all(main.as_slice()).map_err(|e| e.to_string())?;
    }

    println!("Bytecode size is {} bytes.", main.len());

    Ok(main)
}

/// Downloads a non-local dependency that's hosted on GitHub.
/// By default, it stores the dependency in `~/.forc/`.
/// A given dependency `dep` is stored under `~/.forc/dep/default/$owner-$repo-$hash`.
/// If no hash (nor any other type of reference) is provided, Forc
/// will download the default branch at the latest commit.
/// If a branch is specified, it will go in `~/.forc/dep/$branch/$owner-$repo-$hash.
/// If a version is specified, it will go in `~/.forc/dep/$version/$owner-$repo-$hash.
/// Version takes precedence over branch reference.
fn download_github_dep(
    dep_name: &String,
    repo_base_url: &str,
    branch: &Option<String>,
    version: &Option<String>,
    offline_mode: bool,
) -> Result<String> {
    let home_dir = match home_dir() {
        None => return Err(anyhow!("Couldn't find home directory (`~/`)")),
        Some(p) => p.to_str().unwrap().to_owned(),
    };

    // Version tag takes precedence over branch reference.
    let out_dir = match &version {
        Some(v) => format!(
            "{}/{}/{}/{}",
            home_dir,
            constants::FORC_DEPENDENCIES_DIRECTORY,
            dep_name,
            v
        ),
        // If no version specified, check if a branch was specified
        None => match &branch {
            Some(b) => format!(
                "{}/{}/{}/{}",
                home_dir,
                constants::FORC_DEPENDENCIES_DIRECTORY,
                dep_name,
                b
            ),
            // If no version and no branch, use default
            None => format!(
                "{}/{}/{}/default",
                home_dir,
                constants::FORC_DEPENDENCIES_DIRECTORY,
                dep_name
            ),
        },
    };

    // Check if dependency is already installed, if so, return its path.
    if Path::new(&out_dir).exists() {
        for entry in fs::read_dir(&out_dir)? {
            let path = entry?.path();
            // If the path to that dependency at that branch/version already
            // exists and there's a directory inside of it,
            // this directory should be the installation path.

            if path.is_dir() {
                return Ok(path.to_str().unwrap().to_string());
            }
        }
    }

    // If offline mode is enabled, don't proceed as it will
    // make use of the network to download the dependency from
    // GitHub.
    // If it's offline mode and the dependency already exists
    // locally, then it would've been returned in the block above.
    if offline_mode {
        return Err(anyhow!(
            "Can't build dependency: dependency {} doesn't exist locally and offline mode is enabled",
            dep_name
        ));
    }

    let github_api_url = build_github_api_url(repo_base_url, &branch, &version);

    println!("Downloading {:?} into {:?}", dep_name, out_dir);

    let downloaded_dir = download_tarball(&github_api_url, &out_dir).unwrap();

    Ok(downloaded_dir)
}

/// Builds a proper URL that's used to call GitHub's API.
/// The dependency is specified as `https://github.com/:owner/:project`
/// And the API URL must be like `https://api.github.com/repos/:owner/:project/tarball`
/// Adding a `:ref` at the end makes it download a branch/tag based repo.
/// Omitting it makes it download the default branch at latest commit.
fn build_github_api_url(
    dependency_url: &str,
    branch: &Option<String>,
    version: &Option<String>,
) -> String {
    let mut pieces = dependency_url.rsplit("/");

    let project_name: &str = match pieces.next() {
        Some(p) => p.into(),
        None => dependency_url.into(),
    };

    let owner_name: &str = match pieces.next() {
        Some(p) => p.into(),
        None => dependency_url.into(),
    };

    // Version tag takes precedence over branch reference.
    match version {
        Some(v) => {
            format!(
                "https://api.github.com/repos/{}/{}/tarball/{}",
                owner_name, project_name, v
            )
        }
        // If no version specified, check if a branch was specified
        None => match branch {
            Some(b) => {
                format!(
                    "https://api.github.com/repos/{}/{}/tarball/{}",
                    owner_name, project_name, b
                )
            }
            // If no version and no branch, download default branch at latest commit
            None => {
                format!(
                    "https://api.github.com/repos/{}/{}/tarball",
                    owner_name, project_name
                )
            }
        },
    }
}

fn download_tarball(url: &str, out_dir: &str) -> Result<String> {
    let mut data = Vec::new();
    let mut handle = Easy::new();

    // Download the tarball.
    handle.url(url).context("failed to configure tarball URL")?;
    handle
        .follow_location(true)
        .context("failed to configure follow location")?;

    handle
        .useragent("forc-builder")
        .context("failed to configure User-Agent")?;
    {
        let mut transfer = handle.transfer();
        transfer
            .write_function(|new_data| {
                data.extend_from_slice(new_data);
                Ok(new_data.len())
            })
            .context("failed to write download data")?;
        transfer.perform().context("failed to download tarball")?;
    }

    // Unpack the tarball.
    Archive::new(GzDecoder::new(Cursor::new(data)))
        .unpack(out_dir)
        .context("failed to unpack tarball")?;

    for entry in fs::read_dir(out_dir)? {
        let path = entry?.path();
        match path.is_dir() {
            true => return Ok(path.to_str().unwrap().to_string()),
            false => (),
        }
    }

    Err(anyhow!("couldn't find downloaded dependency"))
}

/// Takes a dependency and returns a namespace of exported things from that dependency
/// trait implementations are included as well
fn compile_dependency_lib<'source, 'manifest>(
    project_path: &PathBuf,
    dependency_name: &'manifest str,
    dependency_lib: &Dependency,
    namespace: &mut Namespace<'source>,
) -> Result<(), String> {
    let dep_path = match dependency_lib {
        Dependency::Simple(..) => {
            return Err(
                "Not yet implemented: Simple version-spec dependencies require a registry.".into(),
            )
        }
        Dependency::Detailed(DependencyDetails { path, .. }) => path,
    };

    let dep_path =
        match dep_path {
            Some(p) => p,
            None => return Err(
                "Only simple path imports are supported right now. Please supply a path relative \
                 to the manifest file."
                    .into(),
            ),
        };

    // dependency paths are relative to the path of the project being compiled
    let mut project_path = project_path.clone();
    project_path.push(dep_path);

    // compile the dependencies of this dependency
    // this should detect circular dependencies
    let manifest_dir = match find_manifest_dir(&project_path) {
        Some(o) => o,
        None => return Err("Manifest not found for dependency.".into()),
    };

    let build_config = BuildConfig::root_from_manifest_path(manifest_dir.clone());

    let manifest_of_dep = read_manifest(&manifest_dir)?;

    // The part below here is just a massive shortcut to get the standard library working
    if let Some(ref deps) = manifest_of_dep.dependencies {
        if deps.len() > 0 {
            // to do this properly, iterate over list of dependencies make sure there are no
            // circular dependencies
            return Err("Unimplemented: dependencies that have dependencies".into());
        }
    }

    let main_file = get_main_file(&manifest_of_dep, &manifest_dir)?;

    let compiled = compile_library(
        main_file,
        &manifest_of_dep.project.name,
        &namespace.clone(),
        build_config.clone(),
    )?;

    namespace.insert_module(dependency_name.to_string(), compiled.namespace);

    // nothing is returned from this method since it mutates the hashmaps it was given
    Ok(())
}

fn read_manifest(manifest_dir: &PathBuf) -> Result<Manifest, String> {
    let manifest_path = {
        let mut man = manifest_dir.clone();
        man.push(crate::utils::constants::MANIFEST_FILE_NAME);
        man
    };
    let manifest_path_str = format!("{:?}", manifest_path);
    let manifest = match std::fs::read_to_string(manifest_path) {
        Ok(o) => o,
        Err(e) => {
            return Err(format!(
                "failed to read manifest at {:?}: {}",
                manifest_path_str, e
            ))
        }
    };
    match toml::from_str(&manifest) {
        Ok(o) => Ok(o),
        Err(e) => Err(format!("Error parsing manifest: {}.", e)),
    }
}

fn compile_library<'source, 'manifest>(
    source: &'source str,
    proj_name: &str,
    namespace: &Namespace<'source>,
    build_config: BuildConfig,
) -> Result<LibraryExports<'source>, String> {
    let res = core_lang::compile_to_asm(&source, namespace, build_config);
    match res {
        CompilationResult::Library { exports, warnings } => {
            for ref warning in warnings.iter() {
                format_warning(warning);
            }
            if warnings.is_empty() {
                let _ = write_green(&format!("Compiled library {:?}.", proj_name));
            } else {
                let _ = write_yellow(&format!(
                    "Compiled library {:?} with {} {}.",
                    proj_name,
                    warnings.len(),
                    if warnings.len() > 1 {
                        "warnings"
                    } else {
                        "warning"
                    }
                ));
            }
            Ok(exports)
        }
        CompilationResult::Failure { errors, warnings } => {
            let e_len = errors.len();

            for ref warning in warnings.iter() {
                format_warning(warning);
            }

            errors.into_iter().for_each(|e| format_err(e));

            write_red(format!(
                "Aborting due to {} {}.",
                e_len,
                if e_len > 1 { "errors" } else { "error" }
            ))
            .unwrap();
            Err(format!("Failed to compile {}", proj_name))
        }
        _ => {
            return Err(format!(
                "Project \"{}\" was included as a dependency but it is not a library.",
                proj_name
            ))
        }
    }
}

fn compile<'source, 'manifest>(
    source: &'source str,
    proj_name: &str,
    namespace: &Namespace<'source>,
    build_config: BuildConfig,
) -> Result<Vec<u8>, String> {
    let res = core_lang::compile_to_bytecode(&source, namespace, build_config);
    match res {
        BytecodeCompilationResult::Success { bytes, warnings } => {
            for ref warning in warnings.iter() {
                format_warning(warning);
            }
            if warnings.is_empty() {
                let _ = write_green(&format!("Compiled script {:?}.", proj_name));
            } else {
                let _ = write_yellow(&format!(
                    "Compiled script {:?} with {} {}.",
                    proj_name,
                    warnings.len(),
                    if warnings.len() > 1 {
                        "warnings"
                    } else {
                        "warning"
                    }
                ));
            }
            Ok(bytes)
        }
        BytecodeCompilationResult::Library { warnings } => {
            for ref warning in warnings.iter() {
                format_warning(warning);
            }
            if warnings.is_empty() {
                let _ = write_green(&format!("Compiled library {:?}.", proj_name));
            } else {
                let _ = write_yellow(&format!(
                    "Compiled library {:?} with {} {}.",
                    proj_name,
                    warnings.len(),
                    if warnings.len() > 1 {
                        "warnings"
                    } else {
                        "warning"
                    }
                ));
            }
            Ok(vec![])
        }
        BytecodeCompilationResult::Failure { errors, warnings } => {
            let e_len = errors.len();

            for ref warning in warnings.iter() {
                format_warning(warning);
            }

            errors.into_iter().for_each(|e| format_err(e));

            write_red(format!(
                "Aborting due to {} {}.",
                e_len,
                if e_len > 1 { "errors" } else { "error" }
            ))
            .unwrap();
            Err(format!("Failed to compile {}", proj_name))
        }
    }
}

fn format_warning(err: &core_lang::CompileWarning) {
    let input = err.span.input();
    let chars = input.chars().map(|x| -> Result<_, ()> { Ok(x) });

    let metrics = source_span::DEFAULT_METRICS;
    let buffer = source_span::SourceBuffer::new(chars, Position::default(), metrics);

    let mut fmt = Formatter::with_margin_color(Color::Blue);

    for c in buffer.iter() {
        let _ = c.unwrap(); // report eventual errors.
    }

    let (start_pos, end_pos) = err.span();
    let lookup = LineColLookup::new(input);
    let (start_line, start_col) = lookup.get(start_pos);
    let (end_line, end_col) = lookup.get(end_pos - 1);

    let err_start = Position::new(start_line - 1, start_col - 1);
    let err_end = Position::new(end_line - 1, end_col - 1);
    let err_span = Span::new(err_start, err_end, err_end.next_column());
    fmt.add(
        err_span,
        Some(err.to_friendly_warning_string()),
        Style::Warning,
    );

    let formatted = fmt.render(buffer.iter(), buffer.span(), &metrics).unwrap();

    println!("{}", formatted);
}

fn format_err(err: core_lang::CompileError) {
    let input = err.pest_span().input();
    let chars = input.chars().map(|x| -> Result<_, ()> { Ok(x) });

    let metrics = source_span::DEFAULT_METRICS;
    let buffer = source_span::SourceBuffer::new(chars, Position::default(), metrics);

    let mut fmt = Formatter::with_margin_color(Color::Blue);

    for c in buffer.iter() {
        let _ = c.unwrap(); // report eventual errors.
    }

    let (start_pos, end_pos) = err.span();
    let lookup = LineColLookup::new(input);
    let (start_line, start_col) = lookup.get(start_pos);
    let (end_line, end_col) = lookup.get(if end_pos == 0 { 0 } else { end_pos - 1 });

    let err_start = Position::new(start_line - 1, start_col - 1);
    let err_end = Position::new(end_line - 1, end_col - 1);
    let err_span = Span::new(err_start, err_end, err_end.next_column());
    fmt.add(err_span, Some(err.to_friendly_error_string()), Style::Error);

    let formatted = fmt.render(buffer.iter(), buffer.span(), &metrics).unwrap();
    fmt.add(
        buffer.span(),
        Some("this is the whole program\nwhat a nice program!".to_string()),
        Style::Error,
    );

    println!("{}", formatted);
}

fn write_red(txt: String) -> io::Result<()> {
    let txt = txt.as_str();
    let bufwtr = BufferWriter::stderr(ColorChoice::Always);
    let mut buffer = bufwtr.buffer();
    buffer.set_color(ColorSpec::new().set_fg(Some(TermColor::Red)))?;
    writeln!(&mut buffer, "{}", txt)?;
    bufwtr.print(&buffer)?;
    buffer.set_color(ColorSpec::new().set_fg(Some(TermColor::White)))?;
    Ok(())
}

fn write_green(txt: &str) -> io::Result<()> {
    let bufwtr = BufferWriter::stderr(ColorChoice::Always);
    let mut buffer = bufwtr.buffer();
    buffer.set_color(ColorSpec::new().set_fg(Some(TermColor::Green)))?;
    writeln!(&mut buffer, "{}", txt)?;
    bufwtr.print(&buffer)?;
    buffer.set_color(ColorSpec::new().set_fg(Some(TermColor::White)))?;
    Ok(())
}

fn write_yellow(txt: &str) -> io::Result<()> {
    let bufwtr = BufferWriter::stderr(ColorChoice::Always);
    let mut buffer = bufwtr.buffer();
    buffer.set_color(ColorSpec::new().set_fg(Some(TermColor::Yellow)))?;
    writeln!(&mut buffer, "{}", txt)?;
    bufwtr.print(&buffer)?;
    buffer.set_color(ColorSpec::new().set_fg(Some(TermColor::White)))?;
    Ok(())
}

fn get_main_file(
    manifest_of_dep: &Manifest,
    manifest_dir: &PathBuf,
) -> Result<&'static mut String, String> {
    let main_path = {
        let mut code_dir = manifest_dir.clone();
        code_dir.push("src");
        code_dir.push(&manifest_of_dep.project.entry);
        code_dir
    };

    // some hackery to get around lifetimes for now, until the AST returns a non-lifetime-bound AST
    let main_file = fs::read_to_string(&main_path).map_err(|e| e.to_string())?;
    let main_file = Box::new(main_file);
    let main_file: &'static mut String = Box::leak(main_file);
    return Ok(main_file);
}
fn compile_to_asm<'source, 'manifest>(
    source: &'source str,
    proj_name: &str,
    namespace: &Namespace<'source>,
    build_config: BuildConfig,
) -> Result<FinalizedAsm<'source>, String> {
    let res = core_lang::compile_to_asm(&source, namespace, build_config);
    match res {
        CompilationResult::Success { asm, warnings } => {
            for ref warning in warnings.iter() {
                format_warning(warning);
            }
            if warnings.is_empty() {
                let _ = write_green(&format!("Compiled script {:?}.", proj_name));
            } else {
                let _ = write_yellow(&format!(
                    "Compiled script {:?} with {} {}.",
                    proj_name,
                    warnings.len(),
                    if warnings.len() > 1 {
                        "warnings"
                    } else {
                        "warning"
                    }
                ));
            }
            Ok(asm)
        }
        CompilationResult::Library { warnings, .. } => {
            for ref warning in warnings.iter() {
                format_warning(warning);
            }
            if warnings.is_empty() {
                let _ = write_green(&format!("Compiled library {:?}.", proj_name));
            } else {
                let _ = write_yellow(&format!(
                    "Compiled library {:?} with {} {}.",
                    proj_name,
                    warnings.len(),
                    if warnings.len() > 1 {
                        "warnings"
                    } else {
                        "warning"
                    }
                ));
            }
            Ok(FinalizedAsm::Library)
        }
        CompilationResult::Failure { errors, warnings } => {
            let e_len = errors.len();

            for ref warning in warnings.iter() {
                format_warning(warning);
            }

            errors.into_iter().for_each(|e| format_err(e));

            write_red(format!(
                "Aborting due to {} {}.",
                e_len,
                if e_len > 1 { "errors" } else { "error" }
            ))
            .unwrap();
            Err(format!("Failed to compile {}", proj_name))
        }
    }
}

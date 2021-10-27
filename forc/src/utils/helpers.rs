use super::constants::{SRC_DIR, SWAY_EXTENSION};
use super::manifest::Manifest;
use source_span::fmt::{Color, Formatter};
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::{fs, str};
use termcolor::{self, Color as TermColor, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn is_sway_file(file: &Path) -> bool {
    let res = file.extension();
    Some(OsStr::new(SWAY_EXTENSION)) == res
}

pub fn get_sway_files(path: PathBuf) -> Vec<PathBuf> {
    let mut files = vec![];
    let mut dir_entries = vec![path];

    while let Some(next_dir) = dir_entries.pop() {
        if let Ok(read_dir) = fs::read_dir(next_dir) {
            for inner_entry in read_dir {
                if let Ok(entry) = inner_entry {
                    let path = entry.path();

                    if path.is_dir() {
                        dir_entries.push(path);
                    } else {
                        if is_sway_file(&path) {
                            files.push(path)
                        }
                    }
                }
            }
        }
    }

    files
}

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

pub fn read_manifest(manifest_dir: &PathBuf) -> Result<Manifest, String> {
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

pub fn get_main_file(
    manifest_of_dep: &Manifest,
    manifest_dir: &PathBuf,
) -> Result<&'static mut String, String> {
    let main_path = {
        let mut code_dir = manifest_dir.clone();
        code_dir.push(SRC_DIR);
        code_dir.push(&manifest_of_dep.project.entry);
        code_dir
    };

    // some hackery to get around lifetimes for now, until the AST returns a non-lifetime-bound AST
    let main_file = std::fs::read_to_string(&main_path).map_err(|e| e.to_string())?;
    let main_file = Box::new(main_file);
    let main_file: &'static mut String = Box::leak(main_file);
    return Ok(main_file);
}

pub fn get_main_path<'sc>(manifest: &Manifest, manifest_dir: &PathBuf) -> PathBuf {
    let mut code_dir = manifest_dir.clone();
    code_dir.push(crate::utils::constants::SRC_DIR);
    code_dir.push(&manifest.project.entry);
    code_dir
}

pub fn get_file_name<'sc>(
    manifest_dir: &PathBuf,
    main_path: &'sc PathBuf,
) -> Result<&'sc Path, String> {
    let mut file_path = manifest_dir.clone();
    file_path.pop();
    match main_path.strip_prefix(file_path.clone()) {
        Ok(o) => Ok(o.clone()),
        Err(err) => Err(err.to_string()),
    }
}

pub fn println_red(txt: &str) -> io::Result<()> {
    println_std_out(txt, TermColor::Red)
}

pub fn println_green(txt: &str) -> io::Result<()> {
    println_std_out(txt, TermColor::Green)
}

pub fn print_blue_err(txt: &str) -> io::Result<()> {
    print_std_err(txt, TermColor::Blue)
}

pub fn println_yellow_err(txt: &str) -> io::Result<()> {
    println_std_err(txt, TermColor::Yellow)
}

pub fn println_red_err(txt: &str) -> io::Result<()> {
    println_std_err(txt, TermColor::Red)
}

pub fn println_green_err(txt: &str) -> io::Result<()> {
    println_std_err(txt, TermColor::Green)
}

fn print_std_out(txt: &str, color: TermColor) -> io::Result<()> {
    let stdout = StandardStream::stdout(ColorChoice::Always);
    print_with_color(txt, color, stdout)
}

fn println_std_out(txt: &str, color: TermColor) -> io::Result<()> {
    let stdout = StandardStream::stdout(ColorChoice::Always);
    println_with_color(txt, color, stdout)
}

fn print_std_err(txt: &str, color: TermColor) -> io::Result<()> {
    let stdout = StandardStream::stderr(ColorChoice::Always);
    print_with_color(txt, color, stdout)
}

fn println_std_err(txt: &str, color: TermColor) -> io::Result<()> {
    let stdout = StandardStream::stderr(ColorChoice::Always);
    println_with_color(txt, color, stdout)
}

fn print_with_color(txt: &str, color: TermColor, stream: StandardStream) -> io::Result<()> {
    let mut stream = stream;
    stream.set_color(ColorSpec::new().set_fg(Some(color)))?;
    write!(&mut stream, "{}", txt)?;
    stream.reset()?;
    Ok(())
}

fn println_with_color(txt: &str, color: TermColor, stream: StandardStream) -> io::Result<()> {
    let mut stream = stream;
    stream.set_color(ColorSpec::new().set_fg(Some(color)))?;
    writeln!(&mut stream, "{}", txt)?;
    stream.reset()?;
    Ok(())
}

pub fn format_err(err: &core_lang::CompileError) {
    let mut fmt = Formatter::with_margin_color(Color::Blue);
    let formatted = err.format(&mut fmt);
    print_blue_err(" --> ").unwrap();
    print!("{}", err.path());
    println!("{}", formatted);
}

pub fn format_warning(warning: &core_lang::CompileWarning) {
    let mut fmt = Formatter::with_margin_color(Color::Blue);
    let formatted = warning.format(&mut fmt);
    print_blue_err(" --> ").unwrap();
    print!("{}", warning.path());
    println!("{}", formatted);
}

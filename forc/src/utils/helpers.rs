use super::constants::SRC_DIR;
use super::manifest::Manifest;
use std::io::{self, Write};
use std::{path::PathBuf, str};
use termcolor::{self, Color as TermColor, ColorChoice, ColorSpec, StandardStream, WriteColor};

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

pub fn print_red(txt: &str) -> io::Result<()> {
    print_std_out(txt, TermColor::Red)
}

pub fn print_green(txt: &str) -> io::Result<()> {
    print_std_out(txt, TermColor::Green)
}

pub fn print_yellow_err(txt: &str) -> io::Result<()> {
    print_std_err(txt, TermColor::Yellow)
}

pub fn print_red_err(txt: &str) -> io::Result<()> {
    print_std_err(txt, TermColor::Red)
}

pub fn print_green_err(txt: &str) -> io::Result<()> {
    print_std_err(txt, TermColor::Green)
}

fn print_std_out(txt: &str, color: TermColor) -> io::Result<()> {
    let stdout = StandardStream::stdout(ColorChoice::Always);
    print_with_color(txt, color, stdout)
}

fn print_std_err(txt: &str, color: TermColor) -> io::Result<()> {
    let stdout = StandardStream::stderr(ColorChoice::Always);
    print_with_color(txt, color, stdout)
}

fn print_with_color(txt: &str, color: TermColor, stream: StandardStream) -> io::Result<()> {
    let mut stream = stream;
    stream.set_color(ColorSpec::new().set_fg(Some(color)))?;
    writeln!(&mut stream, "{}", txt)?;
    stream.reset()?;
    Ok(())
}

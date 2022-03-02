use crate::cli::InitCommand;
use crate::utils::defaults;
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use sway_utils::constants;
use url::Url;

#[derive(Debug)]
struct GitPathInfo {
    owner: String,
    repo_name: String,
    example_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum FileType {
    File,
    Dir,
}

#[derive(serde::Deserialize, Debug)]
struct Links {
    git: String,
    html: String,
    #[serde(rename = "self")]
    cur: String,
}

#[derive(serde::Deserialize, Debug)]
struct ContentResponse {
    #[serde(rename = "_links")]
    links: Links,
    download_url: Option<String>,
    git_url: String,
    html_url: String,
    name: String,
    path: String,
    sha: String,
    size: u64,
    #[serde(rename = "type")]
    file_type: FileType,
    url: String,
}

pub fn init(command: InitCommand) -> Result<(), String> {
    let project_name = command.project_name;

    match command.template {
        Some(template) => {
            let template_url = match template.as_str() {
                "counter" => {
                    Url::parse("https://github.com/FuelLabs/sway/tree/master/examples/hello_world")
                        .unwrap()
                }
                _ => {
                    if template.contains("https") {
                        Url::parse(&template)
                            .map_err(|e| format!("Not a valid URL {}", e))
                            .unwrap()
                    } else {
                        Url::from_file_path(&template)
                            .map_err(|()| "Not a valid local path".to_string())
                            .unwrap()
                    }
                }
            };
            match template_url.host() {
                Some(_) => {
                    init_from_git_template(project_name, &template_url).map_err(|e| e.to_string())
                }
                None => {
                    init_from_local_template(project_name, &template_url).map_err(|e| e.to_string())
                }
            }
        }
        None => init_new_project(project_name).map_err(|e| e.to_string()),
    }
}

pub(crate) fn init_new_project(project_name: String) -> Result<()> {
    let neat_name: String = project_name.split('/').last().unwrap().to_string();

    // Make a new directory for the project
    fs::create_dir_all(Path::new(&project_name).join("src"))?;

    // Make directory for tests
    fs::create_dir_all(Path::new(&project_name).join("tests"))?;

    // Insert default manifest file
    fs::write(
        Path::new(&project_name).join(constants::MANIFEST_FILE_NAME),
        defaults::default_manifest(&neat_name),
    )?;

    // Insert default test manifest file
    fs::write(
        Path::new(&project_name).join(constants::TEST_MANIFEST_FILE_NAME),
        defaults::default_tests_manifest(&neat_name),
    )?;

    // Insert default main function
    fs::write(
        Path::new(&project_name).join("src").join("main.sw"),
        defaults::default_program(),
    )?;

    // Insert default test function
    fs::write(
        Path::new(&project_name).join("tests").join("harness.rs"),
        defaults::default_test_program(),
    )?;

    // Ignore default `out` and `target` directories created by forc and cargo.
    fs::write(
        Path::new(&project_name).join(".gitignore"),
        defaults::default_gitignore(),
    )?;

    Ok(())
}


pub(crate) fn init_from_git_template(project_name: String, example_url: &Url) -> Result<()> {
    let git = parse_github_link(example_url)?;

    let custom_url = format!(
        "https://api.github.com/repos/{}/{}/contents/{}",
        git.owner, git.repo_name, git.example_name
    );

    // Get the path of the example we are using
    let path = std::env::current_dir()?;
    let out_dir = path.join(&project_name);
    let real_name = whoami::realname();

    let responses: Vec<ContentResponse> = ureq::get(&custom_url).call()?.into_json()?;

    // Iterate through the responses to check that the link is a valid sway project
    // by checking for a Forc.toml file. Otherwise, return an error
    let valid_sway_project = responses
        .iter()
        .any(|response| response.name == "Forc.toml");
    if !valid_sway_project {
        return Err(anyhow!(
            "The provided github URL: {} does not contain a Forc.toml file at the root",
            example_url
        ));
    }

    // Download the files and directories from the github example
    download_contents(&custom_url, &out_dir, &responses)
        .with_context(|| format!("couldn't download from: {}", &custom_url))?;

    // Change the project name and author of the Forc.toml file
    edit_forc_toml(&out_dir, &project_name, &real_name)?;
    // Change the project name and authors of the Cargo.toml file
    edit_cargo_toml(&out_dir, &project_name, &real_name)?;

    Ok(())
}

pub(crate) fn init_from_local_template(project_name: String, local_path: &Url) -> Result<()> {
    // Get the path of the example we are using
    let path = std::env::current_dir()?;
    let out_dir = path.join(&project_name);
    let src_dir = local_path
        .to_file_path()
        .map_err(|()| anyhow!("unable to convert file path"))?;
    let real_name = whoami::realname();

    copy_folder(&src_dir, &out_dir)?;

    // Change the project name and author of the Forc.toml file
    edit_forc_toml(&out_dir, &project_name, &real_name)?;
    // Change the project name and authors of the Cargo.toml file
    edit_cargo_toml(&out_dir, &project_name, &real_name)?;

    Ok(())
}

fn parse_github_link(url: &Url) -> Result<GitPathInfo> {
    let mut path_segments = url.path_segments().context("cannot be base")?;

    let owner_name = path_segments
        .next()
        .context("Cannot parse owner name from github URL")?;

    let repo_name = path_segments
        .next()
        .context("Cannot repository name from github URL")?;

    let example_name = match path_segments
        .skip(2)
        .map(|s| s.to_string())
        .reduce(|cur: String, nxt: String| format!("{}/{}", cur, nxt))
    {
        Some(example_name) => example_name,
        None => "".to_string(),
    };
    Ok(GitPathInfo {
        owner: owner_name.to_string(),
        repo_name: repo_name.to_string(),
        example_name,
    })
}

fn edit_forc_toml(out_dir: &Path, project_name: &str, real_name: &str) -> Result<()> {
    let mut file = File::open(out_dir.join(constants::MANIFEST_FILE_NAME))?;
    let mut toml = String::new();
    file.read_to_string(&mut toml)?;
    let mut manifest_toml = toml.parse::<toml_edit::Document>()?;

    let mut authors = toml_edit::Array::default();
    let forc_toml: toml::Value = toml::de::from_str(&toml)?;
    if let Some(table) = forc_toml.as_table() {
        if let Some(package) = table.get("project") {
            // If authors Vec is currently popultated use that
            if let Some(toml::Value::Array(authors_vec)) = package.get("authors") {
                for author in authors_vec {
                    if let toml::value::Value::String(name) = &author {
                        authors.push(name);
                    }
                }
            } else {
                // Otherwise grab the current author from the author field
                // Lets remove the author field all together now that it has become deprecated
                if let Some(project) = manifest_toml.get_mut("project") {
                    if let Some(toml_edit::Item::Value(toml_edit::Value::String(name))) = project
                        .as_table_mut()
                        .context("Unable to get forc manifest as table")?
                        .remove("author")
                    {
                        authors.push(name.value());
                    }
                }
            }
        }
    }
    authors.push(real_name);

    manifest_toml["project"]["authors"] = toml_edit::value(authors);
    manifest_toml["project"]["name"] = toml_edit::value(project_name);

    let mut file = File::create(out_dir.join(constants::MANIFEST_FILE_NAME))?;
    file.write_all(manifest_toml.to_string().as_bytes())?;
    Ok(())
}

fn edit_cargo_toml(out_dir: &Path, project_name: &str, real_name: &str) -> Result<()> {
    let mut file = File::open(out_dir.join(constants::TEST_MANIFEST_FILE_NAME))?;
    let mut toml = String::new();
    file.read_to_string(&mut toml)?;

    let mut updated_authors = toml_edit::Array::default();

    let cargo_toml: toml::Value = toml::de::from_str(&toml)?;
    if let Some(table) = cargo_toml.as_table() {
        if let Some(package) = table.get("package") {
            if let Some(toml::Value::Array(authors_vec)) = package.get("authors") {
                for author in authors_vec {
                    if let toml::value::Value::String(name) = &author {
                        updated_authors.push(name);
                    }
                }
            }
        }
    }
    updated_authors.push(real_name);

    let mut manifest_toml = toml.parse::<toml_edit::Document>()?;
    manifest_toml["package"]["authors"] = toml_edit::value(updated_authors);
    manifest_toml["package"]["name"] = toml_edit::value(project_name);

    let mut file = File::create(out_dir.join(constants::TEST_MANIFEST_FILE_NAME))?;
    file.write_all(manifest_toml.to_string().as_bytes())?;
    Ok(())
}

fn download_file(url: &str, file_name: &str, out_dir: &Path) -> Result<PathBuf> {
    let mut data = Vec::new();
    let resp = ureq::get(url).call()?;
    resp.into_reader().read_to_end(&mut data)?;
    let path = out_dir.canonicalize()?.join(file_name);
    let mut file = File::create(&path)?;
    file.write_all(&data[..])?;
    Ok(path)
}

fn download_contents(url: &str, out_dir: &Path, responses: &[ContentResponse]) -> Result<()> {
    if !out_dir.exists() {
        fs::create_dir(out_dir)?;
    }

    // for all file_type == "file" responses, download the file and save it to the project directory.
    // for all file_type == "dir" responses, recursively call this function.
    for response in responses {
        match &response.file_type {
            FileType::File => {
                if let Some(url) = &response.download_url {
                    download_file(url, &response.name, out_dir)?;
                }
            }
            FileType::Dir => {
                match &response.name.as_str() {
                    // Only download the directory and its contents if it matches src or tests
                    &constants::SRC_DIR | &constants::TEST_DIRECTORY => {
                        let dir = out_dir.join(&response.name);
                        let url = format!("{}/{}", url, response.name);
                        let responses: Vec<ContentResponse> =
                            ureq::get(&url).call()?.into_json()?;
                        download_contents(&url, &dir, &responses)?;
                    }
                    _ => (),
                }
            }
        }
    }

    Ok(())
}

fn copy_folder(src_dir: &Path, out_dir: &Path) -> Result<()> {
    let mut stack = vec![PathBuf::from(src_dir)];

    let output_root = PathBuf::from(out_dir);
    let input_root = PathBuf::from(src_dir).components().count();

    while let Some(working_path) = stack.pop() {
        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };

        if fs::metadata(&dest).is_err() {
            fs::create_dir_all(&dest)?;
        }

        for entry in fs::read_dir(working_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                match path.file_name() {
                    Some(filename) => {
                        let dest_path = dest.join(filename);
                        fs::copy(&path, &dest_path)?;
                    }
                    None => {
                        println!("failed: {:?}", path);
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_github_link;
    use url::Url;

    #[test]
    fn test_github_link_parsing() {
        let example_url =
            Url::parse("https://github.com/FuelLabs/sway/tree/master/examples/hello_world")
                .unwrap();
        let git = parse_github_link(&example_url).unwrap();
        assert_eq!(git.owner, "FuelLabs");
        assert_eq!(git.repo_name, "sway");
        assert_eq!(git.example_name, "examples/hello_world");

        let example_url =
            Url::parse("https://github.com/FuelLabs/swayswap-demo/tree/master/contracts").unwrap();
        let git = parse_github_link(&example_url).unwrap();
        assert_eq!(git.owner, "FuelLabs");
        assert_eq!(git.repo_name, "swayswap-demo");
        assert_eq!(git.example_name, "contracts");
    }
}
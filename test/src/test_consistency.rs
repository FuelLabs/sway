//! This module contains checks that ensure consistency of the tests.

use anyhow::{anyhow, bail, Context, Ok, Result};
use std::path::{Path, PathBuf};
use toml::{Table, Value};

use crate::reduced_std_libs::REDUCED_STD_LIBS_DIR_NAME;

pub(crate) fn check() -> Result<()> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let all_tests_dir = PathBuf::from(format!("{manifest_dir}/src"));

    check_test_forc_tomls(&all_tests_dir)?;

    check_redundant_gitignore_files(&all_tests_dir)?;

    Ok(())
}

fn check_redundant_gitignore_files(all_tests_dir: &Path) -> Result<()> {
    let mut gitignores = vec![];
    find_gitignores(&PathBuf::from(all_tests_dir), &mut gitignores);

    return if gitignores.is_empty() {
        Ok(())
    } else {
        let mut gitignores = gitignores
            .iter()
            .map(|file| file.to_string_lossy().to_string())
            .collect::<Vec<_>>();
        gitignores.sort();

        Err(anyhow!("Redundant .gitignore files.\nTo fix the error, delete these redundant .gitignore files:\n{}", gitignores.join("\n")))
    };

    fn find_gitignores(path: &Path, gitignores: &mut Vec<PathBuf>) {
        const IN_LANGUAGE_TESTS_GITIGNORE: &str = "in_language_tests/.gitignore";

        if path.is_dir() {
            for entry in std::fs::read_dir(path).unwrap() {
                let entry = entry.unwrap().path();
                let entry_name = entry.to_str().unwrap();
                if entry_name.contains(REDUCED_STD_LIBS_DIR_NAME)
                    || entry_name.contains(IN_LANGUAGE_TESTS_GITIGNORE)
                {
                    continue;
                }
                find_gitignores(&entry, gitignores);
            }
        } else if path.is_file()
            && path
                .file_name()
                .map(|f| f.eq_ignore_ascii_case(".gitignore"))
                .unwrap_or(false)
        {
            gitignores.push(path.to_path_buf());
        }
    }
}

/// Checks that every Forc.toml file has the authors, license,
/// and the name property properly set and that the std library
/// is properly imported.
fn check_test_forc_tomls(all_tests_dir: &Path) -> Result<()> {
    let mut forc_tomls = vec![];
    find_test_forc_tomls(&PathBuf::from(all_tests_dir), &mut forc_tomls);

    for forc_toml_path in forc_tomls {
        let forc_toml_file_name = forc_toml_path.as_os_str().to_string_lossy();
        let content = std::fs::read_to_string(&forc_toml_path).unwrap();
        let toml = content.parse::<Table>().unwrap();

        // Skip over workspace configs. We want to test only project configs.
        if content.starts_with("[workspace]") {
            continue;
        }

        let project_name = forc_toml_path
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy();
        check_test_forc_toml(&content, &toml, &project_name)
            .context(format!("Invalid test Forc.toml: {forc_toml_file_name}"))?;
    }

    return Ok(());

    fn check_test_forc_toml(content: &str, toml: &Table, project_name: &str) -> Result<()> {
        let mut errors = vec![];

        if let Some(error) = check_project_authors_field(toml).err() {
            errors.push(error.to_string());
        }

        if let Some(error) = check_project_license_field(toml).err() {
            errors.push(error.to_string());
        }

        if let Some(error) = check_project_name_field(content, toml, project_name).err() {
            errors.push(error.to_string());
        }

        if let Some(error) = check_std_dependency(toml).err() {
            errors.push(error.to_string());
        }

        if !errors.is_empty() {
            bail!("{}", errors.join("\n"));
        }

        Ok(())
    }

    fn check_std_dependency(toml: &Table) -> Result<()> {
        return if let Some(implicit_std) = toml.get("project").and_then(|v| v.get("implicit-std")) {
            match implicit_std.as_bool() {
                Some(true) => Err(anyhow!("'project.implicit-std' cannot be set to `true` in tests. To import the standard library use, e.g., `std = {{ path = \"../<...>/sway-lib-std\" }}`.")),
                Some(false) => Ok(()),
                _ => Err(anyhow!("'project.implicit-std' value is invalid: `{}`. In tests 'project.implicit-std' must be set to `false`.", implicit_std)),
            }
        } else {
            // 'implicit-std' is not explicitly set.
            // Since the default value for 'implicit-std' is `true` we either need to
            // set it explicitly to `false`, or explicitly import local std library.
            let imported_std = imported_lib(toml, "std");

            if imported_std.is_none() {
                Err(anyhow!("`implicit-std` is `true` by default. Either explicitly set it to `false`, or import the standard library by using, e.g., `std = {{ path = \"../<...>/sway-lib-std\" }}`."))
            } else {
                // At least one of the libraries is imported.
                // Let's check that the local library is imported.
                if imported_std.is_some() {
                    check_local_import(imported_std.unwrap(), "std")?;
                }

                Ok(())
            }
        };

        fn imported_lib<'a>(toml: &'a Table, lib_name: &str) -> Option<&'a Value> {
            if let Some(import) = toml.get("dependencies").and_then(|v| v.get(lib_name)) {
                Some(import)
            // We don't have the straight import with the lib name but it can be
            // that we use some other name. In that case, the 'package' field still
            // must have the lib name. Let's try to find that one.
            } else if let Some(import) = toml
                .get("dependencies")
                .and_then(|v| v.as_table())
                .and_then(|t| {
                    t.values().find(|v| {
                        v.get("package")
                            .is_some_and(|p| p.as_str().unwrap_or_default() == lib_name)
                    })
                })
            {
                Some(import)
            } else {
                // We can have the library defined in the patch section.
                toml.get("patch")
                    .and_then(|patch| patch.get("https://github.com/fuellabs/sway"))
                    .and_then(|v| v.get(lib_name))
            }
        }

        fn check_local_import(lib: &Value, lib_name: &str) -> Result<()> {
            let is_local_import = lib
                .get("path")
                .map(|path| {
                    let path = path.as_str().unwrap_or_default();

                    path.ends_with(&format!("../../sway-lib-{lib_name}"))
                        || path
                            .contains(&format!("../../{REDUCED_STD_LIBS_DIR_NAME}/sway-lib-std-"))
                })
                .unwrap_or_default();

            if is_local_import {
                Ok(())
            } else {
                Err(anyhow!("'{lib_name}' library is not properly imported. It must be imported from the Sway repository by using a relative path, e.g., `{lib_name} = {{ path = \"../<...>/sway-lib-{lib_name}\" }}`."))
            }
        }
    }

    fn check_project_authors_field(toml: &Table) -> Result<()> {
        const AUTHOR: &str = "Fuel Labs <contact@fuel.sh>";

        if let Some(field) = toml.get("project").and_then(|v| v.get("authors")) {
            let err = |field: &Value| {
                Err(anyhow!("'project.authors' value is invalid: `{}`. 'project.authors' field is mandatory and must be set to `[\"{AUTHOR}\"]`.", field))
            };

            match field.as_array() {
                Some(value) if value.len() == 1 => match value[0].as_str() {
                    Some(value) if value == AUTHOR => Ok(()),
                    _ => err(field),
                },
                _ => err(field),
            }
        } else {
            Err(anyhow!("'project.authors' field not found. 'project.authors' field is mandatory and must be set to `[\"{AUTHOR}\"]`."))
        }
    }

    fn check_project_license_field(toml: &Table) -> Result<()> {
        check_mandatory_project_field(toml, "license", "Apache-2.0")
    }

    fn check_project_name_field(content: &str, toml: &Table, name: &str) -> Result<()> {
        // In some tests, e.g., when testing workspaces we will
        // want to have specific project names. In that case, mark
        // the `name` field with this comment to skip testing it.

        // Parsed TOML does not contain information about comments.
        // That's why we need the whole string `content` here.
        if content.contains("# NAME_NO_CHECK") {
            return Ok(());
        }

        check_mandatory_project_field(toml, "name", name)
    }

    fn check_mandatory_project_field(
        toml: &Table,
        field_name: &str,
        field_value: &str,
    ) -> Result<()> {
        if let Some(field) = toml.get("project").and_then(|v| v.get(field_name)) {
            match field.as_str() {
                Some(value) if value == field_value => Ok(()),
                _ => Err(anyhow!("'project.{field_name}' value is invalid: `{}`. 'project.{field_name}' field is mandatory and must be set to `\"{field_value}\"`.", field)),
            }
        } else {
            Err(anyhow!("'project.{field_name}' field not found. 'project.{field_name}' field is mandatory and must be set to `\"{field_value}\"`."))
        }
    }

    fn find_test_forc_tomls(path: &Path, forc_tomls: &mut Vec<PathBuf>) {
        if path.is_dir() {
            for entry in std::fs::read_dir(path).unwrap() {
                let entry = entry.unwrap();
                if entry
                    .path()
                    .to_str()
                    .unwrap()
                    .contains(REDUCED_STD_LIBS_DIR_NAME)
                {
                    continue;
                }
                find_test_forc_tomls(&entry.path(), forc_tomls);
            }
        } else if path.is_file()
            && path
                .file_name()
                .map(|f| f.eq_ignore_ascii_case("forc.toml"))
                .unwrap_or(false)
        {
            forc_tomls.push(path.to_path_buf());
        }
    }
}

pub(crate) mod error;

use error::MDPreProcessError;
use regex::Regex;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug)]
struct MarkdownFile {
    path: PathBuf,
    includes: HashSet<PathBuf>,
}

impl MarkdownFile {
    fn parse<P: AsRef<Path>>(path: P) -> Result<Self, MDPreProcessError> {
        let path = path
            .as_ref()
            .canonicalize()
            .map_err(|_| MDPreProcessError::Canonicalize(path.as_ref().to_path_buf()))?;
        let content = fs::read_to_string(&path)?;
        let dir = path.parent().unwrap_or(Path::new("."));
        let re = Regex::new(r"\{\{#include\s+([^\}]+)\}\}")?;

        let includes = re
            .captures_iter(&content)
            .filter_map(|caps| {
                let inc_rel = caps[1].trim();
                let inc_path = dir.join(inc_rel);
                inc_path.canonicalize().ok()
            })
            .collect();

        Ok(MarkdownFile { path, includes })
    }
}

#[derive(Debug, Default)]
struct MarkdownDepGraph {
    graph: HashMap<PathBuf, HashSet<PathBuf>>,
}

impl MarkdownDepGraph {
    fn build(entry: &Path) -> Result<Self, MDPreProcessError> {
        let mut graph = HashMap::new();
        let mut visited = HashSet::new();
        Self::build_recursive(entry, &mut graph, &mut visited)?;
        Ok(MarkdownDepGraph { graph })
    }

    fn build_recursive(
        path: &Path,
        graph: &mut HashMap<PathBuf, HashSet<PathBuf>>,
        visited: &mut HashSet<PathBuf>,
    ) -> Result<(), MDPreProcessError> {
        let file = MarkdownFile::parse(path)?;
        if visited.insert(file.path.clone()) {
            for dep in &file.includes {
                Self::build_recursive(dep, graph, visited)?;
            }
            graph.insert(file.path.clone(), file.includes);
        }
        Ok(())
    }

    fn topological_sort(&self) -> Result<Vec<PathBuf>, MDPreProcessError> {
        let mut in_degree = HashMap::new();
        for (node, deps) in &self.graph {
            in_degree.entry(node.clone()).or_insert(0);
            for dep in deps {
                *in_degree.entry(dep.clone()).or_insert(0) += 1;
            }
        }

        let mut queue: VecDeque<_> = in_degree
            .iter()
            .filter(|&(_, &deg)| deg == 0)
            .map(|(n, _)| n.clone())
            .collect();

        let mut sorted = Vec::new();
        let mut processed = 0;

        while let Some(node) = queue.pop_front() {
            sorted.push(node.clone());
            processed += 1;
            if let Some(deps) = self.graph.get(&node) {
                for dep in deps {
                    let deg = in_degree.get_mut(dep).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(dep.clone());
                    }
                }
            }
        }

        if processed != in_degree.len() {
            return Err(MDPreProcessError::Cycle);
        }
        Ok(sorted)
    }
}

#[derive(Debug)]
struct MarkdownFlattener {
    file_contents: HashMap<PathBuf, String>,
}

impl MarkdownFlattener {
    fn flatten_files(order: &[PathBuf]) -> Result<Self, MDPreProcessError> {
        let mut file_contents = HashMap::new();
        let re = Regex::new(r"\{\{#include\s+([^\}]+)\}\}")?;

        // Process leaves first (reverse topological order)
        for file in order.iter().rev() {
            let content = fs::read_to_string(file)?;
            let expanded = Self::expand_includes(&content, file, &file_contents, &re)?;
            file_contents.insert(file.clone(), expanded);
        }

        Ok(MarkdownFlattener { file_contents })
    }

    fn expand_includes(
        content: &str,
        file: &Path,
        file_contents: &HashMap<PathBuf, String>,
        re: &Regex,
    ) -> Result<String, MDPreProcessError> {
        let dir = file.parent().unwrap_or(Path::new("."));
        let mut result = String::new();
        let mut last_end = 0;

        for caps in re.captures_iter(content) {
            let match_range = caps.get(0).unwrap();

            // Add content before this match
            result.push_str(&content[last_end..match_range.start()]);

            // Process the include
            let inc_rel = caps[1].trim();
            let inc_path = dir.join(inc_rel);

            match inc_path.canonicalize() {
                Ok(canonical_path) => match file_contents.get(&canonical_path) {
                    Some(included_content) => {
                        result.push_str(included_content);
                    }
                    None => {
                        return Err(MDPreProcessError::MissingInclude(canonical_path));
                    }
                },
                Err(_) => {
                    return Err(MDPreProcessError::Canonicalize(inc_path));
                }
            }

            last_end = match_range.end();
        }

        // Add remaining content after last match
        result.push_str(&content[last_end..]);
        Ok(result)
    }

    fn get_file(&self, entry: &Path) -> Option<&str> {
        self.file_contents
            .get(&entry.canonicalize().ok()?)
            .map(|s| s.as_str())
    }
}

pub fn flatten_markdown(entry: &Path) -> Result<String, MDPreProcessError> {
    let dep_graph = MarkdownDepGraph::build(entry)?;
    let order = dep_graph.topological_sort()?;
    let flattener = MarkdownFlattener::flatten_files(&order)?;
    flattener
        .get_file(entry)
        .map(|s| s.to_string())
        .ok_or_else(|| MDPreProcessError::Other("Could not flatten entry file".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    fn write_file<P: AsRef<Path>>(path: P, content: &str) -> Result<(), MDPreProcessError> {
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    #[test]
    fn test_flatten_single_file_no_includes() -> Result<(), MDPreProcessError> {
        let dir = tempdir()?;
        let readme = dir.path().join("README.md");
        write_file(&readme, "# Title\n\nHello world!")?;

        let result = flatten_markdown(&readme)?;
        assert_eq!(result, "# Title\n\nHello world!");
        Ok(())
    }

    #[test]
    fn test_flatten_simple_include() -> Result<(), MDPreProcessError> {
        let dir = tempdir()?;
        let readme = dir.path().join("README.md");
        let inc = dir.path().join("inc.md");
        write_file(&inc, "This is included.")?;
        write_file(&readme, "# Main\n\n{{#include inc.md}}\n\nEnd.")?;

        let result = flatten_markdown(&readme)?;
        assert_eq!(result, "# Main\n\nThis is included.\n\nEnd.");
        Ok(())
    }

    #[test]
    fn test_flatten_nested_includes() -> Result<(), MDPreProcessError> {
        let dir = tempdir()?;
        let readme = dir.path().join("README.md");
        let sub = dir.path().join("sub.md");
        let subsub = dir.path().join("deep.md");
        write_file(&subsub, "Deep content.")?;
        write_file(&sub, "Subhead\n\n{{#include deep.md}}")?;
        write_file(&readme, "# Root\n\n{{#include sub.md}}\n\nEnd.")?;

        let result = flatten_markdown(&readme)?;
        assert_eq!(result, "# Root\n\nSubhead\n\nDeep content.\n\nEnd.");
        Ok(())
    }

    #[test]
    fn test_flatten_multiple_includes() -> Result<(), MDPreProcessError> {
        let dir = tempdir()?;
        let readme = dir.path().join("README.md");
        let a = dir.path().join("a.md");
        let b = dir.path().join("b.md");
        write_file(&a, "Alpha!")?;
        write_file(&b, "Bravo!")?;
        write_file(
            &readme,
            "# Combo\n\n{{#include a.md}}\n\n{{#include b.md}}\nDone.",
        )?;

        let result = flatten_markdown(&readme)?;
        assert_eq!(result, "# Combo\n\nAlpha!\n\nBravo!\nDone.");
        Ok(())
    }

    #[test]
    fn test_flatten_missing_include() -> Result<(), MDPreProcessError> {
        let dir = tempdir()?;
        let readme = dir.path().join("README.md");
        write_file(&readme, "# Main\n\n{{#include missing.md}}\nEnd.")?;

        let result = flatten_markdown(&readme);
        assert!(matches!(
            result,
            Err(MDPreProcessError::Canonicalize(_)) | Err(MDPreProcessError::MissingInclude(_))
        ));
        Ok(())
    }

    #[test]
    fn test_cycle_detection() -> Result<(), MDPreProcessError> {
        let dir = tempdir()?;
        let a = dir.path().join("a.md");
        let b = dir.path().join("b.md");
        write_file(&a, "A here\n{{#include b.md}}")?;
        write_file(&b, "B here\n{{#include a.md}}")?;

        let result = flatten_markdown(&a);
        assert!(matches!(result, Err(MDPreProcessError::Cycle)));
        Ok(())
    }
}

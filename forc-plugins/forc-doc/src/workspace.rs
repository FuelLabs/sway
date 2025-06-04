use anyhow::{Context, Result};
use forc_pkg::{manifest::ManifestFile, PackageManifestFile};
use std::path::{Path, PathBuf};

/// Represents the context in which forc-doc is being run
#[derive(Debug, Clone)]
pub enum DocContext {
    /// Running on a standalone package
    Package(PathBuf),
    /// Running on a workspace root
    Workspace {
        root: PathBuf,
        members: Vec<WorkspaceMember>,
    },
}

#[derive(Debug, Clone)]
pub struct WorkspaceMember {
    pub name: String,
    pub path: PathBuf,
    pub manifest: PackageManifestFile,
}

impl DocContext {
    /// Detect the documentation context from the current directory
    pub fn detect(path: &Path) -> Result<Self> {
        let manifest_path = path.join("Forc.toml");
        
        if !manifest_path.exists() {
            // Try to find workspace root by walking up the directory tree
            return Self::find_workspace_root(path);
        }

        let manifest = ManifestFile::from_file(&manifest_path)
            .with_context(|| format!("Failed to read manifest at {}", manifest_path.display()))?;

        match manifest {
            ManifestFile::Workspace(workspace_manifest) => {
                // This is a workspace root
                let members = Self::collect_workspace_members(path, &workspace_manifest.workspace.members)?;
                Ok(DocContext::Workspace {
                    root: path.to_path_buf(),
                    members,
                })
            }
            ManifestFile::Package(package_manifest) => {
                // Check if this package is part of a workspace
                if let Ok(workspace_context) = Self::find_workspace_root(path.parent().unwrap_or(path)) {
                    return Ok(workspace_context);
                }
                
                // This is a standalone package
                Ok(DocContext::Package(path.to_path_buf()))
            }
        }
    }

    /// Find workspace root by walking up the directory tree
    fn find_workspace_root(start_path: &Path) -> Result<Self> {
        let mut current = start_path;
        
        loop {
            let manifest_path = current.join("Forc.toml");
            
            if manifest_path.exists() {
                let manifest = ManifestFile::from_file(&manifest_path)?;
                
                if let ManifestFile::Workspace(workspace_manifest) = manifest {
                    let members = Self::collect_workspace_members(current, &workspace_manifest.workspace.members)?;
                    return Ok(DocContext::Workspace {
                        root: current.to_path_buf(),
                        members,
                    });
                }
            }
            
            match current.parent() {
                Some(parent) => current = parent,
                None => break,
            }
        }
        
        // Default to treating as standalone package
        Ok(DocContext::Package(start_path.to_path_buf()))
    }

    /// Collect all workspace members from the workspace manifest
    fn collect_workspace_members(
        workspace_root: &Path,
        member_paths: &[String],
    ) -> Result<Vec<WorkspaceMember>> {
        let mut members = Vec::new();
        
        for member_path in member_paths {
            let full_path = workspace_root.join(member_path);
            let manifest_path = full_path.join("Forc.toml");
            
            if !manifest_path.exists() {
                eprintln!("Warning: Workspace member {} does not have a Forc.toml", member_path);
                continue;
            }
            
            let manifest = ManifestFile::from_file(&manifest_path)?;
            
            if let ManifestFile::Package(package_manifest) = manifest {
                members.push(WorkspaceMember {
                    name: package_manifest.project.name.clone(),
                    path: full_path,
                    manifest: package_manifest,
                });
            } else {
                eprintln!("Warning: Workspace member {} is not a package", member_path);
            }
        }
        
        Ok(members)
    }
}
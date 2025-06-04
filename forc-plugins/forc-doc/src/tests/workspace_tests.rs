use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_workspace_detection() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path();
    
    // Create workspace Forc.toml
    let workspace_manifest = r#"[workspace]
members = ["member1", "member2"]
"#;
    fs::write(workspace_root.join("Forc.toml"), workspace_manifest).unwrap();
    
    // Create member packages
    for member in ["member1", "member2"] {
        let member_dir = workspace_root.join(member);
        fs::create_dir_all(&member_dir).unwrap();
        
        let package_manifest = format!(r#"[project]
name = "{}"
"#, member);
        fs::write(member_dir.join("Forc.toml"), package_manifest).unwrap();
    }
    
    // Test workspace detection
    let context = forc_doc::workspace::DocContext::detect(workspace_root).unwrap();
    
    match context {
        forc_doc::workspace::DocContext::Workspace { members, .. } => {
            assert_eq!(members.len(), 2);
            assert!(members.iter().any(|m| m.name == "member1"));
            assert!(members.iter().any(|m| m.name == "member2"));
        }
        _ => panic!("Expected workspace context"),
    }
}
use anyhow::{Context, Result};
use slug::slugify;
use std::env;
use std::path::Path;
use std::process::Command;

pub mod config;

pub fn slugify_title(title: &str) -> String {
    slugify(title)
}

pub fn spawn_editor(path: &Path) -> Result<()> {
    let editor = env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| "vim".to_string());

    let status = Command::new(editor)
        .arg(path)
        .status()
        .context("Failed to spawn editor")?;

    if !status.success() {
        anyhow::bail!("Editor exited with non-zero status");
    }

    Ok(())
}

pub fn move_file(old_path: &Path, new_path: &Path) -> Result<()> {
    std::fs::rename(old_path, new_path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_slugify_title() {
        assert_eq!(slugify_title("Hello World"), "hello-world");
        assert_eq!(
            slugify_title("Hello! World? (Part 1)"),
            "hello-world-part-1"
        );
        assert_eq!(slugify_title("Rust & Go"), "rust-go");
    }

    #[test]
    fn test_move_file() {
        let dir = tempdir().unwrap();
        let old_path = dir.path().join("old.txt");
        let new_path = dir.path().join("new.txt");

        fs::write(&old_path, "test").unwrap();
        move_file(&old_path, &new_path).unwrap();

        assert!(!old_path.exists());
        assert!(new_path.exists());
        assert_eq!(fs::read_to_string(&new_path).unwrap(), "test");
    }
}

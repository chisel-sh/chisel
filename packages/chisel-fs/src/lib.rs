use anyhow::{Context, Result};
use slug::slugify;
use std::env;
use std::path::Path;
use std::process::Command;

pub mod config;

pub fn slugify_title(title: &str) -> String {
    slugify(title)
}

/// Truncate to at most `max` characters, appending "..." when truncated.
/// Counts chars and cuts on char boundaries — byte-index slicing panics on
/// multibyte characters like em dashes.
pub fn truncate_with_ellipsis(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let truncated: String = s.chars().take(max.saturating_sub(3)).collect();
    format!("{}...", truncated)
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
    fn test_truncate_with_ellipsis() {
        // Under the limit: unchanged
        assert_eq!(truncate_with_ellipsis("short", 10), "short");
        assert_eq!(truncate_with_ellipsis("exactly-10", 10), "exactly-10");

        // Over the limit: truncated to max chars including the ellipsis
        assert_eq!(truncate_with_ellipsis("abcdefghijk", 10), "abcdefg...");

        // Multibyte chars must not panic and count as single chars
        let em = format!("{}\u{2014}\u{2014}\u{2014}", "a".repeat(23)); // 26 chars, 32 bytes
        assert_eq!(truncate_with_ellipsis(&em, 26), em);
        assert_eq!(
            truncate_with_ellipsis(&em, 25),
            format!("{}...", "a".repeat(22))
        );

        // Em dash straddling the old byte cutoff must survive intact
        let straddle = format!("{}\u{2014}tail", "x".repeat(24));
        let out = truncate_with_ellipsis(&straddle, 26);
        assert_eq!(out, format!("{}...", "x".repeat(23)));
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

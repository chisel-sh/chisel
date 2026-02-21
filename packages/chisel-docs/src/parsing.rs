use crate::{DocFrontmatter, DocSection};
use serde_yaml;

pub fn parse_frontmatter(raw_content: &str) -> (Option<DocFrontmatter>, String) {
    if raw_content.starts_with("---") {
        let parts: Vec<&str> = raw_content.splitn(3, "---").collect();
        if parts.len() == 3 && parts[0].is_empty() {
            let fm_str = parts[1];
            let content = parts[2].to_string();
            let fm: Option<DocFrontmatter> = serde_yaml::from_str(fm_str).ok();
            return (fm, content.trim_start().to_string());
        }
    }
    (None, raw_content.to_string())
}

pub fn parse_sections(content: &str) -> Vec<DocSection> {
    let mut sections = Vec::new();
    for line in content.lines() {
        if line.starts_with('#') {
            let level = line.chars().take_while(|&c| c == '#').count() as u8;
            if level > 0 {
                let title = line.trim_start_matches('#').trim().to_string();
                sections.push(DocSection { level, title });
            }
        }
    }
    sections
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let content = "---
title: Test
created_at: 2024-01-01T00:00:00Z
order: 5
---
# Hello
World";
        let (fm, body) = parse_frontmatter(content);
        if fm.is_none() {
            // Debugging output if it still fails
            println!("Failed to parse frontmatter");
        }
        assert!(fm.is_some());
        let fm = fm.unwrap();
        assert_eq!(fm.title, "Test");
        assert_eq!(fm.order, Some(5));
        assert_eq!(body, "# Hello\nWorld");
    }

    #[test]
    fn test_parse_sections() {
        let content = "# Title
## Subtitle
### Third
Some text
## Another";
        let sections = parse_sections(content);
        assert_eq!(sections.len(), 4);
        assert_eq!(sections[0].level, 1);
        assert_eq!(sections[0].title, "Title");
        assert_eq!(sections[1].level, 2);
        assert_eq!(sections[3].level, 2);
    }
}

use std::path::Path;

use serde::Deserialize;

use crate::domain::{slugify, Skill, SkillSource, OTHER_GROUP_ID};

#[derive(Debug, Deserialize, Default)]
struct FrontMatter {
    name: Option<String>,
    description: Option<String>,
}

/// Extracts and parses the `---\n...\n---` YAML front matter block from a
/// SKILL.md file's contents. Returns `None` if the file doesn't start with a
/// front matter block at all (tolerated — such a file is simply skipped by
/// the caller rather than treated as an error, since local discovery must
/// never fail the whole app over one malformed file).
fn extract_front_matter(content: &str) -> Option<FrontMatter> {
    let mut lines = content.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    let mut yaml_lines = Vec::new();
    for line in lines {
        if line.trim() == "---" {
            let yaml = yaml_lines.join("\n");
            return serde_norway::from_str::<FrontMatter>(&yaml).ok();
        }
        yaml_lines.push(line);
    }
    None
}

/// Walks `<project_root>/.agents/skills/*/SKILL.md`, parsing front matter for
/// `name`/`description`. Never preselected, always marked `local`/`installed`
/// (a discovered skill is, by definition, already on disk) — group
/// assignment and preselection can be overridden by a matching config entry,
/// which `app::skills_service` applies on top of this base list.
pub fn discover_local_skills(project_root: &Path) -> Vec<Skill> {
    discover_skills_in_directory(&project_root.join(".agents").join("skills"))
}

/// Scans a user-configured source catalog. The directory contains Skill
/// folders; it is never interpreted as an installation destination.
pub fn discover_skills_in_directory(skills_dir: &Path) -> Vec<Skill> {
    let mut results = Vec::new();

    let Ok(entries) = std::fs::read_dir(skills_dir) else {
        return results;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let skill_md_path = path.join("SKILL.md");
        let Ok(content) = std::fs::read_to_string(&skill_md_path) else {
            continue;
        };
        let dir_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("skill")
            .to_string();

        let front_matter = extract_front_matter(&content).unwrap_or_default();
        let display_name = front_matter
            .name
            .clone()
            .unwrap_or_else(|| dir_name.clone());
        let description = front_matter.description.unwrap_or_default();
        let id = slugify(&dir_name);

        results.push(Skill {
            id,
            name: display_name.clone(),
            display_name,
            description,
            source: SkillSource::Local {
                path: skill_md_path.to_string_lossy().to_string(),
            },
            repository: String::new(),
            repository_url: String::new(),
            skill_name: dir_name,
            skills_url: String::new(),
            group_id: OTHER_GROUP_ID.to_string(),
            tags: Vec::new(),
            preselected: false,
            local: true,
            installed: true,
            enabled: true,
        });
    }

    results.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_skill(root: &Path, dir: &str, front_matter: &str) {
        let skill_dir = root.join(".agents").join("skills").join(dir);
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), front_matter).unwrap();
    }

    #[test]
    fn discovers_skills_with_front_matter() {
        let tmp = tempfile::tempdir().unwrap();
        write_skill(
            tmp.path(),
            "tauri-v2",
            "---\nname: tauri-v2\ndescription: \"Tauri v2 skill\"\nversion: 1.0.1\n---\n\n# Body\n",
        );

        let skills = discover_local_skills(tmp.path());
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].id, "tauri-v2");
        assert_eq!(skills[0].display_name, "tauri-v2");
        assert_eq!(skills[0].description, "Tauri v2 skill");
        assert!(skills[0].local);
        assert!(skills[0].installed);
        assert!(!skills[0].preselected);
        assert_eq!(skills[0].group_id, "other");
    }

    #[test]
    fn falls_back_to_directory_name_without_front_matter() {
        let tmp = tempfile::tempdir().unwrap();
        write_skill(tmp.path(), "no-front-matter", "# Just a heading\n");

        let skills = discover_local_skills(tmp.path());
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].display_name, "no-front-matter");
    }

    #[test]
    fn returns_empty_when_no_agents_dir() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(discover_local_skills(tmp.path()).is_empty());
    }

    #[test]
    fn ignores_files_that_are_not_directories() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join(".agents").join("skills")).unwrap();
        fs::write(
            tmp.path().join(".agents").join("skills").join("stray.txt"),
            "x",
        )
        .unwrap();
        assert!(discover_local_skills(tmp.path()).is_empty());
    }
}

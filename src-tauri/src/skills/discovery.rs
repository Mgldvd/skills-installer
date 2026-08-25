use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::domain::{slugify, Skill, SkillSource, OTHER_GROUP_ID};

#[derive(Debug, Deserialize, Default)]
pub struct FrontMatter {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Extracts and parses the `---\n...\n---` YAML front matter block from a
/// SKILL.md file's contents. Returns `None` if the file doesn't start with a
/// front matter block at all (tolerated — such a file is simply skipped by
/// the caller rather than treated as an error, since local discovery must
/// never fail the whole app over one malformed file).
pub fn extract_front_matter(content: &str) -> Option<FrontMatter> {
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

/// Project-relative install destinations that are distinguishable on disk.
/// Two different sources feed this map, and they answer two different
/// questions:
///   1. The `skills` CLI's own Supported Agents table (vercel-labs/skills
///      README) — the authoritative *install destination* for each agent,
///      since that CLI is what actually writes the files. This is what
///      `SUPPORTED_AGENTS` in the frontend's `projectPath` mirrors, and it's
///      the primary source for the dedicated per-agent entries below.
///   2. Each agent's own official product documentation about what it
///      *additionally reads* for cross-tool compatibility, beyond wherever
///      the CLI happened to write it — never by attribution/guesswork.
///
/// A skill placed in one of these directories is genuinely readable by
/// every listed agent, not just whichever one the install request happened
/// to name, so it's reported installed for every one of them:
///   - `.agents/skills`: the cross-client "AGENTS.md skills" convention.
///     It's the CLI's own install destination for `universal`, `codex`,
///     `gemini-cli`, `cursor`, `opencode`, `github-copilot`, `vscode`, and
///     `zed` (none of these have a dedicated directory of their own for
///     this scope). `pi` and `openclaw` *do* have their own dedicated
///     project directories (below) but also document reading this one.
///   - `.claude/skills`: Claude Code's own directory, but also read for
///     compatibility by `cursor`, `opencode`, `github-copilot`, and
///     `vscode` (each documents this explicitly), in addition to
///     `claude-code` itself.
///   - `.windsurf/skills`: exclusive to `windsurf` — no other agent's docs
///     mention reading it.
///   - `.pi/skills`: `pi`'s own dedicated CLI install destination, in
///     addition to the shared `.agents/skills` above.
///   - `.github/skills`: exclusive to `vscode` — VS Code's own dedicated
///     directory for GitHub Copilot Chat/agent mode, in addition to the
///     shared `.agents/skills` and `.claude/skills` above. The generic
///     `github-copilot` agent (Copilot CLI/other surfaces) is a distinct
///     row from `vscode` in this app, even though both install through the
///     same underlying `skills` CLI agent — see `domain::cli_agent_id`.
///   - `skills` (bare, project root): `openclaw`'s own dedicated CLI
///     install destination, in addition to the shared `.agents/skills`
///     above. A generic name, but `discover_skills_in_directory` only
///     counts genuine `SKILL.md`-bearing subdirectories, so an unrelated
///     folder that happens to be named `skills/` won't false-positive
///     unless it coincidentally contains one too.
///   - `.openhands/skills`: `openhands`'s own dedicated CLI install
///     destination, in addition to the shared `.agents/skills` above.
///   - `.hermes/skills`: `hermes-agent`'s own dedicated CLI install
///     destination. Not added to the shared `.agents/skills` list — per
///     Hermes' own docs, reading that directory is a manual/opt-in extra,
///     not automatic.
///
/// A directory can therefore report multiple agent ids even when it isn't
/// `.agents/skills`, and an agent can appear in more than one directory's
/// list — `discover_installed_agents` dedupes per skill before returning.
const AGENT_PROJECT_DIRS: &[(&[&str], &[&str])] = &[
    (
        &[".agents", "skills"],
        &[
            "universal",
            "codex",
            "gemini-cli",
            "cursor",
            "opencode",
            "github-copilot",
            "pi",
            "vscode",
            "openclaw",
            "zed",
            "openhands",
        ],
    ),
    (
        &[".claude", "skills"],
        &[
            "claude-code",
            "cursor",
            "opencode",
            "github-copilot",
            "vscode",
        ],
    ),
    (&[".windsurf", "skills"], &["windsurf"]),
    (&[".pi", "skills"], &["pi"]),
    (&[".github", "skills"], &["vscode"]),
    (&["skills"], &["openclaw"]),
    (&[".openhands", "skills"], &["openhands"]),
    (&[".hermes", "skills"], &["hermes-agent"]),
];

/// Maps each installed skill's directory name (`skill_name`) to the ids of
/// the agents whose install destination under `project_root` contains it.
pub fn discover_installed_agents(project_root: &Path) -> HashMap<String, Vec<String>> {
    let mut agents_by_skill: HashMap<String, Vec<String>> = HashMap::new();
    for (segments, agent_ids) in AGENT_PROJECT_DIRS {
        let dir = segments
            .iter()
            .fold(project_root.to_path_buf(), |acc, part| acc.join(part));
        for skill in discover_skills_in_directory(&dir) {
            let entry = agents_by_skill.entry(skill.skill_name).or_default();
            for agent_id in *agent_ids {
                entry.push(agent_id.to_string());
            }
        }
    }
    dedupe_agent_lists(&mut agents_by_skill);
    agents_by_skill
}

/// Per-agent *global* (`$HOME`-relative) install destination — the Global
/// scope counterpart to `AGENT_PROJECT_DIRS`, mirroring the frontend's
/// `SUPPORTED_AGENTS[].globalPath` (same authoritative source: the real
/// `skills` CLI's own Supported Agents table). Kept in sync by hand — there
/// is no shared source of truth across the Rust/TS boundary. Unlike project
/// paths, these are almost all agent-exclusive; only `github-copilot` and
/// `vscode` share one (`~/.copilot/skills`), since VS Code's Copilot Chat
/// and other Copilot surfaces are account-scoped to the same directory —
/// see `domain::cli_agent_id`.
const AGENT_GLOBAL_DIRS: &[(&str, &[&str])] = &[
    (".config/agents/skills", &["universal"]),
    (".claude/skills", &["claude-code"]),
    (".codex/skills", &["codex"]),
    (".gemini/skills", &["gemini-cli"]),
    (".cursor/skills", &["cursor"]),
    (".codeium/windsurf/skills", &["windsurf"]),
    (".config/opencode/skills", &["opencode"]),
    (".copilot/skills", &["github-copilot", "vscode"]),
    (".pi/agent/skills", &["pi"]),
    (".openclaw/skills", &["openclaw"]),
    (".agents/skills", &["zed"]),
    (".openhands/skills", &["openhands"]),
    (".hermes/skills", &["hermes-agent"]),
];

/// The Global-scope counterpart to `discover_installed_agents`: maps each
/// installed skill's directory name to the ids of the agents whose *global*
/// destination under `home` contains it. Takes the home directory as a
/// parameter rather than reading `$HOME` itself, both to stay pure/testable
/// and so callers can resolve (and fail gracefully on) a missing `$HOME`
/// however fits their own context — see `SkillsService::load_state_for`.
pub fn discover_installed_agents_globally(home: &Path) -> HashMap<String, Vec<String>> {
    let mut agents_by_skill: HashMap<String, Vec<String>> = HashMap::new();
    for (relative, agent_ids) in AGENT_GLOBAL_DIRS {
        for skill in discover_skills_in_directory(&home.join(relative)) {
            let entry = agents_by_skill.entry(skill.skill_name).or_default();
            for agent_id in *agent_ids {
                entry.push(agent_id.to_string());
            }
        }
    }
    dedupe_agent_lists(&mut agents_by_skill);
    agents_by_skill
}

// An agent can be reachable through more than one destination (e.g. Cursor
// reads both `.agents/skills` and `.claude/skills`), so dedupe — and sort
// for a deterministic, UI-friendly order.
fn dedupe_agent_lists(agents_by_skill: &mut HashMap<String, Vec<String>>) {
    for agents in agents_by_skill.values_mut() {
        agents.sort();
        agents.dedup();
    }
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
            installed_agents: Vec::new(),
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

    #[test]
    fn discover_installed_agents_globally_finds_a_skill_in_an_agent_exclusive_global_dir() {
        let home = tempfile::tempdir().unwrap();
        write_skill_at(&home.path().join(".codex").join("skills"), "triage");

        let agents = discover_installed_agents_globally(home.path());
        assert_eq!(
            agents.get("triage").cloned().unwrap_or_default(),
            vec!["codex"]
        );
    }

    #[test]
    fn discover_installed_agents_globally_reports_both_agents_sharing_the_copilot_dir() {
        let home = tempfile::tempdir().unwrap();
        write_skill_at(&home.path().join(".copilot").join("skills"), "triage");

        let mut agents = discover_installed_agents_globally(home.path())
            .get("triage")
            .cloned()
            .unwrap_or_default();
        agents.sort();
        assert_eq!(agents, vec!["github-copilot", "vscode"]);
    }

    #[test]
    fn discover_installed_agents_globally_is_empty_for_a_home_with_nothing_installed() {
        let home = tempfile::tempdir().unwrap();
        assert!(discover_installed_agents_globally(home.path()).is_empty());
    }

    fn write_skill_at(skills_dir: &Path, dir: &str) {
        let skill_dir = skills_dir.join(dir);
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            format!("---\nname: {dir}\ndescription: fixture\n---\n"),
        )
        .unwrap();
    }
}

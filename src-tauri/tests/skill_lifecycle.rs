//! Exercises the app's core purpose end to end — installing and managing
//! Skills for a given project folder — against a real, inspectable fixture
//! directory (`<repo root>/.generated/test-skills`) instead of an ephemeral
//! tempdir, so a failure can be diagnosed by looking at what's actually left
//! on disk. `.generated/` is gitignored; each test claims its own
//! subdirectory under it and wipes that subdirectory before running, so
//! tests never interfere with each other or with leftovers from a previous
//! run.

use std::fs;
use std::path::{Path, PathBuf};

use skills_installer_lib::app::SkillsService;
use skills_installer_lib::config::ConfigurationService;
use skills_installer_lib::domain::UiPreferences;
use skills_installer_lib::preferences::PreferencesService;
use skills_installer_lib::skills::{discover_installed_agents, discover_local_skills};

/// A fresh, isolated fixture directory under
/// `<repo root>/.generated/test-skills/<case>`.
fn fixture_root(case: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri has a parent directory (the repo root)")
        .join(".generated")
        .join("test-skills")
        .join(case);
    if root.exists() {
        fs::remove_dir_all(&root).expect("clear previous test-skills fixture");
    }
    fs::create_dir_all(&root).expect("create test-skills fixture directory");
    root
}

/// Writes a minimal installed `SKILL.md` under `<project_root>/<dest_segments>/<skill_dir>`
/// — the same shape a real install (or the discovery unit tests) produces.
fn write_skill(project_root: &Path, dest_segments: &[&str], skill_dir: &str, name: &str) {
    let dir = dest_segments
        .iter()
        .fold(project_root.to_path_buf(), |acc, part| acc.join(part))
        .join(skill_dir);
    fs::create_dir_all(&dir).expect("create skill directory");
    fs::write(
        dir.join("SKILL.md"),
        format!("---\nname: {name}\ndescription: fixture skill\n---\n\nBody.\n"),
    )
    .expect("write SKILL.md");
}

fn remove_skill(project_root: &Path, dest_segments: &[&str], skill_dir: &str) {
    let dir = dest_segments
        .iter()
        .fold(project_root.to_path_buf(), |acc, part| acc.join(part))
        .join(skill_dir);
    fs::remove_dir_all(&dir).expect("remove skill directory");
}

/// A `SkillsService` wired to `root`: `skills.yaml` and `preferences.json`
/// live directly under it, and its "local Skills catalog" source is
/// `root/.agents/skills` — the same directory a real project-scope install
/// into the shared convention path writes to.
fn service_for(root: &Path, config_yaml: &str) -> SkillsService {
    fs::write(root.join("skills.yaml"), config_yaml).expect("write skills.yaml");
    let config_service = ConfigurationService::new(None, root.to_path_buf());
    let preferences = PreferencesService::with_path(root.join("preferences.json"));
    preferences
        .save(&UiPreferences {
            local_source_path: Some(root.join(".agents").join("skills").display().to_string()),
            ..Default::default()
        })
        .expect("save preferences");
    // A distinct, always-empty subdirectory of the fixture — never `root`
    // itself, so Global-scope discovery never overlaps the project-scope
    // fixtures these tests write directly under `root`.
    SkillsService::new(config_service, root.to_path_buf(), preferences)
        .with_home_override(root.join("home"))
}

const EMPTY_CONFIG: &str = r#"
version: 1
groups: []
skills: []
"#;

// --- Installing marks a Skill installed, per agent -------------------------

#[test]
fn installing_into_the_shared_convention_dir_marks_it_installed_for_every_agent_that_reads_it() {
    let root = fixture_root("install-shared-dir");
    write_skill(
        &root,
        &[".agents", "skills"],
        "issue-triage",
        "issue-triage",
    );

    let installed = discover_installed_agents(&root);
    let mut agents = installed.get("issue-triage").cloned().unwrap_or_default();
    agents.sort();
    let mut expected = vec![
        "cursor",
        "codex",
        "gemini-cli",
        "github-copilot",
        "openclaw",
        "opencode",
        "openhands",
        "pi",
        "universal",
        "vscode",
        "zed",
    ];
    expected.sort();
    assert_eq!(agents, expected);
}

#[test]
fn installing_only_into_claude_codes_directory_also_marks_it_installed_for_agents_that_read_it_for_compatibility(
) {
    let root = fixture_root("install-claude-only");
    write_skill(
        &root,
        &[".claude", "skills"],
        "issue-triage",
        "issue-triage",
    );

    // Per their own docs, Cursor, OpenCode, and GitHub Copilot all also read
    // `.claude/skills` for compatibility — but Universal, Codex, and Gemini
    // CLI never do, so they must stay out.
    let installed = discover_installed_agents(&root);
    let mut agents = installed.get("issue-triage").cloned().unwrap_or_default();
    agents.sort();
    let mut expected = vec![
        "claude-code",
        "cursor",
        "opencode",
        "github-copilot",
        "vscode",
    ];
    expected.sort();
    assert_eq!(agents, expected);
}

#[test]
fn installing_for_several_distinguishable_destinations_at_once_reports_every_one() {
    let root = fixture_root("install-multi-destination");
    write_skill(
        &root,
        &[".agents", "skills"],
        "issue-triage",
        "issue-triage",
    );
    write_skill(
        &root,
        &[".claude", "skills"],
        "issue-triage",
        "issue-triage",
    );
    write_skill(
        &root,
        &[".windsurf", "skills"],
        "issue-triage",
        "issue-triage",
    );

    let installed = discover_installed_agents(&root);
    let mut agents = installed.get("issue-triage").cloned().unwrap_or_default();
    agents.sort();
    let mut expected = vec![
        "claude-code",
        "cursor",
        "codex",
        "gemini-cli",
        "github-copilot",
        "openclaw",
        "opencode",
        "openhands",
        "pi",
        "universal",
        "vscode",
        "windsurf",
        "zed",
    ];
    expected.sort();
    assert_eq!(agents, expected);
}

#[test]
fn the_skill_shows_up_in_local_discovery_while_its_files_are_on_disk() {
    let root = fixture_root("local-discovery-lifecycle");
    write_skill(
        &root,
        &[".agents", "skills"],
        "issue-triage",
        "Issue Triage",
    );

    // `skill_name` tracks the front matter's own `name:` ("Issue Triage"),
    // not the enclosing directory ("issue-triage") — the real Skills CLI
    // matches `--skill` against that raw front matter value, not the
    // folder it lives in (see `discover_skills_in_directory`).
    let skills = discover_local_skills(&root);
    let found = skills
        .iter()
        .find(|s| s.skill_name == "Issue Triage")
        .expect("discovered locally");
    assert!(found.local);
    assert!(found.installed);
    assert_eq!(found.display_name, "Issue Triage");
}

// --- Deleting un-marks a Skill as installed ---------------------------------

#[test]
fn deleting_the_installed_skill_files_un_marks_it_as_installed_and_drops_it_from_local_discovery() {
    let root = fixture_root("delete-lifecycle");
    write_skill(
        &root,
        &[".agents", "skills"],
        "issue-triage",
        "issue-triage",
    );
    write_skill(
        &root,
        &[".claude", "skills"],
        "issue-triage",
        "issue-triage",
    );

    assert!(discover_installed_agents(&root).contains_key("issue-triage"));
    assert!(!discover_local_skills(&root).is_empty());

    // Deleting a Skill removes its directory from every destination it was
    // installed into — this is what the app's uninstall does per agent.
    remove_skill(&root, &[".agents", "skills"], "issue-triage");
    remove_skill(&root, &[".claude", "skills"], "issue-triage");

    assert!(
        !discover_installed_agents(&root).contains_key("issue-triage"),
        "a deleted Skill must no longer be reported as installed"
    );
    assert!(
        discover_local_skills(&root)
            .iter()
            .all(|s| s.skill_name != "issue-triage"),
        "a deleted Skill must no longer be locally discoverable"
    );
}

#[test]
fn partially_deleting_a_multi_destination_install_only_drops_the_removed_destinations_agents() {
    let root = fixture_root("partial-delete-lifecycle");
    write_skill(
        &root,
        &[".agents", "skills"],
        "issue-triage",
        "issue-triage",
    );
    write_skill(
        &root,
        &[".claude", "skills"],
        "issue-triage",
        "issue-triage",
    );

    remove_skill(&root, &[".claude", "skills"], "issue-triage");

    let installed = discover_installed_agents(&root);
    let mut agents = installed.get("issue-triage").cloned().unwrap_or_default();
    agents.sort();
    let mut expected = vec![
        "cursor",
        "codex",
        "gemini-cli",
        "github-copilot",
        "openclaw",
        "opencode",
        "openhands",
        "pi",
        "universal",
        "vscode",
        "zed",
    ];
    expected.sort();
    assert_eq!(
        agents, expected,
        "the .agents/skills copy must still be reported installed after removing only the Claude Code copy"
    );
}

// --- The same lifecycle through SkillsService::load_state_for, the exact ---
// --- function the GUI calls for whatever folder the user has selected. ----

#[test]
fn load_state_for_marks_a_configured_skill_installed_once_its_files_appear_for_the_selected_folder()
{
    let root = fixture_root("service-install-lifecycle");
    let service = service_for(
        &root,
        r##"
version: 1
groups:
  - id: testing
    name: Testing
    color: "#6B82D9"
    order: 1
    enabled: true
skills:
  - id: triage
    name: Issue Triage
    url: https://www.skills.sh/mattpocock/skills/triage
    group: testing
    preselected: false
    enabled: true
"##,
    );

    let before = service
        .load_state_for(&root, skills_installer_lib::domain::InstallScope::Project)
        .expect("load state before install");
    let triage_before = before
        .skills
        .iter()
        .find(|s| s.skill_name == "triage")
        .expect("configured Skill present");
    assert!(
        !triage_before.installed,
        "not installed before its files exist"
    );

    write_skill(&root, &[".claude", "skills"], "triage", "triage");
    write_skill(&root, &[".agents", "skills"], "triage", "triage");

    let after = service
        .load_state_for(&root, skills_installer_lib::domain::InstallScope::Project)
        .expect("load state after install");
    let triage_after = after
        .skills
        .iter()
        .find(|s| s.skill_name == "triage")
        .expect("configured Skill still present");
    assert!(triage_after.installed);
    let mut agents = triage_after.installed_agents.clone();
    agents.sort();
    let mut expected = vec![
        "claude-code",
        "cursor",
        "codex",
        "gemini-cli",
        "github-copilot",
        "openclaw",
        "opencode",
        "openhands",
        "pi",
        "universal",
        "vscode",
        "zed",
    ];
    expected.sort();
    assert_eq!(agents, expected);
}

#[test]
fn load_state_for_marks_a_configured_skill_uninstalled_once_its_files_are_deleted() {
    let root = fixture_root("service-delete-lifecycle");
    let service = service_for(
        &root,
        r##"
version: 1
groups:
  - id: testing
    name: Testing
    color: "#6B82D9"
    order: 1
    enabled: true
skills:
  - id: triage
    name: Issue Triage
    url: https://www.skills.sh/mattpocock/skills/triage
    group: testing
    preselected: false
    enabled: true
"##,
    );

    write_skill(&root, &[".claude", "skills"], "triage", "triage");
    let installed = service
        .load_state_for(&root, skills_installer_lib::domain::InstallScope::Project)
        .expect("load state while installed");
    assert!(
        installed
            .skills
            .iter()
            .find(|s| s.skill_name == "triage")
            .expect("configured Skill present")
            .installed
    );

    remove_skill(&root, &[".claude", "skills"], "triage");

    let deleted = service
        .load_state_for(&root, skills_installer_lib::domain::InstallScope::Project)
        .expect("load state after delete");
    let triage_after = deleted
        .skills
        .iter()
        .find(|s| s.skill_name == "triage")
        .expect("configured Skill still present in the catalog");
    assert!(
        !triage_after.installed,
        "must no longer be installed once its files are gone"
    );
    assert!(triage_after.installed_agents.is_empty());
}

#[test]
fn load_state_for_scopes_installed_status_to_whatever_folder_is_passed_in() {
    let root = fixture_root("service-folder-scoping");
    let service = service_for(&root, EMPTY_CONFIG);
    write_skill(
        &root,
        &[".agents", "skills"],
        "issue-triage",
        "Issue Triage",
    );

    let here = service
        .load_state_for(&root, skills_installer_lib::domain::InstallScope::Project)
        .expect("load state for the fixture root");
    assert!(here
        .skills
        .iter()
        .any(|s| s.skill_name == "Issue Triage" && s.installed));

    // A different, empty folder must not inherit that status — the GUI's
    // "Installed" badge tracks whatever folder is currently selected, not
    // wherever the app happened to launch from. The Skill itself still shows
    // up (the local catalog source directory is a fixed preference, not
    // scoped to the queried folder) — only `installed` must flip.
    let elsewhere = fixture_root("service-folder-scoping-elsewhere");
    let there = service
        .load_state_for(
            &elsewhere,
            skills_installer_lib::domain::InstallScope::Project,
        )
        .expect("load state for an empty folder");
    let triage_there = there
        .skills
        .iter()
        .find(|s| s.skill_name == "Issue Triage")
        .expect("still listed in the catalog");
    assert!(
        !triage_there.installed,
        "an unrelated folder must not show a Skill installed there as installed"
    );
}

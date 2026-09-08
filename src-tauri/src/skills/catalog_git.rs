use std::path::Path;
use std::process::Command;

/// Whether `dir` sits inside a git working tree — a cheap `git` shell-out.
/// Gates the fast path in `SkillsService::check_local_updates`: with the
/// catalog under version control, `dirty_skill_names` tells us exactly
/// which skills need re-signing instead of hashing the whole catalog every
/// time. Without it (`false`), there's no cheap way to know what changed —
/// callers fall back to signing everything, and the GUI suggests running
/// `git init` there.
pub fn is_versioned(dir: &Path) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|out| out.status.success() && String::from_utf8_lossy(&out.stdout).trim() == "true")
        .unwrap_or(false)
}

/// Top-level catalog skill directory names with uncommitted changes
/// (`git status --porcelain`, deduped down to each entry's first path
/// segment) — only these actually need re-signing on this pass.
pub fn dirty_skill_names(dir: &Path) -> Vec<String> {
    let Ok(output) = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["status", "--porcelain"])
        .output()
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }

    let mut names: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            // Porcelain v1 format is "XY path" (or "XY orig -> path" for a
            // rename) — the path portion starts right after the 2-letter
            // status + 1 space.
            let path = line.get(3..)?;
            let path = path.rsplit(" -> ").next().unwrap_or(path);
            let path = path.trim_matches('"');
            path.split('/').next().map(str::to_string)
        })
        .filter(|name| !name.is_empty())
        .collect();
    names.sort();
    names.dedup();
    names
}

/// Whether `dir`'s working tree currently has any uncommitted changes at
/// all (tracked or untracked) — used after signing to decide whether to
/// suggest a commit.
pub fn is_dirty(dir: &Path) -> bool {
    !dirty_skill_names(dir).is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(dir: &Path, args: &[&str]) {
        let status = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?} failed");
    }

    fn init_repo(dir: &Path) {
        run(dir, &["init", "-q"]);
        run(dir, &["config", "user.email", "test@example.com"]);
        run(dir, &["config", "user.name", "Test"]);
    }

    #[test]
    fn a_plain_directory_is_not_versioned() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(!is_versioned(tmp.path()));
        assert!(dirty_skill_names(tmp.path()).is_empty());
    }

    #[test]
    fn a_git_repo_is_versioned() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path());
        assert!(is_versioned(tmp.path()));
    }

    #[test]
    fn a_clean_repo_has_no_dirty_skills() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path());
        std::fs::create_dir(tmp.path().join("triage")).unwrap();
        std::fs::write(tmp.path().join("triage/SKILL.md"), "v1").unwrap();
        run(tmp.path(), &["add", "-A"]);
        run(tmp.path(), &["commit", "-q", "-m", "init"]);

        assert!(dirty_skill_names(tmp.path()).is_empty());
        assert!(!is_dirty(tmp.path()));
    }

    #[test]
    fn an_edited_tracked_file_flags_its_skill_folder() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path());
        std::fs::create_dir(tmp.path().join("triage")).unwrap();
        std::fs::write(tmp.path().join("triage/SKILL.md"), "v1").unwrap();
        run(tmp.path(), &["add", "-A"]);
        run(tmp.path(), &["commit", "-q", "-m", "init"]);

        std::fs::write(tmp.path().join("triage/SKILL.md"), "v2").unwrap();

        assert_eq!(dirty_skill_names(tmp.path()), vec!["triage".to_string()]);
        assert!(is_dirty(tmp.path()));
    }

    #[test]
    fn an_untracked_new_skill_folder_is_flagged_too() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path());
        std::fs::write(tmp.path().join(".gitkeep"), "").unwrap();
        run(tmp.path(), &["add", "-A"]);
        run(tmp.path(), &["commit", "-q", "-m", "init"]);

        std::fs::create_dir(tmp.path().join("new-skill")).unwrap();
        std::fs::write(tmp.path().join("new-skill/SKILL.md"), "v1").unwrap();

        assert_eq!(dirty_skill_names(tmp.path()), vec!["new-skill".to_string()]);
    }

    #[test]
    fn multiple_dirty_folders_are_deduped_and_sorted() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path());
        for name in ["zeta", "alpha"] {
            std::fs::create_dir(tmp.path().join(name)).unwrap();
            std::fs::write(tmp.path().join(name).join("SKILL.md"), "v1").unwrap();
        }
        run(tmp.path(), &["add", "-A"]);
        run(tmp.path(), &["commit", "-q", "-m", "init"]);

        std::fs::write(tmp.path().join("zeta/SKILL.md"), "v2").unwrap();
        std::fs::write(tmp.path().join("zeta/extra.md"), "new file").unwrap();
        std::fs::write(tmp.path().join("alpha/SKILL.md"), "v2").unwrap();

        assert_eq!(
            dirty_skill_names(tmp.path()),
            vec!["alpha".to_string(), "zeta".to_string()]
        );
    }
}

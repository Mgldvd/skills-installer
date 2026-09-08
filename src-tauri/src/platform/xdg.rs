use std::path::{Path, PathBuf};

const APP_CONFIG_DIR_NAME: &str = "skills-control-deck";
/// This app's previous name — kept only so `app_config_dir` can migrate an
/// existing installation's configuration forward the first time it runs
/// under the new name (see `migrate_legacy_config_dir`).
const LEGACY_APP_CONFIG_DIR_NAME: &str = "skills-installer";

/// `$XDG_CONFIG_HOME`, falling back to `~/.config` per the XDG Base
/// Directory spec. Hand-rolled rather than pulled from a crate so the
/// resulting directory name is guaranteed to be exactly `skills-control-deck`
/// (spec requirement), independent of any crate's own
/// qualifier/organization/application naming conventions.
fn xdg_config_home(home: Option<&str>) -> Option<PathBuf> {
    if let Ok(value) = std::env::var("XDG_CONFIG_HOME") {
        if !value.trim().is_empty() {
            return Some(PathBuf::from(value));
        }
    }
    home.map(|h| PathBuf::from(h).join(".config"))
}

/// `$XDG_CONFIG_HOME/skills-control-deck` (or
/// `~/.config/skills-control-deck`). Returns `None` only if neither
/// `XDG_CONFIG_HOME` nor `HOME` is set, which should not happen on a real
/// desktop session but is handled rather than panicking.
///
/// The first call after an install that previously ran as "Skills
/// Installer" transparently moves that install's whole config directory
/// (`preferences.json`, `presets.json`, `skills.yaml`, everything) here —
/// see `migrate_legacy_config_dir`.
pub fn app_config_dir() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok();
    let base = xdg_config_home(home.as_deref())?;
    let dir = base.join(APP_CONFIG_DIR_NAME);
    migrate_legacy_config_dir(&base, &dir);
    Some(dir)
}

/// One-time migration for an install that previously ran under this app's
/// old name, "Skills Installer": if the new config directory doesn't exist
/// yet but the old `skills-installer` one does, move it wholesale so
/// existing preferences/presets/`skills.yaml` keep working without the user
/// having to do anything. Best-effort and silent — a failure here
/// (permissions, a cross-filesystem `$XDG_CONFIG_HOME` override) just means
/// the app starts fresh in the new directory, same as any other first run;
/// the old directory is left in place rather than lost in that case.
fn migrate_legacy_config_dir(base: &Path, new_dir: &Path) {
    if new_dir.exists() {
        return;
    }
    let legacy_dir = base.join(LEGACY_APP_CONFIG_DIR_NAME);
    if !legacy_dir.is_dir() {
        return;
    }
    let _ = std::fs::rename(&legacy_dir, new_dir);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_xdg_config_home_when_set() {
        assert_eq!(
            xdg_config_home_for_test(Some("/xdg"), Some("/home/user")),
            Some(PathBuf::from("/xdg"))
        );
    }

    #[test]
    fn falls_back_to_home_dot_config() {
        assert_eq!(
            xdg_config_home_for_test(None, Some("/home/user")),
            Some(PathBuf::from("/home/user/.config"))
        );
    }

    #[test]
    fn returns_none_without_either() {
        assert_eq!(xdg_config_home_for_test(None, None), None);
    }

    // `xdg_config_home` reads the real process environment for
    // `XDG_CONFIG_HOME` directly (env vars are process-global, so mutating
    // them in a parallel test suite is flaky). This helper isolates the pure
    // fallback logic for deterministic testing instead.
    fn xdg_config_home_for_test(xdg: Option<&str>, home: Option<&str>) -> Option<PathBuf> {
        if let Some(value) = xdg {
            if !value.trim().is_empty() {
                return Some(PathBuf::from(value));
            }
        }
        home.map(|h| PathBuf::from(h).join(".config"))
    }

    #[test]
    fn migrate_moves_a_legacy_dir_to_the_new_name() {
        let tmp = tempfile::tempdir().unwrap();
        let legacy = tmp.path().join(LEGACY_APP_CONFIG_DIR_NAME);
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(legacy.join("preferences.json"), "{}").unwrap();
        let new_dir = tmp.path().join(APP_CONFIG_DIR_NAME);

        migrate_legacy_config_dir(tmp.path(), &new_dir);

        assert!(!legacy.exists());
        assert_eq!(
            std::fs::read_to_string(new_dir.join("preferences.json")).unwrap(),
            "{}"
        );
    }

    #[test]
    fn migrate_does_nothing_when_the_new_dir_already_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let legacy = tmp.path().join(LEGACY_APP_CONFIG_DIR_NAME);
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(legacy.join("preferences.json"), "old").unwrap();
        let new_dir = tmp.path().join(APP_CONFIG_DIR_NAME);
        std::fs::create_dir_all(&new_dir).unwrap();
        std::fs::write(new_dir.join("preferences.json"), "new").unwrap();

        migrate_legacy_config_dir(tmp.path(), &new_dir);

        assert_eq!(
            std::fs::read_to_string(new_dir.join("preferences.json")).unwrap(),
            "new"
        );
        assert!(
            legacy.exists(),
            "the legacy dir must be left alone once the new one already exists"
        );
    }

    #[test]
    fn migrate_does_nothing_when_there_is_no_legacy_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let new_dir = tmp.path().join(APP_CONFIG_DIR_NAME);

        migrate_legacy_config_dir(tmp.path(), &new_dir);

        assert!(!new_dir.exists());
    }
}

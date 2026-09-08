use std::path::{Path, PathBuf};

use crate::platform::app_config_dir;

/// Where a loaded configuration actually came from. Mirrors the documented
/// precedence chain exactly:
///   1. `--config <path>`
///   2. `./skills.yaml`
///   3. `./skills.confg` (legacy filename)
///   4. `$XDG_CONFIG_HOME/skills-control-deck/skills.yaml` (falls back to
///      `~/.config/skills-control-deck/skills.yaml`)
///   5. embedded default configuration (compiled into the binary)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigSource {
    Explicit(PathBuf),
    Cwd(PathBuf),
    LegacyCwd(PathBuf),
    UserConfig(PathBuf),
    Embedded,
}

impl ConfigSource {
    pub fn path(&self) -> Option<&Path> {
        match self {
            ConfigSource::Explicit(p)
            | ConfigSource::Cwd(p)
            | ConfigSource::LegacyCwd(p)
            | ConfigSource::UserConfig(p) => Some(p),
            ConfigSource::Embedded => None,
        }
    }

    pub fn is_embedded(&self) -> bool {
        matches!(self, ConfigSource::Embedded)
    }
}

/// Pure precedence logic, parameterized over the two things that would
/// otherwise make this untestable (the real cwd and the real user config
/// path) so tests can exercise every branch with tempdirs instead of
/// mutating global process state.
pub fn locate(
    explicit_override: Option<&Path>,
    cwd: &Path,
    user_config_path: Option<&Path>,
) -> ConfigSource {
    if let Some(path) = explicit_override {
        return ConfigSource::Explicit(path.to_path_buf());
    }

    let cwd_yaml = cwd.join("skills.yaml");
    if cwd_yaml.is_file() {
        return ConfigSource::Cwd(cwd_yaml);
    }

    let cwd_confg = cwd.join("skills.confg");
    if cwd_confg.is_file() {
        return ConfigSource::LegacyCwd(cwd_confg);
    }

    if let Some(user_path) = user_config_path {
        if user_path.is_file() {
            return ConfigSource::UserConfig(user_path.to_path_buf());
        }
    }

    ConfigSource::Embedded
}

/// Real-environment entry point used by `app`/`cli`.
pub fn locate_real(explicit_override: Option<&Path>, cwd: &Path) -> ConfigSource {
    locate(explicit_override, cwd, user_config_path().as_deref())
}

pub fn user_config_path() -> Option<PathBuf> {
    app_config_dir().map(|dir| dir.join("skills.yaml"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn explicit_override_wins_over_everything() {
        let tmp = tempfile::tempdir().unwrap();
        let explicit = tmp.path().join("custom.yaml");
        fs::write(&explicit, "skills: []").unwrap();
        fs::write(tmp.path().join("skills.yaml"), "skills: []").unwrap();

        let source = locate(Some(&explicit), tmp.path(), None);
        assert_eq!(source, ConfigSource::Explicit(explicit));
    }

    #[test]
    fn cwd_yaml_wins_over_legacy_and_user_config() {
        let tmp = tempfile::tempdir().unwrap();
        let cwd_yaml = tmp.path().join("skills.yaml");
        fs::write(&cwd_yaml, "skills: []").unwrap();
        fs::write(tmp.path().join("skills.confg"), "skills: []").unwrap();

        let source = locate(None, tmp.path(), None);
        assert_eq!(source, ConfigSource::Cwd(cwd_yaml));
    }

    #[test]
    fn legacy_cwd_wins_over_user_config() {
        let tmp = tempfile::tempdir().unwrap();
        let legacy = tmp.path().join("skills.confg");
        fs::write(&legacy, "skills: []").unwrap();
        let user_dir = tmp.path().join("user-config");
        fs::create_dir_all(&user_dir).unwrap();
        let user_path = user_dir.join("skills.yaml");
        fs::write(&user_path, "skills: []").unwrap();

        let source = locate(None, tmp.path(), Some(&user_path));
        assert_eq!(source, ConfigSource::LegacyCwd(legacy));
    }

    #[test]
    fn user_config_wins_over_embedded() {
        let tmp = tempfile::tempdir().unwrap();
        let cwd = tmp.path().join("project");
        fs::create_dir_all(&cwd).unwrap();
        let user_path = tmp.path().join("user-config").join("skills.yaml");
        fs::create_dir_all(user_path.parent().unwrap()).unwrap();
        fs::write(&user_path, "skills: []").unwrap();

        let source = locate(None, &cwd, Some(&user_path));
        assert_eq!(source, ConfigSource::UserConfig(user_path));
    }

    #[test]
    fn falls_back_to_embedded_when_nothing_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let missing_user_path = tmp.path().join("does-not-exist").join("skills.yaml");

        let source = locate(None, tmp.path(), Some(&missing_user_path));
        assert_eq!(source, ConfigSource::Embedded);
    }
}

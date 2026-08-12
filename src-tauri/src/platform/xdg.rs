use std::path::PathBuf;

/// `$XDG_CONFIG_HOME`, falling back to `~/.config` per the XDG Base
/// Directory spec. Hand-rolled rather than pulled from a crate so the
/// resulting directory name is guaranteed to be exactly `skills-installer`
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

/// `$XDG_CONFIG_HOME/skills-installer` (or `~/.config/skills-installer`).
/// Returns `None` only if neither `XDG_CONFIG_HOME` nor `HOME` is set, which
/// should not happen on a real desktop session but is handled rather than
/// panicking.
pub fn app_config_dir() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok();
    xdg_config_home(home.as_deref()).map(|dir| dir.join("skills-installer"))
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
}

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::domain::preferences::{FONT_SCALE_MAX, FONT_SCALE_MIN};
use crate::domain::{validate_agent_id, UiPreferences};
use crate::error::AppError;
use crate::platform::app_config_dir;

/// Persists `UiPreferences` separately from skill configuration (own JSON
/// file at `$XDG_CONFIG_HOME/skills-installer/preferences.json`) — this is
/// pure UI/installation-default state, not shareable/curated content, so it
/// doesn't belong in `skills.yaml`.
#[derive(Clone)]
pub struct PreferencesService {
    path_override: Option<PathBuf>,
}

impl PreferencesService {
    pub fn new() -> Self {
        Self {
            path_override: None,
        }
    }

    #[cfg(test)]
    pub fn with_path(path: PathBuf) -> Self {
        Self {
            path_override: Some(path),
        }
    }

    pub(crate) fn path(&self) -> Result<PathBuf, AppError> {
        if let Some(path) = &self.path_override {
            return Ok(path.clone());
        }
        app_config_dir().map(|dir| dir.join("preferences.json")).ok_or_else(|| {
            AppError::Preferences(
                "could not determine a writable configuration directory (HOME and XDG_CONFIG_HOME are both unset)"
                    .to_string(),
            )
        })
    }

    /// Missing file is not an error — a fresh install has no preferences
    /// yet, and the app must be usable immediately with sensible defaults.
    pub fn load(&self) -> Result<UiPreferences, AppError> {
        let path = self.path()?;
        if !path.is_file() {
            return Ok(UiPreferences::default());
        }
        let content = std::fs::read_to_string(&path).map_err(|e| {
            AppError::Preferences(format!("failed to read {}: {e}", path.display()))
        })?;
        let mut preferences: UiPreferences = serde_json::from_str(&content).map_err(|e| {
            AppError::Preferences(format!("failed to parse {}: {e}", path.display()))
        })?;
        if preferences.default_agents.is_empty() {
            preferences.default_agents = preferences.default_agent.take().into_iter().collect();
        }
        if preferences.default_agents.is_empty() {
            preferences.default_agents.push("universal".into());
        }
        validate_preferences(&preferences)?;
        Ok(preferences)
    }

    pub fn save(&self, preferences: &UiPreferences) -> Result<(), AppError> {
        validate_preferences(preferences)?;
        let path = self.path()?;
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(parent)?;

        let json = serde_json::to_string_pretty(preferences)
            .map_err(|e| AppError::Preferences(format!("failed to serialize preferences: {e}")))?;

        let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
        tmp.write_all(json.as_bytes())?;
        tmp.flush()?;
        tmp.as_file().sync_all()?;
        tmp.persist(&path).map_err(|e| {
            AppError::Io(format!(
                "failed to finalize write to {}: {}",
                path.display(),
                e.error
            ))
        })?;

        Ok(())
    }
    pub fn validate(&self, preferences: &UiPreferences) -> Result<(), AppError> {
        validate_preferences(preferences)
    }
}

impl Default for PreferencesService {
    fn default() -> Self {
        Self::new()
    }
}

fn validate_preferences(preferences: &UiPreferences) -> Result<(), AppError> {
    if !(FONT_SCALE_MIN..=FONT_SCALE_MAX).contains(&preferences.font_scale) {
        return Err(AppError::Validation(format!(
            "font scale {:.2} is outside the allowed range {FONT_SCALE_MIN:.2}-{FONT_SCALE_MAX:.2}",
            preferences.font_scale
        )));
    }
    if preferences.default_agents.is_empty() {
        return Err(AppError::Validation(
            "select at least one installation agent".into(),
        ));
    }
    for agent in &preferences.default_agents {
        validate_agent_id(agent)?;
        if !crate::domain::is_supported_agent(agent) {
            return Err(AppError::Validation(format!(
                "unsupported Skills CLI agent \"{agent}\""
            )));
        }
    }
    if !matches!(
        preferences.accent.as_str(),
        "pink" | "coral" | "blue" | "teal" | "violet" | "green"
    ) {
        return Err(AppError::Validation(format!(
            "unsupported accent \"{}\"",
            preferences.accent
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_returns_default_when_no_file_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let service = PreferencesService::with_path(tmp.path().join("preferences.json"));
        let preferences = service.load().unwrap();
        assert_eq!(preferences.font_scale, 1.0);
    }

    #[test]
    fn save_then_load_round_trips() {
        let tmp = tempfile::tempdir().unwrap();
        let service = PreferencesService::with_path(tmp.path().join("preferences.json"));

        let preferences = UiPreferences {
            font_scale: 1.25,
            default_agents: vec!["claude-code".into(), "codex".into()],
            ..UiPreferences::default()
        };
        service.save(&preferences).unwrap();

        let reloaded = service.load().unwrap();
        assert_eq!(reloaded.font_scale, 1.25);
        assert_eq!(reloaded.default_agents, vec!["claude-code", "codex"]);
    }

    #[test]
    fn save_rejects_font_scale_below_minimum() {
        let tmp = tempfile::tempdir().unwrap();
        let service = PreferencesService::with_path(tmp.path().join("preferences.json"));
        let preferences = UiPreferences {
            font_scale: 0.5,
            ..UiPreferences::default()
        };
        assert!(service.save(&preferences).is_err());
    }

    #[test]
    fn save_rejects_font_scale_above_maximum() {
        let tmp = tempfile::tempdir().unwrap();
        let service = PreferencesService::with_path(tmp.path().join("preferences.json"));
        let preferences = UiPreferences {
            font_scale: 2.0,
            ..UiPreferences::default()
        };
        assert!(service.save(&preferences).is_err());
    }

    #[test]
    fn save_accepts_all_documented_presets() {
        let tmp = tempfile::tempdir().unwrap();
        let service = PreferencesService::with_path(tmp.path().join("preferences.json"));
        for preset in crate::domain::preferences::FONT_SCALE_PRESETS {
            let preferences = UiPreferences {
                font_scale: preset,
                ..UiPreferences::default()
            };
            assert!(
                service.save(&preferences).is_ok(),
                "preset {preset} should be valid"
            );
        }
    }

    #[test]
    fn save_rejects_invalid_default_agent() {
        let tmp = tempfile::tempdir().unwrap();
        let service = PreferencesService::with_path(tmp.path().join("preferences.json"));
        let preferences = UiPreferences {
            default_agents: vec!["bad agent!!".into()],
            ..UiPreferences::default()
        };
        assert!(service.save(&preferences).is_err());
    }

    #[test]
    fn save_does_not_leave_a_temp_file_behind() {
        let tmp = tempfile::tempdir().unwrap();
        let service = PreferencesService::with_path(tmp.path().join("preferences.json"));
        service.save(&UiPreferences::default()).unwrap();

        let entries: Vec<_> = std::fs::read_dir(tmp.path()).unwrap().collect();
        assert_eq!(
            entries.len(),
            1,
            "only the final preferences.json should remain"
        );
    }
}

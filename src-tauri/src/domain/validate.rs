use crate::error::AppError;

const MAX_ID_LEN: usize = 64;

/// Shared charset rule for skill/group/agent ids: lowercase-friendly slug
/// characters only. Applied in Rust regardless of what the frontend already
/// checked, per the "never trust frontend validation alone" requirement —
/// these ids end up in file paths, config keys, and (for agent) a CLI
/// argument, so a permissive charset here would be a real defect, not just
/// cosmetic.
fn is_valid_slug(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_ID_LEN
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        && value
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphanumeric())
}

pub fn validate_skill_id(value: &str) -> Result<(), AppError> {
    if is_valid_slug(value) {
        Ok(())
    } else {
        Err(AppError::Validation(format!(
            "skill id \"{value}\" must be a non-empty slug (letters, digits, '-', '_', max {MAX_ID_LEN} chars) starting with a letter or digit"
        )))
    }
}

pub fn validate_group_id(value: &str) -> Result<(), AppError> {
    if is_valid_slug(value) {
        Ok(())
    } else {
        Err(AppError::Validation(format!(
            "group id \"{value}\" must be a non-empty slug (letters, digits, '-', '_', max {MAX_ID_LEN} chars) starting with a letter or digit"
        )))
    }
}

pub fn validate_agent_id(value: &str) -> Result<(), AppError> {
    if is_valid_slug(value) {
        Ok(())
    } else {
        Err(AppError::Validation(format!(
            "agent id \"{value}\" must be a non-empty slug (letters, digits, '-', '_', max {MAX_ID_LEN} chars) starting with a letter or digit"
        )))
    }
}

pub fn is_supported_agent(agent: &str) -> bool {
    matches!(
        agent,
        "universal"
            | "claude-code"
            | "codex"
            | "gemini-cli"
            | "cursor"
            | "windsurf"
            | "opencode"
            | "github-copilot"
    )
}

/// Best-effort conversion of arbitrary text (a directory name, a legacy
/// config's missing/duplicate id, ...) into something that passes
/// `is_valid_slug`. Used where we must tolerate messy input rather than
/// reject it outright (local skill discovery, legacy config migration) —
/// strict rejection via `validate_*` is reserved for user-submitted new
/// ids from the GUI/CLI.
pub fn slugify(input: &str) -> String {
    let slug: String = input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    // Trimming '-' from both ends guarantees the first remaining character
    // (if any) is alphanumeric, since only alphanumerics and '-' occur here.
    let trimmed: String = slug.trim_matches('-').chars().take(MAX_ID_LEN).collect();
    if trimmed.is_empty() {
        "item".to_string()
    } else {
        trimmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_typical_slugs() {
        assert!(validate_skill_id("triage").is_ok());
        assert!(validate_group_id("frontend-team_2").is_ok());
        assert!(validate_agent_id("claude-code").is_ok());
    }

    #[test]
    fn rejects_empty_and_oversized() {
        assert!(validate_skill_id("").is_err());
        assert!(validate_skill_id(&"a".repeat(MAX_ID_LEN + 1)).is_err());
    }

    #[test]
    fn rejects_unsafe_characters() {
        for bad in ["../etc/passwd", "rm -rf", "a;b", "a b", "a/b", "$(id)"] {
            assert!(
                validate_skill_id(bad).is_err(),
                "expected {bad:?} to be rejected"
            );
        }
    }

    #[test]
    fn rejects_leading_symbol() {
        assert!(validate_group_id("-leading-dash").is_err());
        assert!(validate_group_id("_leading-underscore").is_err());
    }

    #[test]
    fn slugify_produces_valid_ids() {
        for input in [
            "Issue Triage!",
            "tauri-v2",
            "  weird///chars??  ",
            "",
            "---",
        ] {
            let slug = slugify(input);
            assert!(
                validate_skill_id(&slug).is_ok(),
                "slugify({input:?}) = {slug:?} was not valid"
            );
        }
    }

    #[test]
    fn slugify_is_deterministic() {
        assert_eq!(slugify("Issue Triage"), slugify("Issue Triage"));
    }
}

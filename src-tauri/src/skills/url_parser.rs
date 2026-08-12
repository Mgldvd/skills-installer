use url::Url;

use crate::domain::ParsedSkillSource;
use crate::error::AppError;

/// Abstraction over "turn a skills.sh URL into a `ParsedSkillSource`" so the
/// rest of the app (config migration, the Add-dialog preview command, the
/// source resolver) never parses URLs itself. `Send + Sync` because it's held
/// behind `Arc` in `ApplicationServices` and used from both the async Tauri
/// command layer and the synchronous CLI path.
pub trait SkillUrlParser: Send + Sync {
    fn parse(&self, raw_url: &str) -> Result<ParsedSkillSource, AppError>;
}

/// The only real implementation. Deliberately uses the `url` crate (WHATWG
/// URL Standard parser) instead of hand-rolled string splitting: this is a
/// security boundary (requirement: reject `javascript:`/`file:`/`data:`, host
/// confusion like `skills.sh.evil.com`, userinfo tricks like
/// `https://skills.sh@evil.com/...`), and those are exactly the classes of
/// bug a naive `split('/')` parser gets wrong.
pub struct SkillsShUrlParser;

impl SkillUrlParser for SkillsShUrlParser {
    fn parse(&self, raw_url: &str) -> Result<ParsedSkillSource, AppError> {
        let trimmed = raw_url.trim();
        let parsed = Url::parse(trimmed)
            .map_err(|_| AppError::UrlParse(format!("\"{raw_url}\" is not a valid URL")))?;

        if parsed.scheme() != "https" {
            return Err(AppError::UrlParse(format!(
                "only https:// URLs are supported (got scheme \"{}\")",
                parsed.scheme()
            )));
        }

        if !parsed.username().is_empty() || parsed.password().is_some() {
            return Err(AppError::UrlParse(
                "URL must not contain embedded credentials".to_string(),
            ));
        }

        let host = parsed
            .host_str()
            .ok_or_else(|| AppError::UrlParse("URL has no host".to_string()))?;
        if host != "skills.sh" && host != "www.skills.sh" {
            return Err(AppError::UrlParse(format!(
                "expected host skills.sh or www.skills.sh (got \"{host}\")"
            )));
        }

        let segments: Vec<&str> = parsed
            .path_segments()
            .map(|iter| iter.filter(|segment| !segment.is_empty()).collect())
            .unwrap_or_default();

        if segments.len() != 3 {
            return Err(AppError::UrlParse(format!(
                "expected https://www.skills.sh/<owner>/<repository>/<skill-name> (found {} path segment(s))",
                segments.len()
            )));
        }

        let [owner, repository, skill_name] = [segments[0], segments[1], segments[2]];
        for (label, value) in [
            ("owner", owner),
            ("repository", repository),
            ("skill name", skill_name),
        ] {
            if !is_safe_path_segment(value) {
                return Err(AppError::UrlParse(format!(
                    "{label} \"{value}\" contains characters that are not allowed"
                )));
            }
        }

        Ok(ParsedSkillSource {
            canonical_url: format!("https://www.skills.sh/{owner}/{repository}/{skill_name}"),
            owner: owner.to_string(),
            repository: repository.to_string(),
            skill_name: skill_name.to_string(),
            repository_url: format!("https://github.com/{owner}/{repository}"),
        })
    }
}

fn is_safe_path_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_spec_example() {
        let parsed = SkillsShUrlParser
            .parse("https://www.skills.sh/mattpocock/skills/triage")
            .unwrap();
        assert_eq!(parsed.owner, "mattpocock");
        assert_eq!(parsed.repository, "skills");
        assert_eq!(parsed.skill_name, "triage");
        assert_eq!(
            parsed.repository_url,
            "https://github.com/mattpocock/skills"
        );
        assert_eq!(
            parsed.canonical_url,
            "https://www.skills.sh/mattpocock/skills/triage"
        );
    }

    #[test]
    fn parses_the_real_world_example_without_www() {
        // Confirmed live example: https://www.skills.sh/pbakaus/impeccable/impeccable
        let parsed = SkillsShUrlParser
            .parse("https://skills.sh/pbakaus/impeccable/impeccable")
            .unwrap();
        assert_eq!(parsed.owner, "pbakaus");
        assert_eq!(parsed.repository, "impeccable");
        assert_eq!(parsed.skill_name, "impeccable");
        assert_eq!(
            parsed.repository_url,
            "https://github.com/pbakaus/impeccable"
        );
    }

    #[test]
    fn rejects_wrong_host() {
        assert!(SkillsShUrlParser.parse("https://example.com/test").is_err());
    }

    #[test]
    fn rejects_under_specified_paths() {
        assert!(SkillsShUrlParser.parse("https://skills.sh/owner").is_err());
        assert!(SkillsShUrlParser
            .parse("https://skills.sh/owner/repository")
            .is_err());
        assert!(SkillsShUrlParser.parse("https://skills.sh/").is_err());
    }

    #[test]
    fn rejects_dangerous_schemes() {
        for bad in [
            "javascript:alert(1)",
            "file:///etc/passwd",
            "data:text/html,<script>alert(1)</script>",
        ] {
            assert!(
                SkillsShUrlParser.parse(bad).is_err(),
                "expected {bad:?} to be rejected"
            );
        }
    }

    #[test]
    fn rejects_host_confusion_and_credentials() {
        assert!(SkillsShUrlParser
            .parse("https://skills.sh.evil.com/mattpocock/skills/triage")
            .is_err());
        assert!(SkillsShUrlParser
            .parse("https://skills.sh@evil.com/mattpocock/skills/triage")
            .is_err());
        assert!(SkillsShUrlParser
            .parse("https://user:pass@skills.sh/mattpocock/skills/triage")
            .is_err());
    }

    #[test]
    fn rejects_unsafe_path_segments() {
        assert!(SkillsShUrlParser
            .parse("https://skills.sh/owner/repo/../../etc")
            .is_err());
        assert!(SkillsShUrlParser
            .parse("https://skills.sh/owner/repo/name%20with%20spaces")
            .is_err());
    }
}

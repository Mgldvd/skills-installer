use std::path::{Path, PathBuf};

use serde::Serialize;
use tokio::process::Command;
use url::Url;

use crate::error::AppError;
use crate::installer::DependencyResolver;
use crate::skills::extract_front_matter;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackSkillPreview {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackPreview {
    pub canonical_url: String,
    pub suggested_name: String,
    pub skills: Vec<PackSkillPreview>,
}

pub async fn discover_pack(raw_url: &str) -> Result<PackPreview, AppError> {
    let (canonical_url, pack_id) = parse_pack_url(raw_url)?;
    let temp = tempfile::tempdir()?;
    let resolved = DependencyResolver::new().resolve().await?;
    let mut args = resolved.leading_args;
    args.extend([
        "add".into(),
        canonical_url.clone(),
        "--skill".into(),
        "*".into(),
        "--agent".into(),
        "codex".into(),
        "--copy".into(),
        "--yes".into(),
    ]);
    let output = Command::new(&resolved.program)
        .args(args)
        .current_dir(temp.path())
        .env("NO_COLOR", "1")
        .output()
        .await
        .map_err(|error| AppError::Process(format!("could not inspect pack: {error}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = if stderr.trim().is_empty() {
            stdout.trim()
        } else {
            stderr.trim()
        };
        return Err(AppError::Process(format!(
            "could not inspect pack: {detail}"
        )));
    }

    let mut skill_files = Vec::new();
    collect_skill_files(temp.path(), 0, &mut skill_files)?;
    let mut skills = skill_files
        .into_iter()
        .filter_map(|path| {
            let content = std::fs::read_to_string(&path).ok()?;
            let front = extract_front_matter(&content)?;
            let fallback = path.parent()?.file_name()?.to_str()?.to_string();
            Some(PackSkillPreview {
                name: front
                    .name
                    .filter(|name| !name.trim().is_empty())
                    .unwrap_or(fallback),
                description: front.description.unwrap_or_default(),
            })
        })
        .collect::<Vec<_>>();
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills.dedup_by(|a, b| a.name == b.name);
    if skills.is_empty() {
        return Err(AppError::Validation(
            "the pack did not contain any valid Skills".into(),
        ));
    }
    Ok(PackPreview {
        canonical_url,
        suggested_name: format!("Pack {pack_id}"),
        skills,
    })
}

fn parse_pack_url(raw_url: &str) -> Result<(String, String), AppError> {
    let parsed =
        Url::parse(raw_url.trim()).map_err(|_| AppError::UrlParse("invalid pack URL".into()))?;
    if parsed.scheme() != "https"
        || !matches!(parsed.host_str(), Some("skills.sh" | "www.skills.sh"))
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return Err(AppError::UrlParse(
            "expected https://skills.sh/p/<pack-id>".into(),
        ));
    }
    let segments = parsed
        .path_segments()
        .map(|parts| parts.filter(|part| !part.is_empty()).collect::<Vec<_>>())
        .unwrap_or_default();
    if segments.len() != 2
        || segments[0] != "p"
        || !segments[1]
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AppError::UrlParse(
            "expected https://skills.sh/p/<pack-id>".into(),
        ));
    }
    Ok((
        format!("https://skills.sh/p/{}", segments[1]),
        segments[1].to_string(),
    ))
}

fn collect_skill_files(
    dir: &Path,
    depth: usize,
    output: &mut Vec<PathBuf>,
) -> Result<(), AppError> {
    if depth > 6 {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_skill_files(&path, depth + 1, output)?;
        } else if path.file_name().and_then(|name| name.to_str()) == Some("SKILL.md") {
            output.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_pack_url;

    #[test]
    fn accepts_only_safe_skills_sh_pack_urls() {
        assert_eq!(
            parse_pack_url("https://www.skills.sh/p/abc_123").unwrap().0,
            "https://skills.sh/p/abc_123"
        );
        assert!(parse_pack_url("https://skills.sh/owner/repo/skill").is_err());
        assert!(parse_pack_url("https://skills.sh.evil.com/p/abc").is_err());
    }
}

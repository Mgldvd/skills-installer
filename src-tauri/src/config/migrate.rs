use std::collections::HashSet;

use crate::domain::{
    ApplicationConfig, ConfigDefaults, InstallScope, Skill, SkillGroup, SkillSource, SkillTag,
    OTHER_GROUP_ID,
};
use crate::skills::{SkillUrlParser, SkillsShUrlParser};

use super::color::{deterministic_color_for_group_id, validate_and_normalize_color};
use super::disk::DiskConfig;

/// Converts any recognized on-disk shape (full curated config or bare legacy
/// `skills:` array — both parse into the same `DiskConfig`, see
/// `config::disk`) into the frontend-facing `ApplicationConfig`. Never fails:
/// unparseable individual skill URLs are skipped (logged), unknown group
/// references fall back to `other`, missing/invalid colors get a
/// deterministic palette color. A whole-file *load* should degrade
/// gracefully; only user-submitted *edits* are rejected outright by the
/// stricter `validate_*` functions.
pub fn migrate(
    mut disk: DiskConfig,
    source_path: Option<String>,
    is_embedded_default: bool,
) -> ApplicationConfig {
    let parser = SkillsShUrlParser;

    // Old releases called free-form skill tags "Packs" and some external
    // configs used a top-level `packs: [{ name, skills }]` form. Normalize
    // both representations once into stable tag ids and skill references.
    let mut used_tag_ids = HashSet::new();
    let mut tags: Vec<SkillTag> = disk
        .tags
        .iter()
        .map(|tag| {
            used_tag_ids.insert(tag.id.clone());
            SkillTag {
                id: tag.id.clone(),
                name: tag.name.trim().to_string(),
                color: validate_and_normalize_color(&tag.color)
                    .unwrap_or_else(|_| deterministic_color_for_group_id(&tag.id)),
                order: tag.order,
                enabled: tag.enabled,
            }
        })
        .collect();

    for pack in &disk.packs {
        let requested = pack
            .id
            .clone()
            .unwrap_or_else(|| crate::domain::slugify(&pack.name));
        let id = if let Some(existing) = tags
            .iter()
            .find(|t| t.id == requested || t.name.eq_ignore_ascii_case(pack.name.trim()))
        {
            existing.id.clone()
        } else {
            let id = dedupe_id(&requested, &mut used_tag_ids);
            tags.push(SkillTag {
                id: id.clone(),
                name: pack.name.trim().to_string(),
                color: pack
                    .color
                    .as_deref()
                    .and_then(|c| validate_and_normalize_color(c).ok())
                    .unwrap_or_else(|| deterministic_color_for_group_id(&id)),
                order: (tags.len() as i32 + 1) * 10,
                enabled: true,
            });
            id
        };
        for skill_id in &pack.skills {
            if let Some(skill) = disk
                .skills
                .iter_mut()
                .find(|s| s.id.as_deref() == Some(skill_id))
            {
                if !skill.tags.contains(&id) {
                    skill.tags.push(id.clone());
                }
            }
        }
    }

    // Free-form legacy values become definitions; references are rewritten
    // case-insensitively to the stable id.
    for skill in &mut disk.skills {
        for reference in &mut skill.tags {
            if let Some(existing) = tags
                .iter()
                .find(|t| t.id == *reference || t.name.eq_ignore_ascii_case(reference))
            {
                *reference = existing.id.clone();
            } else {
                let name = reference.trim().to_string();
                let id = dedupe_id(&name, &mut used_tag_ids);
                tags.push(SkillTag {
                    id: id.clone(),
                    name,
                    color: deterministic_color_for_group_id(&id),
                    order: (tags.len() as i32 + 1) * 10,
                    enabled: true,
                });
                *reference = id;
            }
        }
        skill.tags.sort();
        skill.tags.dedup();
    }
    tags.sort_by_key(|tag| tag.order);

    let mut groups: Vec<SkillGroup> = disk
        .groups
        .iter()
        .map(|g| {
            let color = g
                .color
                .as_deref()
                .and_then(|c| validate_and_normalize_color(c).ok())
                .unwrap_or_else(|| deterministic_color_for_group_id(&g.id));
            SkillGroup {
                id: g.id.clone(),
                name: g.name.clone(),
                color,
                order: g.order,
                enabled: g.enabled,
            }
        })
        .collect();

    let known_group_ids: HashSet<String> = groups.iter().map(|g| g.id.clone()).collect();
    let mut used_skill_ids: HashSet<String> = HashSet::new();
    let mut skills = Vec::new();

    for disk_skill in &disk.skills {
        let parsed = match parser.parse(&disk_skill.url) {
            Ok(parsed) => parsed,
            Err(err) => {
                tracing::warn!(
                    url = %disk_skill.url,
                    error = %err,
                    "skipping configured skill with an unparseable URL"
                );
                continue;
            }
        };

        let group_id = match &disk_skill.group {
            Some(g) if known_group_ids.contains(g) => g.clone(),
            _ => OTHER_GROUP_ID.to_string(),
        };

        let base_id = disk_skill
            .id
            .clone()
            .unwrap_or_else(|| parsed.skill_name.clone());
        let id = dedupe_id(&base_id, &mut used_skill_ids);

        skills.push(Skill {
            id,
            name: disk_skill.name.clone(),
            display_name: disk_skill.name.clone(),
            description: disk_skill.description.clone().unwrap_or_default(),
            source: SkillSource::Remote,
            repository: parsed.repository,
            repository_url: parsed.repository_url,
            skill_name: parsed.skill_name,
            skills_url: parsed.canonical_url,
            group_id,
            tags: disk_skill.tags.clone(),
            preselected: disk_skill.preselected,
            local: false,
            installed: false,
            installed_agents: Vec::new(),
            enabled: disk_skill.enabled,
        });
    }

    if !groups.iter().any(|g| g.id == OTHER_GROUP_ID) {
        groups.push(SkillGroup {
            id: OTHER_GROUP_ID.to_string(),
            name: "Other".to_string(),
            color: deterministic_color_for_group_id(OTHER_GROUP_ID),
            order: i32::MAX,
            enabled: true,
        });
    }
    groups.sort_by_key(|g| g.order);

    let defaults = ConfigDefaults {
        agent: disk.defaults.agent.clone(),
        copy: disk.defaults.copy,
        scope: match disk.defaults.scope.as_deref() {
            Some("global") => InstallScope::Global,
            _ => InstallScope::Project,
        },
    };

    ApplicationConfig {
        version: disk.version.max(1),
        defaults,
        groups,
        tags,
        skills,
        local_skill_tags: disk.local_skill_tags,
        unrecognized_skills: Vec::new(),
        source_path,
        is_embedded_default,
        // `ConfigurationService` operates purely on file paths and has no
        // notion of "project root" — `SkillsService::load_state` (the only
        // caller that matters for the GUI/CLI) overwrites this with the
        // real value once migration returns.
        project_root: String::new(),
    }
}

/// Also reused by `app::skills_service` when creating new skills/groups from
/// the GUI/CLI, so ids are deduplicated the same way whether they came from
/// a migrated legacy file or a brand-new "Add Skill" submission.
pub(crate) fn dedupe_id(base: &str, used: &mut HashSet<String>) -> String {
    let base = crate::domain::slugify(base);
    let mut candidate = base.clone();
    let mut counter = 2;
    while used.contains(&candidate) {
        candidate = format!("{base}-{counter}");
        counter += 1;
    }
    used.insert(candidate.clone());
    candidate
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::disk::{DiskDefaults, DiskGroup, DiskPack, DiskSkill};

    fn full_disk_config() -> DiskConfig {
        DiskConfig {
            version: 1,
            defaults: DiskDefaults {
                agent: Some("universal".into()),
                copy: true,
                scope: Some("project".into()),
            },
            groups: vec![DiskGroup {
                id: "testing".into(),
                name: "Testing".into(),
                color: Some("#6B82D9".into()),
                order: 20,
                enabled: true,
            }],
            tags: vec![],
            packs: vec![],
            local_skill_tags: Default::default(),
            skills: vec![DiskSkill {
                id: Some("triage".into()),
                name: "Issue Triage".into(),
                url: "https://www.skills.sh/mattpocock/skills/triage".into(),
                description: Some("Helps triage development issues.".into()),
                group: Some("testing".into()),
                tags: vec!["issues".into(), "workflow".into()],
                preselected: true,
                enabled: true,
            }],
        }
    }

    #[test]
    fn migrates_full_curated_config() {
        let config = migrate(full_disk_config(), Some("/tmp/skills.yaml".into()), false);
        assert_eq!(config.skills.len(), 1);
        let skill = &config.skills[0];
        assert_eq!(skill.id, "triage");
        assert_eq!(skill.group_id, "testing");
        assert_eq!(skill.repository, "skills");
        assert_eq!(skill.repository_url, "https://github.com/mattpocock/skills");
        assert!(skill.preselected);
        // "testing" (configured) + synthetic "other"
        assert_eq!(config.groups.len(), 2);
    }

    #[test]
    fn migrates_bare_legacy_config_with_defaults() {
        let disk = DiskConfig {
            version: 1,
            defaults: DiskDefaults::default(),
            groups: vec![],
            tags: vec![],
            packs: vec![],
            local_skill_tags: Default::default(),
            skills: vec![DiskSkill {
                id: None,
                name: "Example".into(),
                url: "https://www.skills.sh/mattpocock/skills/triage".into(),
                description: None,
                group: None,
                tags: vec![],
                preselected: true,
                enabled: true,
            }],
        };

        let config = migrate(disk, None, true);
        assert_eq!(config.skills.len(), 1);
        let skill = &config.skills[0];
        assert_eq!(skill.group_id, "other");
        assert!(skill.preselected);
        assert!(skill.enabled);
        assert_eq!(
            skill.id, "triage",
            "id should default to the URL-derived skill name"
        );
        assert!(config.is_embedded_default);
        assert!(config.groups.iter().any(|g| g.id == "other"));
    }

    #[test]
    fn migrates_legacy_pack_membership_and_color_idempotently() {
        let mut disk = full_disk_config();
        disk.skills[0].tags.clear();
        disk.packs.push(DiskPack {
            id: None,
            name: "Recommended".into(),
            color: Some("#e75480".into()),
            skills: vec!["triage".into()],
        });
        let first = migrate(disk, None, true);
        assert_eq!(first.tags.len(), 1);
        assert_eq!(first.tags[0].id, "recommended");
        assert_eq!(first.tags[0].color, "#E75480");
        assert_eq!(first.skills[0].tags, vec!["recommended"]);

        let canonical = DiskConfig {
            version: first.version,
            defaults: DiskDefaults::default(),
            groups: vec![],
            packs: vec![],
            local_skill_tags: Default::default(),
            tags: first
                .tags
                .iter()
                .map(|t| crate::config::disk::DiskTag {
                    id: t.id.clone(),
                    name: t.name.clone(),
                    color: t.color.clone(),
                    order: t.order,
                    enabled: t.enabled,
                })
                .collect(),
            skills: vec![DiskSkill {
                id: Some("triage".into()),
                name: "Issue Triage".into(),
                url: "https://www.skills.sh/mattpocock/skills/triage".into(),
                description: None,
                group: None,
                tags: vec!["recommended".into()],
                preselected: false,
                enabled: true,
            }],
        };
        let second = migrate(canonical, None, true);
        assert_eq!(second.tags, first.tags);
        assert_eq!(second.skills[0].tags, vec!["recommended"]);
    }

    #[test]
    fn deduplicates_colliding_skill_ids() {
        let mut disk = full_disk_config();
        disk.skills.push(DiskSkill {
            id: Some("triage".into()),
            name: "Another Triage".into(),
            url: "https://www.skills.sh/pbakaus/impeccable/impeccable".into(),
            description: None,
            group: None,
            tags: vec![],
            preselected: false,
            enabled: true,
        });

        let config = migrate(disk, None, false);
        let ids: Vec<&str> = config.skills.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids, vec!["triage", "triage-2"]);
    }

    #[test]
    fn skips_skills_with_unparseable_urls_without_failing() {
        let mut disk = full_disk_config();
        disk.skills.push(DiskSkill {
            id: Some("bad".into()),
            name: "Bad".into(),
            url: "https://example.com/not-a-skill-url".into(),
            description: None,
            group: None,
            tags: vec![],
            preselected: false,
            enabled: true,
        });

        let config = migrate(disk, None, false);
        assert_eq!(
            config.skills.len(),
            1,
            "the unparseable entry should be skipped, not fail the load"
        );
    }

    #[test]
    fn falls_back_to_deterministic_color_for_invalid_disk_color() {
        let mut disk = full_disk_config();
        disk.groups[0].color = Some("not-a-color".into());
        let config = migrate(disk, None, false);
        let testing_group = config.groups.iter().find(|g| g.id == "testing").unwrap();
        assert_eq!(
            testing_group.color,
            deterministic_color_for_group_id("testing")
        );
    }

    #[test]
    fn unknown_group_reference_falls_back_to_other() {
        let mut disk = full_disk_config();
        disk.skills[0].group = Some("nonexistent".into());
        let config = migrate(disk, None, false);
        assert_eq!(config.skills[0].group_id, "other");
    }
}

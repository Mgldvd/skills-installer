use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::config::migrate::dedupe_id;
use crate::config::{validate_and_normalize_color, ConfigurationService};
use crate::domain::{
    slugify, validate_skill_id, ApplicationConfig, LocalUpdatesReport, Skill, SkillGroup,
    SkillSource, SkillTag, UnrecognizedSkill, OTHER_GROUP_ID,
};
use crate::error::AppError;
use crate::preferences::PreferencesService;
use crate::skills::catalog_git;
use crate::skills::pack_import::{PackPreview, PackSkillPreview};
use crate::skills::signature;
use crate::skills::{
    discover_installed_skills, discover_installed_skills_globally, discover_skills_in_directory,
    SkillUrlParser, SkillsShUrlParser,
};

#[derive(Debug, Clone, Default)]
pub struct NewSkillInput {
    pub url: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub group_id: String,
    pub tags: Vec<String>,
    pub preselected: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackImportResult {
    pub tag: SkillTag,
    pub added: Vec<Skill>,
    pub skipped: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SkillUpdateInput {
    pub url: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub group_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub preselected: Option<bool>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct NewGroupInput {
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Default)]
pub struct GroupUpdateInput {
    pub name: Option<String>,
    pub color: Option<String>,
    pub order: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct TagUpdateInput {
    pub name: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone)]
pub enum DeleteGroupStrategy {
    RequireEmpty,
    MoveToOther,
    MoveTo(String),
}

/// The application-level view of "all skills" (configured + locally
/// discovered, merged) and every skill/group mutation. `ConfigurationService`
/// only knows about the persisted (remote) skill/group list; this is the
/// layer that adds local discovery on top for display, and is what both the
/// Tauri command layer and the CLI actually call.
pub struct SkillsService {
    config_service: ConfigurationService,
    project_root: PathBuf,
    preferences: PreferencesService,
    /// Overrides the real `$HOME` for Global-scope discovery — `None` (the
    /// real constructor's only option) resolves `$HOME` at call time; tests
    /// use `with_home_override` to point it at an isolated tempdir instead,
    /// so a test run never picks up whatever the developer's own machine
    /// actually has installed globally.
    home_override: Option<PathBuf>,
    /// Scope `load_state()` (the zero-arg convenience wrapper) discovers
    /// against — see `load_state_for`. Defaults to `Project`; the GUI always
    /// calls `load_state_for` with an explicit scope instead once it knows
    /// the user's current selection (see `commands::installation::refresh`),
    /// so this default only matters for the app's very first load and for
    /// the bare CLI, neither of which has a scope switch of its own yet.
    scope: crate::domain::InstallScope,
}

impl SkillsService {
    pub fn import_pack(
        &self,
        preview: PackPreview,
        pack_name: String,
        color: String,
        group_id: String,
    ) -> Result<PackImportResult, AppError> {
        let mut config = self.config_service.load()?;
        if !config.groups.iter().any(|group| group.id == group_id) {
            return Err(AppError::Validation(format!(
                "unknown group \"{group_id}\""
            )));
        }
        let name = validate_tag_name(&config.tags, &pack_name, None)?;
        let color = validate_and_normalize_color(&color)?;
        let mut used_tag_ids = config.tags.iter().map(|tag| tag.id.clone()).collect();
        let tag = SkillTag {
            id: dedupe_id(&slugify(&name), &mut used_tag_ids),
            name,
            color,
            order: config.tags.iter().map(|tag| tag.order).max().unwrap_or(0) + 10,
            enabled: true,
        };
        let mut used_skill_ids = config.skills.iter().map(|skill| skill.id.clone()).collect();
        let existing_names = config
            .skills
            .iter()
            .map(|skill| skill.skill_name.clone())
            .collect::<HashSet<_>>();
        let mut added = Vec::new();
        let mut skipped = Vec::new();
        for PackSkillPreview { name, description } in preview.skills {
            if existing_names.contains(&name) {
                skipped.push(name);
                continue;
            }
            let id = dedupe_id(&slugify(&name), &mut used_skill_ids);
            let skill = Skill {
                id,
                name: name.clone(),
                display_name: name.clone(),
                description,
                source: SkillSource::Remote,
                repository: preview.suggested_name.clone(),
                repository_url: preview.canonical_url.clone(),
                skill_name: name.clone(),
                skills_url: format!("{}#{}", preview.canonical_url, name),
                group_id: group_id.clone(),
                tags: vec![tag.id.clone()],
                preselected: false,
                local: false,
                installed: false,
                installed_agents: Vec::new(),
                enabled: true,
            };
            config.skills.push(skill.clone());
            added.push(skill);
        }
        if added.is_empty() {
            return Err(AppError::Validation(
                "every Skill in this pack is already in the catalog".into(),
            ));
        }
        config.tags.push(tag.clone());
        self.config_service.save(&config)?;
        Ok(PackImportResult {
            tag,
            added,
            skipped,
        })
    }

    pub fn configured_state(&self) -> Result<ApplicationConfig, AppError> {
        self.config_service.load()
    }
    pub fn replace_config(&self, mut incoming: ApplicationConfig) -> Result<(), AppError> {
        let current = self.config_service.load()?;
        incoming.source_path = current.source_path;
        incoming.is_embedded_default = current.is_embedded_default;
        incoming.project_root = self.project_root.display().to_string();
        self.config_service.save(&incoming)?;
        Ok(())
    }
    pub fn create_tag(&self, name: String, color: String) -> Result<SkillTag, AppError> {
        let mut config = self.config_service.load()?;
        let name = validate_tag_name(&config.tags, &name, None)?;
        let color = validate_and_normalize_color(&color)?;
        let mut used: HashSet<String> = config.tags.iter().map(|t| t.id.clone()).collect();
        let id = dedupe_id(&slugify(&name), &mut used);
        let order = config.tags.iter().map(|t| t.order).max().unwrap_or(0) + 10;
        let tag = SkillTag {
            id,
            name,
            color,
            order,
            enabled: true,
        };
        config.tags.push(tag.clone());
        self.config_service.save(&config)?;
        Ok(tag)
    }

    pub fn update_tag(&self, tag_id: &str, input: TagUpdateInput) -> Result<SkillTag, AppError> {
        let mut config = self.config_service.load()?;
        let index = config
            .tags
            .iter()
            .position(|t| t.id == tag_id)
            .ok_or_else(|| AppError::NotFound(format!("tag \"{tag_id}\" not found")))?;
        if let Some(name) = input.name {
            config.tags[index].name = validate_tag_name(&config.tags, &name, Some(tag_id))?;
        }
        if let Some(color) = input.color {
            config.tags[index].color = validate_and_normalize_color(&color)?;
        }
        let tag = config.tags[index].clone();
        self.config_service.save(&config)?;
        Ok(tag)
    }

    pub fn delete_tag(&self, tag_id: &str) -> Result<(), AppError> {
        let mut config = self.config_service.load()?;
        let before = config.tags.len();
        config.tags.retain(|t| t.id != tag_id);
        if before == config.tags.len() {
            return Err(AppError::NotFound(format!("tag \"{tag_id}\" not found")));
        }
        for skill in &mut config.skills {
            skill.tags.retain(|id| id != tag_id);
        }
        for tag_ids in config.local_skill_tags.values_mut() {
            tag_ids.retain(|id| id != tag_id);
        }
        self.config_service.save(&config)?;
        Ok(())
    }

    pub fn reorder_tags(&self, tag_ids: Vec<String>) -> Result<Vec<SkillTag>, AppError> {
        let mut config = self.config_service.load()?;
        let existing: HashSet<&str> = config.tags.iter().map(|tag| tag.id.as_str()).collect();
        let requested: HashSet<&str> = tag_ids.iter().map(String::as_str).collect();
        if tag_ids.len() != config.tags.len()
            || requested.len() != tag_ids.len()
            || requested != existing
        {
            return Err(AppError::Validation(
                "tag order must contain every configured tag exactly once".to_string(),
            ));
        }
        for (index, tag_id) in tag_ids.iter().enumerate() {
            if let Some(tag) = config.tags.iter_mut().find(|tag| &tag.id == tag_id) {
                tag.order = ((index + 1) * 10) as i32;
            }
        }
        config.tags.sort_by_key(|tag| tag.order);
        let tags = config.tags.clone();
        self.config_service.save(&config)?;
        Ok(tags)
    }

    pub fn set_tag_assignment(
        &self,
        skill_id: &str,
        tag_id: &str,
        assigned: bool,
    ) -> Result<Skill, AppError> {
        let mut config = self.config_service.load()?;
        if !config.tags.iter().any(|t| t.id == tag_id) {
            return Err(AppError::NotFound(format!("tag \"{tag_id}\" not found")));
        }
        let result = if let Some(skill) = config.skills.iter_mut().find(|s| s.id == skill_id) {
            skill.tags.retain(|id| id != tag_id);
            if assigned {
                skill.tags.push(tag_id.to_string());
            }
            skill.tags.sort();
            skill.tags.dedup();
            skill.clone()
        } else {
            let mut local = self.find_local_catalog_skill(skill_id)?;
            let tag_ids = config
                .local_skill_tags
                .entry(skill_id.to_string())
                .or_default();
            tag_ids.retain(|id| id != tag_id);
            if assigned {
                tag_ids.push(tag_id.to_string());
            }
            tag_ids.sort();
            tag_ids.dedup();
            if tag_ids.is_empty() {
                config.local_skill_tags.remove(skill_id);
            }
            local.tags = config
                .local_skill_tags
                .get(skill_id)
                .cloned()
                .unwrap_or_default();
            local
        };
        self.config_service.save(&config)?;
        Ok(result)
    }

    /// Looks up `skill_id` in the user's Local Skill Source catalog (or the
    /// `~/.control/skill` fallback) — for a skill `load_state_for` merged
    /// into the in-memory `config.skills` view (see its `local_skills` merge
    /// loop) but that was never itself persisted into `config.skills`, so a
    /// fresh `config_service.load()` inside a mutation doesn't have it.
    /// Shared by `set_tag_assignment` and `update_skill`, the two mutations
    /// reachable against such a skill (assigning it a Pack from its card,
    /// and editing it from its details dialog).
    ///
    /// `discover_skills_in_directory` marks everything it finds `installed:
    /// true` — correct when it's scanning a real install destination, but
    /// this is the Local Skill Source *catalog*, never one (see its own doc
    /// comment). Neither caller here re-derives real per-agent install
    /// status the way `load_state_for` does, so this resets both fields to
    /// "not installed" rather than hand back a stale, misleading `true`
    /// paired with an empty `installed_agents` list — the caller's mutation
    /// (a Pack toggle, an edit) doesn't itself install anything, and the
    /// next `load_state_for` refresh fills in the real answer.
    fn find_local_catalog_skill(&self, skill_id: &str) -> Result<Skill, AppError> {
        let source = self
            .preferences
            .load()?
            .local_source_path
            .map(|path| expand_user_path(&path))
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|home| PathBuf::from(home).join(".control/skill"))
            });
        let mut skill = source
            .as_deref()
            .map(discover_skills_in_directory)
            .unwrap_or_default()
            .into_iter()
            .find(|skill| skill.id == skill_id)
            .ok_or_else(|| AppError::NotFound(format!("skill \"{skill_id}\" not found")))?;
        skill.installed = false;
        skill.installed_agents = Vec::new();
        Ok(skill)
    }

    pub fn new(
        config_service: ConfigurationService,
        project_root: PathBuf,
        preferences: PreferencesService,
    ) -> Self {
        Self {
            config_service,
            project_root,
            preferences,
            home_override: None,
            scope: crate::domain::InstallScope::Project,
        }
    }

    /// Points Global-scope discovery at an isolated directory instead of
    /// the real `$HOME` — see `home_override`. Not `#[cfg(test)]`: the
    /// separate `tests/skill_lifecycle.rs` integration binary links this
    /// crate normally (not under `cfg(test)`), so a test-gated method here
    /// would be invisible to it, same reasoning as `PreferencesService::with_path`.
    pub fn with_home_override(mut self, home: PathBuf) -> Self {
        self.home_override = Some(home);
        self
    }

    /// Sets the scope `load_state()` discovers against — see `scope`.
    pub fn with_scope(mut self, scope: crate::domain::InstallScope) -> Self {
        self.scope = scope;
        self
    }

    /// Configured (remote) skills plus locally discovered skills, merged:
    /// a configured skill whose `skill_name` matches something found on
    /// disk is marked `installed`; a locally discovered skill is only
    /// listed separately when there is no matching configured entry
    /// (otherwise it would show up as two cards for the same thing).
    pub fn load_state(&self) -> Result<ApplicationConfig, AppError> {
        self.load_state_for(&self.project_root, self.scope)
    }

    /// Same as `load_state`, but with an explicit scope override instead of
    /// this service's stored default — used when the caller (the GUI, via
    /// `commands::installation::refresh`) knows the user's current scope
    /// but has no explicit project-folder override to go with it (an empty
    /// selection falls back to this service's own launch-time `project_root`
    /// either way).
    pub fn load_state_with_scope(
        &self,
        scope: crate::domain::InstallScope,
    ) -> Result<ApplicationConfig, AppError> {
        self.load_state_for(&self.project_root, scope)
    }

    /// Same as `load_state`, but scans `project_root` for installed Skills
    /// instead of the app's launch-time directory, and discovers against
    /// exactly one scope — `Project` scans only `project_root`'s agent
    /// directories, `Global` scans only the user's home directory, never
    /// both — the app has a single active scope, not a per-agent mix. The
    /// GUI calls this with whatever
    /// folder/scope the user currently has selected in the header — the
    /// "Installed" badge must track that selection, not the folder the app
    /// happened to start in, or picking an empty folder would still show
    /// Skills as installed from wherever the app launched.
    pub fn load_state_for(
        &self,
        project_root: &Path,
        scope: crate::domain::InstallScope,
    ) -> Result<ApplicationConfig, AppError> {
        let mut config = self.config_service.load()?;
        let installed_skills = match scope {
            crate::domain::InstallScope::Project => discover_installed_skills(project_root),
            crate::domain::InstallScope::Global => {
                // No resolvable home (override unset and `$HOME` missing)
                // just means no Global results — best effort, same as
                // everywhere else `discover_installed_skills_globally` gets
                // called.
                let home = self
                    .home_override
                    .clone()
                    .or_else(|| std::env::var_os("HOME").map(PathBuf::from));
                home.map(|home| discover_installed_skills_globally(&home))
                    .unwrap_or_default()
            }
        };
        let installed_agents: HashMap<String, Vec<String>> = installed_skills
            .iter()
            .map(|skill| (skill.skill_name.clone(), skill.installed_agents.clone()))
            .collect();
        let local_source = self.resolve_local_source()?;
        let local_skills = local_source
            .as_deref()
            .map(discover_skills_in_directory)
            .unwrap_or_default();

        let configured_skill_names: HashSet<String> =
            config.skills.iter().map(|s| s.skill_name.clone()).collect();

        for skill in config.skills.iter_mut() {
            if let Some(agents) = installed_agents.get(&skill.skill_name) {
                skill.installed = true;
                skill.installed_agents = agents.clone();
            }
        }

        for mut local in local_skills {
            local.installed_agents = installed_agents
                .get(&local.skill_name)
                .cloned()
                .unwrap_or_default();
            local.installed = !local.installed_agents.is_empty();
            if !configured_skill_names.contains(&local.skill_name) {
                local.tags = config
                    .local_skill_tags
                    .get(&local.id)
                    .cloned()
                    .unwrap_or_default();
                config.skills.push(local);
            }
        }

        // Recomputed from `config.skills` *after* the local-catalog merge
        // above, not from `configured_skill_names` — a skill the merge just
        // pushed in from the Local Skill Source catalog counts as known too,
        // otherwise it would show up both as a card and as "unrecognized".
        let known_skill_names: HashSet<&str> = config
            .skills
            .iter()
            .map(|s| s.skill_name.as_str())
            .collect();
        let mut unrecognized_skills: Vec<UnrecognizedSkill> = Vec::new();
        for skill in installed_skills {
            if known_skill_names.contains(skill.skill_name.as_str()) {
                continue;
            }
            // `discover_installed_skills` only ever produces `Local` skills
            // (it walks real directories, never the remote catalog), so this
            // always matches — the `if let` is just defensive, not a case
            // this data flow can actually hit.
            let SkillSource::Local { path } = &skill.source else {
                continue;
            };
            // `path` is the SKILL.md file itself; the "copy into catalog"
            // action needs the skill's directory, one level up.
            let dir_path = Path::new(path)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());
            unrecognized_skills.push(UnrecognizedSkill {
                skill_name: skill.skill_name,
                display_name: skill.display_name,
                path: dir_path,
            });
        }
        unrecognized_skills.sort_by_key(|skill| skill.display_name.to_lowercase());
        config.unrecognized_skills = unrecognized_skills;

        config.skills.sort_by(|a, b| {
            a.display_name
                .to_lowercase()
                .cmp(&b.display_name.to_lowercase())
        });
        config.groups.sort_by_key(|g| g.order);
        config.project_root = project_root.display().to_string();

        Ok(config)
    }

    /// The user's configured Local Skill Source catalog directory
    /// (Preferences → "Local Skill Source"), falling back to
    /// `~/.control/skill` when unset. Shared by `load_state_for` (the
    /// catalog listing) and `check_local_updates` (`.signature` lookup).
    fn resolve_local_source(&self) -> Result<Option<PathBuf>, AppError> {
        Ok(self
            .preferences
            .load()?
            .local_source_path
            .map(|path| expand_user_path(&path))
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|home| PathBuf::from(home).join(".control/skill"))
            }))
    }

    /// Copies an "unrecognized" skill's on-disk directory (`source_dir`, as
    /// reported in `ApplicationConfig::unrecognized_skills[].path`) into the
    /// user's Local Skill Source catalog directory — a real, independent
    /// copy, never a symlink, so the two directories are free to drift apart
    /// afterward like any other catalog entry. The next `load_state_for`
    /// call picks it up as a normal catalog Skill and drops it from
    /// `unrecognized_skills`, since `discover_skills_in_directory` will find
    /// it there too.
    pub fn copy_unrecognized_skill_into_catalog(&self, source_dir: &Path) -> Result<(), AppError> {
        if !source_dir.is_dir() {
            return Err(AppError::Validation(format!(
                "\"{}\" is not a directory",
                source_dir.display()
            )));
        }
        let dir_name = source_dir.file_name().ok_or_else(|| {
            AppError::Validation(format!(
                "\"{}\" has no directory name",
                source_dir.display()
            ))
        })?;
        let catalog_dir = self.resolve_local_source()?.ok_or_else(|| {
            AppError::Validation(
                "no Local Skill Source directory configured — set one in Preferences".into(),
            )
        })?;
        let destination = catalog_dir.join(dir_name);
        if destination.exists() {
            return Err(AppError::Validation(format!(
                "\"{}\" already exists in your catalog",
                dir_name.to_string_lossy()
            )));
        }
        copy_dir_recursive(source_dir, &destination)
    }

    /// Same resolution as `refresh`'s `project_path`: an explicit,
    /// non-blank path wins; otherwise falls back to this service's own
    /// `project_root`.
    pub fn check_local_updates_for(
        &self,
        project_root: Option<&str>,
    ) -> Result<LocalUpdatesReport, AppError> {
        let project_root = project_root
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| self.project_root.clone());
        self.check_local_updates(&project_root)
    }

    /// Compares each installed Local skill's `.signature` file (copied
    /// alongside its content by the Skills CLI at install time, one per
    /// skill directory) against the same file in the Local Skill Source
    /// catalog it was installed from. A mismatch means the catalog copy has
    /// changed since install — the skill is out of date.
    ///
    /// Before comparing, catalog skills are (re)signed via
    /// `skills::signature::sign_folder` — this is also, incidentally, what
    /// makes sure every catalog skill *is* signed at all, without depending
    /// on the user separately running the standalone `sign-folder` script
    /// (not guaranteed to be installed) against their catalog. Which
    /// skills need that depends on whether the catalog is a git repo (see
    /// `skills::catalog_git`):
    /// - Versioned: only skills `git status` reports as changed, plus any
    ///   still missing a `.signature` outright — cheap enough to run on
    ///   every check, or even automatically when a project opens.
    /// - Not versioned: there's no cheap way to know what changed, so
    ///   every catalog skill is re-signed, same as before git-awareness
    ///   existed. `catalog_versioned: Some(false)` in the result tells the
    ///   GUI to suggest `git init`-ing the catalog instead of relying on
    ///   this full-rehash fallback forever (`None` instead means there's no
    ///   catalog directory there at all to suggest that for).
    ///
    /// `catalog_dirty_skill_names` (only ever populated when versioned) is
    /// read again after signing, so it also covers catalog skills that
    /// were previously clean but had no `.signature` yet — the GUI uses it
    /// to suggest a commit.
    pub fn check_local_updates(&self, project_root: &Path) -> Result<LocalUpdatesReport, AppError> {
        let Some(local_source) = self.resolve_local_source()? else {
            return Ok(LocalUpdatesReport::default());
        };
        let installed_dir = project_root.join(".agents").join("skills");

        let Ok(entries) = std::fs::read_dir(&local_source) else {
            return Ok(LocalUpdatesReport::default());
        };

        let versioned = catalog_git::is_versioned(&local_source);
        let dirty_before: std::collections::HashSet<String> = if versioned {
            catalog_git::dirty_skill_names(&local_source)
                .into_iter()
                .collect()
        } else {
            std::collections::HashSet::new()
        };

        let mut outdated = Vec::new();
        for entry in entries.flatten() {
            let source_dir = entry.path();
            if !source_dir.is_dir() {
                continue;
            }
            let Some(dir_name) = source_dir.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let has_signature = source_dir.join(".signature").is_file();
            let needs_signing = !versioned || !has_signature || dirty_before.contains(dir_name);
            if needs_signing {
                // Best-effort: an unreadable/unsignable catalog folder (bad
                // permissions, a broken symlink inside it) shouldn't blank
                // out the check for every other skill — it just falls
                // through to comparing whatever `.signature` (if any) is
                // already there.
                let _ = signature::sign_folder(&source_dir);
            }

            let installed_signature =
                std::fs::read(installed_dir.join(dir_name).join(".signature"));
            let source_signature = std::fs::read(source_dir.join(".signature"));
            // Missing on either side (not installed, or signing above still
            // couldn't produce a signature) means there's nothing to
            // compare — never flagged.
            let (Ok(installed_signature), Ok(source_signature)) =
                (installed_signature, source_signature)
            else {
                continue;
            };
            if installed_signature != source_signature {
                outdated.push(slugify(dir_name));
            }
        }

        let catalog_dirty_skill_names = if versioned {
            catalog_git::dirty_skill_names(&local_source)
        } else {
            Vec::new()
        };

        Ok(LocalUpdatesReport {
            outdated_skill_ids: outdated,
            catalog_versioned: Some(versioned),
            catalog_dirty_skill_names,
        })
    }

    pub fn add_skill(&self, input: NewSkillInput) -> Result<Skill, AppError> {
        let parsed = SkillsShUrlParser.parse(&input.url)?;
        let mut config = self.config_service.load()?;

        if config
            .skills
            .iter()
            .any(|s| s.skills_url == parsed.canonical_url)
        {
            return Err(AppError::Validation(format!(
                "a skill for \"{}\" is already configured",
                parsed.canonical_url
            )));
        }
        if !config.groups.iter().any(|g| g.id == input.group_id) {
            return Err(AppError::Validation(format!(
                "unknown group \"{}\"",
                input.group_id
            )));
        }

        let mut used_ids: HashSet<String> = config.skills.iter().map(|s| s.id.clone()).collect();
        let id = dedupe_id(&parsed.skill_name, &mut used_ids);
        let display_name = input
            .display_name
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| parsed.skill_name.clone());

        let skill = Skill {
            id,
            name: display_name.clone(),
            display_name,
            description: input.description.unwrap_or_default(),
            source: SkillSource::Remote,
            repository: parsed.repository,
            repository_url: parsed.repository_url,
            skill_name: parsed.skill_name,
            skills_url: parsed.canonical_url,
            group_id: input.group_id,
            tags: input.tags,
            preselected: input.preselected,
            local: false,
            installed: false,
            installed_agents: Vec::new(),
            enabled: input.enabled,
        };

        config.skills.push(skill.clone());
        self.config_service.save(&config)?;
        Ok(skill)
    }

    pub fn update_skill(&self, skill_id: &str, input: SkillUpdateInput) -> Result<Skill, AppError> {
        validate_skill_id(skill_id)?;
        let mut config = self.config_service.load()?;
        let index = config.skills.iter().position(|s| s.id == skill_id);

        // A skill discovered live from the Local Skill Source catalog (see
        // `load_state_for`'s `local_skills` merge) but never persisted into
        // `config.skills` itself — the Edit dialog still opens for it and
        // sends the same full payload, but Packs (`local_skill_tags`) are
        // the only field such a skill has anywhere to persist to; the rest
        // (name, description, preselected, enabled…) come live from its own
        // SKILL.md and are dropped, same as `set_tag_assignment`'s fallback.
        let Some(index) = index else {
            let mut local = self.find_local_catalog_skill(skill_id)?;
            if let Some(tags) = input.tags {
                let tag_ids = config
                    .local_skill_tags
                    .entry(skill_id.to_string())
                    .or_default();
                *tag_ids = tags;
                tag_ids.sort();
                tag_ids.dedup();
                if tag_ids.is_empty() {
                    config.local_skill_tags.remove(skill_id);
                }
            }
            local.tags = config
                .local_skill_tags
                .get(skill_id)
                .cloned()
                .unwrap_or_default();
            self.config_service.save(&config)?;
            return Ok(local);
        };

        if let Some(url) = &input.url {
            let parsed = SkillsShUrlParser.parse(url)?;
            config.skills[index].repository = parsed.repository;
            config.skills[index].repository_url = parsed.repository_url;
            config.skills[index].skill_name = parsed.skill_name;
            config.skills[index].skills_url = parsed.canonical_url;
        }
        if let Some(name) = input.display_name {
            let trimmed = name.trim().to_string();
            if !trimmed.is_empty() {
                config.skills[index].display_name = trimmed.clone();
                config.skills[index].name = trimmed;
            }
        }
        if let Some(description) = input.description {
            config.skills[index].description = description;
        }
        if let Some(group_id) = input.group_id {
            if !config.groups.iter().any(|g| g.id == group_id) {
                return Err(AppError::Validation(format!(
                    "unknown group \"{group_id}\""
                )));
            }
            config.skills[index].group_id = group_id;
        }
        if let Some(tags) = input.tags {
            config.skills[index].tags = tags;
        }
        if let Some(preselected) = input.preselected {
            config.skills[index].preselected = preselected;
        }
        if let Some(enabled) = input.enabled {
            config.skills[index].enabled = enabled;
        }

        let updated = config.skills[index].clone();
        self.config_service.save(&config)?;
        Ok(updated)
    }

    /// "Remove from configuration" only — never touches anything already
    /// installed on disk. Uninstalling is a completely separate operation
    /// (`InstallationService`/`Installer::remove`).
    pub fn delete_skill(&self, skill_id: &str) -> Result<(), AppError> {
        let mut config = self.config_service.load()?;
        let before = config.skills.len();
        config.skills.retain(|s| s.id != skill_id);
        if config.skills.len() == before {
            return Err(AppError::NotFound(format!(
                "skill \"{skill_id}\" not found"
            )));
        }
        self.config_service.save(&config)?;
        Ok(())
    }

    pub fn create_group(&self, input: NewGroupInput) -> Result<SkillGroup, AppError> {
        let mut config = self.config_service.load()?;
        let color = validate_and_normalize_color(&input.color)?;
        let name = input.name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::Validation(
                "group name must not be empty".to_string(),
            ));
        }

        let mut used_ids: HashSet<String> = config.groups.iter().map(|g| g.id.clone()).collect();
        let id = dedupe_id(&slugify(&name), &mut used_ids);
        // Exclude the synthetic "Other" group (order = i32::MAX, always
        // sorts last) from this calculation — including it would both
        // overflow the `+ 10` and place every new group after "Other".
        let order = config
            .groups
            .iter()
            .filter(|g| g.id != OTHER_GROUP_ID)
            .map(|g| g.order)
            .max()
            .unwrap_or(0)
            + 10;

        let group = SkillGroup {
            id,
            name,
            color,
            order,
            enabled: true,
        };

        config.groups.push(group.clone());
        self.config_service.save(&config)?;
        Ok(group)
    }

    pub fn update_group(
        &self,
        group_id: &str,
        input: GroupUpdateInput,
    ) -> Result<SkillGroup, AppError> {
        let mut config = self.config_service.load()?;
        let index = config
            .groups
            .iter()
            .position(|g| g.id == group_id)
            .ok_or_else(|| AppError::NotFound(format!("group \"{group_id}\" not found")))?;

        if let Some(name) = input.name {
            let trimmed = name.trim().to_string();
            if trimmed.is_empty() {
                return Err(AppError::Validation(
                    "group name must not be empty".to_string(),
                ));
            }
            config.groups[index].name = trimmed;
        }
        if let Some(color) = input.color {
            config.groups[index].color = validate_and_normalize_color(&color)?;
        }
        if let Some(order) = input.order {
            config.groups[index].order = order;
        }
        if let Some(enabled) = input.enabled {
            config.groups[index].enabled = enabled;
        }

        let updated = config.groups[index].clone();
        self.config_service.save(&config)?;
        Ok(updated)
    }

    /// An empty group can always be deleted outright. A non-empty group
    /// requires an explicit `DeleteGroupStrategy` — skills are never
    /// silently deleted along with their group.
    pub fn delete_group(
        &self,
        group_id: &str,
        strategy: DeleteGroupStrategy,
    ) -> Result<(), AppError> {
        if group_id == OTHER_GROUP_ID {
            return Err(AppError::Validation(
                "the \"Other\" group cannot be deleted".to_string(),
            ));
        }

        let mut config = self.config_service.load()?;
        if !config.groups.iter().any(|g| g.id == group_id) {
            return Err(AppError::NotFound(format!(
                "group \"{group_id}\" not found"
            )));
        }

        let affected: Vec<usize> = config
            .skills
            .iter()
            .enumerate()
            .filter(|(_, s)| s.group_id == group_id)
            .map(|(i, _)| i)
            .collect();

        if !affected.is_empty() {
            match strategy {
                DeleteGroupStrategy::RequireEmpty => {
                    return Err(AppError::Validation(format!(
                        "group \"{group_id}\" still has {} skill(s); choose a strategy to move them first",
                        affected.len()
                    )));
                }
                DeleteGroupStrategy::MoveToOther => {
                    for i in affected {
                        config.skills[i].group_id = OTHER_GROUP_ID.to_string();
                    }
                }
                DeleteGroupStrategy::MoveTo(target) => {
                    if target == group_id {
                        return Err(AppError::Validation(
                            "cannot move skills into the group being deleted".to_string(),
                        ));
                    }
                    if !config.groups.iter().any(|g| g.id == target) {
                        return Err(AppError::Validation(format!(
                            "unknown target group \"{target}\""
                        )));
                    }
                    for i in affected {
                        config.skills[i].group_id = target.clone();
                    }
                }
            }
        }

        config.groups.retain(|g| g.id != group_id);
        self.config_service.save(&config)?;
        Ok(())
    }
}

fn validate_tag_name(
    tags: &[SkillTag],
    raw: &str,
    except_id: Option<&str>,
) -> Result<String, AppError> {
    let name = raw.trim().to_string();
    if name.is_empty() || name.chars().count() > 64 {
        return Err(AppError::Validation(
            "tag name must be between 1 and 64 characters".into(),
        ));
    }
    if tags
        .iter()
        .any(|t| Some(t.id.as_str()) != except_id && t.name.eq_ignore_ascii_case(&name))
    {
        return Err(AppError::Validation(format!(
            "a tag named \"{name}\" already exists"
        )));
    }
    Ok(name)
}

fn expand_user_path(path: &str) -> PathBuf {
    if path == "~" {
        return std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(path));
    }
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(path)
}

/// Recursively copies every file and subdirectory from `src` into `dst`
/// (created if missing) — a real, file-by-file copy, not a symlink, so the
/// destination is independent of the source from that point on. Symlinks
/// inside `src` are skipped rather than followed or recreated: not expected
/// in a hand-authored or CLI-installed skill directory, and safer to ignore
/// than to risk copying outside `src` or creating a broken link.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), AppError> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else if file_type.is_file() {
            std::fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service_with_curated_config(tmp: &std::path::Path) -> SkillsService {
        std::fs::write(
            tmp.join("skills.yaml"),
            r##"
version: 1
groups:
  - id: testing
    name: Testing
    color: "#6B82D9"
    order: 20
    enabled: true
skills:
  - id: triage
    name: Issue Triage
    url: https://www.skills.sh/mattpocock/skills/triage
    group: testing
    preselected: true
    enabled: true
"##,
        )
        .unwrap();
        let config_service = ConfigurationService::new(None, tmp.to_path_buf());
        let preferences = PreferencesService::with_path(tmp.join("preferences.json"));
        preferences
            .save(&crate::domain::UiPreferences {
                local_source_path: Some(tmp.join(".agents/skills").display().to_string()),
                ..Default::default()
            })
            .unwrap();
        // A distinct, always-empty subdirectory — never the same path as
        // `tmp` itself, so Global-scope discovery never overlaps whatever
        // project-scope fixtures a test writes directly under `tmp`.
        SkillsService::new(config_service, tmp.to_path_buf(), preferences)
            .with_home_override(tmp.join("home"))
    }

    #[test]
    fn add_skill_persists_and_round_trips() {
        let tmp = tempfile::tempdir().unwrap();
        let service = service_with_curated_config(tmp.path());

        let skill = service
            .add_skill(NewSkillInput {
                url: "https://www.skills.sh/pbakaus/impeccable/impeccable".to_string(),
                display_name: Some("Impeccable Polish".to_string()),
                description: None,
                group_id: "testing".to_string(),
                tags: vec![],
                preselected: false,
                enabled: true,
            })
            .unwrap();

        assert_eq!(skill.display_name, "Impeccable Polish");
        assert_eq!(skill.skill_name, "impeccable");

        let state = service.load_state().unwrap();
        assert!(state.skills.iter().any(|s| s.id == skill.id));
    }

    #[test]
    fn tag_crud_assignment_and_deletion_are_canonical() {
        let tmp = tempfile::tempdir().unwrap();
        let service = service_with_curated_config(tmp.path());
        let tag = service
            .create_tag(" Recommended ".into(), "#e75480".into())
            .unwrap();
        assert_eq!(tag.id, "recommended");
        assert_eq!(tag.color, "#E75480");
        let assigned = service.set_tag_assignment("triage", &tag.id, true).unwrap();
        assert_eq!(assigned.tags, vec!["recommended"]);
        let assigned_again = service.set_tag_assignment("triage", &tag.id, true).unwrap();
        assert_eq!(
            assigned_again.tags,
            vec!["recommended"],
            "duplicate assignments are normalized"
        );
        let renamed = service
            .update_tag(
                &tag.id,
                TagUpdateInput {
                    name: Some("Essentials".into()),
                    color: Some("#56a37b".into()),
                },
            )
            .unwrap();
        assert_eq!(renamed.id, tag.id, "renaming preserves the stable id");
        assert_eq!(renamed.color, "#56A37B");
        service.delete_tag(&tag.id).unwrap();
        let state = service.load_state().unwrap();
        assert!(state.tags.is_empty());
        assert!(state
            .skills
            .iter()
            .find(|s| s.id == "triage")
            .unwrap()
            .tags
            .is_empty());
    }

    #[test]
    fn tags_reject_duplicate_names_and_unsafe_colors() {
        let tmp = tempfile::tempdir().unwrap();
        let service = service_with_curated_config(tmp.path());
        service
            .create_tag("Recommended".into(), "#E75480".into())
            .unwrap();
        assert!(service
            .create_tag("recommended".into(), "#808089".into())
            .is_err());
        assert!(service
            .create_tag("Unsafe".into(), "url(evil)".into())
            .is_err());
    }

    #[test]
    fn assigns_tags_to_discovered_local_skills_and_persists_them() {
        let tmp = tempfile::tempdir().unwrap();
        let local_dir = tmp.path().join(".agents/skills/agent-extension-pi-creator");
        std::fs::create_dir_all(&local_dir).unwrap();
        std::fs::write(
            local_dir.join("SKILL.md"),
            "---\nname: agent-extension-pi-creator\n---\n",
        )
        .unwrap();
        let service = service_with_curated_config(tmp.path());
        let bash = service.create_tag("Bash".into(), "#D98B45".into()).unwrap();

        let assigned = service
            .set_tag_assignment("agent-extension-pi-creator", &bash.id, true)
            .unwrap();
        assert!(assigned.local);
        assert_eq!(assigned.tags, vec!["bash"]);
        let reloaded = service.load_state().unwrap();
        assert_eq!(
            reloaded
                .skills
                .iter()
                .find(|s| s.id == "agent-extension-pi-creator")
                .unwrap()
                .tags,
            vec!["bash"]
        );

        let unassigned = service
            .set_tag_assignment("agent-extension-pi-creator", &bash.id, false)
            .unwrap();
        assert!(unassigned.tags.is_empty());
        assert!(!service
            .load_state()
            .unwrap()
            .local_skill_tags
            .contains_key("agent-extension-pi-creator"));
    }

    #[test]
    fn update_skill_assigns_tags_to_a_discovered_local_skill_not_in_config() {
        // Regression test: the Edit dialog opens for a skill merged into the
        // in-memory `load_state` view by `load_state_for`'s local-catalog
        // merge (see `assigns_tags_to_discovered_local_skills_and_persists_them`
        // above) but never persisted into `config.skills` — `update_skill`
        // used to require the skill already be in `config.skills` and threw
        // "skill not found" on Save, even though `set_tag_assignment`
        // already handled exactly this case for the card's own tag toggles.
        let tmp = tempfile::tempdir().unwrap();
        let local_dir = tmp.path().join(".agents/skills/agent-extension-pi-creator");
        std::fs::create_dir_all(&local_dir).unwrap();
        std::fs::write(
            local_dir.join("SKILL.md"),
            "---\nname: agent-extension-pi-creator\n---\n",
        )
        .unwrap();
        let service = service_with_curated_config(tmp.path());
        let bash = service.create_tag("Bash".into(), "#D98B45".into()).unwrap();

        let updated = service
            .update_skill(
                "agent-extension-pi-creator",
                SkillUpdateInput {
                    tags: Some(vec![bash.id.clone()]),
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(updated.local);
        assert_eq!(updated.tags, vec!["bash"]);
        assert_eq!(
            service.load_state().unwrap().local_skill_tags["agent-extension-pi-creator"],
            vec!["bash"]
        );
        // Regression: this skill only exists on disk in the Local Skill
        // Source *catalog* directory, not under any real install
        // destination — `discover_skills_in_directory` (which this
        // fallback uses) unconditionally marks whatever it finds
        // `installed: true`, which used to leak straight through and make
        // a never-installed catalog Skill's card show "Installed" with
        // every agent reported missing right after a Pack edit.
        assert!(!updated.installed);
        assert!(updated.installed_agents.is_empty());
    }

    #[test]
    fn skill_can_have_zero_one_or_multiple_tags() {
        let tmp = tempfile::tempdir().unwrap();
        let service = service_with_curated_config(tmp.path());
        let first = service
            .create_tag("First".into(), "#E75480".into())
            .unwrap();
        let second = service
            .create_tag("Second".into(), "#5F82C9".into())
            .unwrap();
        assert!(service.load_state().unwrap().skills[0].tags.is_empty());
        service
            .set_tag_assignment("triage", &first.id, true)
            .unwrap();
        let two = service
            .set_tag_assignment("triage", &second.id, true)
            .unwrap();
        assert_eq!(two.tags.len(), 2);
        let one = service
            .set_tag_assignment("triage", &first.id, false)
            .unwrap();
        assert_eq!(one.tags, vec![second.id.clone()]);
        let still_one = service
            .set_tag_assignment("triage", &first.id, false)
            .unwrap();
        assert_eq!(
            still_one.tags,
            vec![second.id],
            "unassignment is idempotent"
        );
    }

    #[test]
    fn add_skill_rejects_duplicate_url() {
        let tmp = tempfile::tempdir().unwrap();
        let service = service_with_curated_config(tmp.path());

        let result = service.add_skill(NewSkillInput {
            url: "https://www.skills.sh/mattpocock/skills/triage".to_string(),
            group_id: "testing".to_string(),
            enabled: true,
            ..Default::default()
        });
        assert!(result.is_err());
    }

    #[test]
    fn add_skill_rejects_unknown_group() {
        let tmp = tempfile::tempdir().unwrap();
        let service = service_with_curated_config(tmp.path());

        let result = service.add_skill(NewSkillInput {
            url: "https://www.skills.sh/pbakaus/impeccable/impeccable".to_string(),
            group_id: "does-not-exist".to_string(),
            enabled: true,
            ..Default::default()
        });
        assert!(result.is_err());
    }

    #[test]
    fn update_skill_changes_group_and_preserves_other_fields() {
        let tmp = tempfile::tempdir().unwrap();
        let service = service_with_curated_config(tmp.path());

        service
            .create_group(NewGroupInput {
                name: "Frontend".to_string(),
                color: "#E75480".to_string(),
            })
            .unwrap();

        let updated = service
            .update_skill(
                "triage",
                SkillUpdateInput {
                    group_id: Some("frontend".to_string()),
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(updated.group_id, "frontend");
        assert!(updated.preselected, "unrelated fields must be preserved");
    }

    #[test]
    fn update_skill_reparses_new_url() {
        let tmp = tempfile::tempdir().unwrap();
        let service = service_with_curated_config(tmp.path());

        let updated = service
            .update_skill(
                "triage",
                SkillUpdateInput {
                    url: Some("https://www.skills.sh/pbakaus/impeccable/impeccable".to_string()),
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(updated.skill_name, "impeccable");
        assert_eq!(updated.repository, "impeccable");
    }

    #[test]
    fn delete_skill_removes_configuration_entry_only() {
        let tmp = tempfile::tempdir().unwrap();
        let service = service_with_curated_config(tmp.path());
        service.delete_skill("triage").unwrap();
        let state = service.load_state().unwrap();
        assert!(!state.skills.iter().any(|s| s.id == "triage"));
    }

    #[test]
    fn create_group_assigns_deterministic_slug_id() {
        let tmp = tempfile::tempdir().unwrap();
        let service = service_with_curated_config(tmp.path());
        let group = service
            .create_group(NewGroupInput {
                name: "AI & ML".to_string(),
                color: "#8C6FB0".to_string(),
            })
            .unwrap();
        assert!(!group.id.is_empty());
        assert!(crate::domain::validate_group_id(&group.id).is_ok());
    }

    #[test]
    fn delete_group_with_skills_requires_strategy() {
        let tmp = tempfile::tempdir().unwrap();
        let service = service_with_curated_config(tmp.path());
        let result = service.delete_group("testing", DeleteGroupStrategy::RequireEmpty);
        assert!(result.is_err());
    }

    #[test]
    fn delete_group_move_to_other_reassigns_skills() {
        let tmp = tempfile::tempdir().unwrap();
        let service = service_with_curated_config(tmp.path());
        service
            .delete_group("testing", DeleteGroupStrategy::MoveToOther)
            .unwrap();

        let state = service.load_state().unwrap();
        let skill = state.skills.iter().find(|s| s.id == "triage").unwrap();
        assert_eq!(skill.group_id, "other");
        assert!(!state.groups.iter().any(|g| g.id == "testing"));
    }

    #[test]
    fn delete_group_move_to_specific_group_reassigns_skills() {
        let tmp = tempfile::tempdir().unwrap();
        let service = service_with_curated_config(tmp.path());
        service
            .create_group(NewGroupInput {
                name: "Frontend".to_string(),
                color: "#E75480".to_string(),
            })
            .unwrap();

        service
            .delete_group(
                "testing",
                DeleteGroupStrategy::MoveTo("frontend".to_string()),
            )
            .unwrap();

        let state = service.load_state().unwrap();
        let skill = state.skills.iter().find(|s| s.id == "triage").unwrap();
        assert_eq!(skill.group_id, "frontend");
    }

    #[test]
    fn other_group_cannot_be_deleted() {
        let tmp = tempfile::tempdir().unwrap();
        let service = service_with_curated_config(tmp.path());
        let result = service.delete_group(OTHER_GROUP_ID, DeleteGroupStrategy::MoveToOther);
        assert!(result.is_err());
    }

    #[test]
    fn load_state_merges_local_skills_without_duplicating_installed_configured_ones() {
        let tmp = tempfile::tempdir().unwrap();
        let service = service_with_curated_config(tmp.path());

        let skill_dir = tmp.path().join(".agents").join("skills").join("triage");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: triage\ndescription: local copy\n---\n",
        )
        .unwrap();

        let extra_dir = tmp.path().join(".agents").join("skills").join("only-local");
        std::fs::create_dir_all(&extra_dir).unwrap();
        std::fs::write(extra_dir.join("SKILL.md"), "---\nname: Only Local\n---\n").unwrap();

        let state = service.load_state().unwrap();
        let matching: Vec<_> = state
            .skills
            .iter()
            .filter(|s| s.skill_name == "triage")
            .collect();
        assert_eq!(
            matching.len(),
            1,
            "must not duplicate a configured skill that is now installed locally"
        );
        assert!(matching[0].installed);
        assert!(state.skills.iter().any(|s| s.display_name == "Only Local"));
    }

    #[test]
    fn load_state_for_scopes_installed_status_to_the_given_project_root() {
        let tmp = tempfile::tempdir().unwrap();
        let service = service_with_curated_config(tmp.path());

        let skill_dir = tmp.path().join(".agents").join("skills").join("triage");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: triage\ndescription: local copy\n---\n",
        )
        .unwrap();

        // The app's own launch-time project root does have `triage` installed.
        let here = service.load_state().unwrap();
        let triage_here = here
            .skills
            .iter()
            .find(|s| s.skill_name == "triage")
            .unwrap();
        assert!(triage_here.installed);

        // A different, empty folder the user just selected in the GUI must
        // not inherit that status — this is the exact bug report: picking
        // an empty destination folder still showed Skills as installed.
        let empty_folder = tempfile::tempdir().unwrap();
        let elsewhere = service
            .load_state_for(empty_folder.path(), crate::domain::InstallScope::Project)
            .unwrap();
        let triage_elsewhere = elsewhere
            .skills
            .iter()
            .find(|s| s.skill_name == "triage")
            .unwrap();
        assert!(!triage_elsewhere.installed);
        assert_eq!(
            elsewhere.project_root,
            empty_folder.path().display().to_string()
        );
    }

    #[test]
    fn load_state_reports_which_agents_a_skill_is_installed_for() {
        let tmp = tempfile::tempdir().unwrap();
        let service = service_with_curated_config(tmp.path());

        // `.agents/skills` is shared by several agents by convention, so a
        // skill placed there is reported installed for the whole group —
        // see AGENT_PROJECT_DIRS.
        let shared_dir = tmp.path().join(".agents").join("skills").join("triage");
        std::fs::create_dir_all(&shared_dir).unwrap();
        std::fs::write(
            shared_dir.join("SKILL.md"),
            "---\nname: triage\ndescription: shared copy\n---\n",
        )
        .unwrap();

        let state = service.load_state().unwrap();
        let triage = state
            .skills
            .iter()
            .find(|s| s.skill_name == "triage")
            .unwrap();
        assert!(triage.installed);
        assert_eq!(
            triage.installed_agents,
            vec![
                "codex",
                "cursor",
                "gemini-cli",
                "github-copilot",
                "openclaw",
                "opencode",
                "openhands",
                "pi",
                "universal",
                "vscode",
                "zed",
            ]
        );
    }

    #[test]
    fn load_state_for_never_mixes_project_and_global_scope_discovery() {
        // The app has a single active scope: Project-scope discovery must
        // never pick up a Global-only install, and vice versa — replaces
        // the old always-merge-both behavior this test used to assert.
        let tmp = tempfile::tempdir().unwrap();
        let service = service_with_curated_config(tmp.path());

        // Installed at project scope for Codex/Cursor/... via the shared
        // `.agents/skills` convention.
        let shared_dir = tmp.path().join(".agents").join("skills").join("triage");
        std::fs::create_dir_all(&shared_dir).unwrap();
        std::fs::write(
            shared_dir.join("SKILL.md"),
            "---\nname: triage\ndescription: project copy\n---\n",
        )
        .unwrap();

        // Also installed at Global scope, exclusively for Claude Code — the
        // Global scope's `.claude/skills` isn't shared the way the Project
        // scope's is (see AGENT_GLOBAL_DIRS).
        let global_dir = tmp
            .path()
            .join("home")
            .join(".claude")
            .join("skills")
            .join("triage");
        std::fs::create_dir_all(&global_dir).unwrap();
        std::fs::write(
            global_dir.join("SKILL.md"),
            "---\nname: triage\ndescription: global copy\n---\n",
        )
        .unwrap();

        let project_state = service
            .load_state_for(tmp.path(), crate::domain::InstallScope::Project)
            .unwrap();
        let triage_project = project_state
            .skills
            .iter()
            .find(|s| s.skill_name == "triage")
            .unwrap();
        assert!(triage_project.installed);
        assert_eq!(
            triage_project.installed_agents,
            vec![
                "codex",
                "cursor",
                "gemini-cli",
                "github-copilot",
                "openclaw",
                "opencode",
                "openhands",
                "pi",
                "universal",
                "vscode",
                "zed",
            ],
            "Project scope must not include claude-code, which is only installed globally here"
        );

        let global_state = service
            .load_state_for(tmp.path(), crate::domain::InstallScope::Global)
            .unwrap();
        let triage_global = global_state
            .skills
            .iter()
            .find(|s| s.skill_name == "triage")
            .unwrap();
        assert!(triage_global.installed);
        assert_eq!(
            triage_global.installed_agents,
            vec!["claude-code"],
            "Global scope must report only claude-code, not the project-scoped agents"
        );
    }

    #[test]
    fn load_state_scopes_installed_agents_to_the_destination_that_actually_has_the_skill() {
        let tmp = tempfile::tempdir().unwrap();
        let config_service = ConfigurationService::new(None, tmp.path().to_path_buf());
        let preferences = PreferencesService::with_path(tmp.path().join("preferences.json"));
        preferences
            .save(&crate::domain::UiPreferences {
                local_source_path: Some(tmp.path().join("catalog").display().to_string()),
                ..Default::default()
            })
            .unwrap();
        let service = SkillsService::new(config_service, tmp.path().to_path_buf(), preferences)
            .with_home_override(tmp.path().join("home"));

        // Present in the catalog (so it's discovered as a Skill at all) and
        // only actually installed under claude-code's own destination —
        // which Cursor, OpenCode, and GitHub Copilot also read by
        // convention, so it's reported installed for them too.
        let catalog_dir = tmp.path().join("catalog").join("only-claude");
        std::fs::create_dir_all(&catalog_dir).unwrap();
        std::fs::write(
            catalog_dir.join("SKILL.md"),
            "---\nname: Only Claude\ndescription: claude-code only\n---\n",
        )
        .unwrap();
        let claude_dir = tmp
            .path()
            .join(".claude")
            .join("skills")
            .join("only-claude");
        std::fs::create_dir_all(&claude_dir).unwrap();
        std::fs::write(
            claude_dir.join("SKILL.md"),
            "---\nname: Only Claude\ndescription: claude-code only\n---\n",
        )
        .unwrap();

        let state = service.load_state().unwrap();
        // `skill_name` now tracks the SKILL.md front matter's own `name:`
        // ("Only Claude") rather than the enclosing directory
        // ("only-claude") — see `discover_skills_in_directory`. The real
        // Skills CLI keeps that raw front matter value untouched in the
        // installed copy even though it slugifies the destination
        // directory, so this fixture's directory/front-matter split
        // mirrors what actually lands on disk.
        let only_claude = state
            .skills
            .iter()
            .find(|s| s.skill_name == "Only Claude")
            .unwrap();
        assert!(only_claude.installed);
        assert_eq!(
            only_claude.installed_agents,
            vec![
                "claude-code",
                "cursor",
                "github-copilot",
                "opencode",
                "vscode"
            ]
        );
    }

    #[test]
    fn check_local_updates_flags_only_installed_skills_with_a_changed_signature() {
        let tmp = tempfile::tempdir().unwrap();
        let project_root = tmp.path().join("project");
        let catalog = tmp.path().join("catalog");
        std::fs::create_dir_all(&project_root).unwrap();
        std::fs::create_dir_all(&catalog).unwrap();

        // Simulates what the real Skills CLI does on install: sign the
        // catalog copy, then carry that exact `.signature` over alongside a
        // fresh install of the same content.
        let install = |name: &str, content: &[u8]| {
            let source = catalog.join(name);
            std::fs::create_dir_all(&source).unwrap();
            std::fs::write(source.join("SKILL.md"), content).unwrap();
            signature::sign_folder(&source).unwrap();
            let installed = project_root.join(".agents/skills").join(name);
            std::fs::create_dir_all(&installed).unwrap();
            std::fs::copy(source.join(".signature"), installed.join(".signature")).unwrap();
        };

        // Changed: the catalog is edited after install, so re-signing it
        // during the check yields a different digest than what's installed.
        install("changed", b"v1");
        std::fs::write(catalog.join("changed").join("SKILL.md"), b"v2").unwrap();

        // Unchanged: catalog content untouched since install -> not flagged.
        install("unchanged", b"same");

        // In the catalog but never installed -> nothing to compare, not flagged.
        std::fs::create_dir_all(catalog.join("not-installed")).unwrap();
        std::fs::write(catalog.join("not-installed").join("SKILL.md"), b"whatever").unwrap();

        // Installed but has no .signature at all (predates this feature,
        // or wasn't installed through the Skills CLI) -> not flagged.
        std::fs::create_dir_all(project_root.join(".agents/skills/no-signature")).unwrap();
        std::fs::create_dir_all(catalog.join("no-signature")).unwrap();
        std::fs::write(catalog.join("no-signature").join("SKILL.md"), b"whatever").unwrap();

        let config_service = ConfigurationService::new(None, tmp.path().to_path_buf());
        let preferences = PreferencesService::with_path(tmp.path().join("preferences.json"));
        preferences
            .save(&crate::domain::UiPreferences {
                local_source_path: Some(catalog.display().to_string()),
                ..Default::default()
            })
            .unwrap();
        let service = SkillsService::new(config_service, project_root.clone(), preferences);

        let report = service.check_local_updates(&project_root).unwrap();
        assert_eq!(report.outdated_skill_ids, vec!["changed".to_string()]);
        assert_eq!(
            report.catalog_versioned,
            Some(false),
            "the catalog here is a plain directory, not a git repo"
        );
    }

    #[test]
    fn check_local_updates_only_resigns_dirty_skills_in_a_versioned_catalog() {
        let tmp = tempfile::tempdir().unwrap();
        let project_root = tmp.path().join("project");
        let catalog = tmp.path().join("catalog");
        std::fs::create_dir_all(&project_root).unwrap();
        std::fs::create_dir_all(&catalog).unwrap();

        let git = |args: &[&str]| {
            let status = std::process::Command::new("git")
                .arg("-C")
                .arg(&catalog)
                .args(args)
                .status()
                .unwrap();
            assert!(status.success(), "git {args:?} failed");
        };
        git(&["init", "-q"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);

        let install = |name: &str, content: &[u8]| {
            let source = catalog.join(name);
            std::fs::create_dir_all(&source).unwrap();
            std::fs::write(source.join("SKILL.md"), content).unwrap();
            signature::sign_folder(&source).unwrap();
            let installed = project_root.join(".agents/skills").join(name);
            std::fs::create_dir_all(&installed).unwrap();
            std::fs::copy(source.join(".signature"), installed.join(".signature")).unwrap();
        };
        install("edited", b"v1");
        install("untouched", b"same");
        git(&["add", "-A"]);
        git(&["commit", "-q", "-m", "install"]);

        // Edit one catalog skill without committing — the git-aware fast
        // path should catch this via `git status`, not by re-hashing the
        // untouched one too.
        std::fs::write(catalog.join("edited/SKILL.md"), b"v2").unwrap();

        let config_service = ConfigurationService::new(None, tmp.path().to_path_buf());
        let preferences = PreferencesService::with_path(tmp.path().join("preferences.json"));
        preferences
            .save(&crate::domain::UiPreferences {
                local_source_path: Some(catalog.display().to_string()),
                ..Default::default()
            })
            .unwrap();
        let service = SkillsService::new(config_service, project_root.clone(), preferences);

        let report = service.check_local_updates(&project_root).unwrap();
        assert_eq!(report.catalog_versioned, Some(true));
        assert_eq!(report.outdated_skill_ids, vec!["edited".to_string()]);
        assert_eq!(report.catalog_dirty_skill_names, vec!["edited".to_string()]);
    }

    #[test]
    fn check_local_updates_is_empty_when_no_local_source_is_configured() {
        let tmp = tempfile::tempdir().unwrap();
        let config_service = ConfigurationService::new(None, tmp.path().to_path_buf());
        let preferences = PreferencesService::with_path(tmp.path().join("preferences.json"));
        preferences
            .save(&crate::domain::UiPreferences {
                local_source_path: None,
                ..Default::default()
            })
            .unwrap();
        let service = SkillsService::new(config_service, tmp.path().to_path_buf(), preferences);

        // No $HOME-dependent fallback is exercised here since HOME may or
        // may not resolve to a real `.control/skill` directory on the
        // machine running the tests; either way, a nonexistent catalog
        // must resolve to no updates rather than an error.
        assert_eq!(
            service
                .check_local_updates(tmp.path())
                .unwrap()
                .outdated_skill_ids,
            Vec::<String>::new()
        );
    }
}

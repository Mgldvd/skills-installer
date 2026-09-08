mod config;
mod dependency;
mod group;
mod install;
mod local_catalog;
mod parsed_source;
pub mod preferences;
mod preset;
mod skill;
mod tag;
mod validate;

pub use config::{ApplicationConfig, ConfigDefaults};
pub use dependency::{DependencySource, DependencyStatus};
pub use group::{SkillGroup, OTHER_GROUP_ID};
pub use install::{
    InstallOptions, InstallProgressEvent, InstallRequest, InstallResult, InstallScope,
    OutputStream, SkillInstallOutcome, SkillInstallStatus, UninstallRequest, UninstallResult,
};
pub use local_catalog::LocalUpdatesReport;
pub use parsed_source::ParsedSkillSource;
pub use preferences::UiPreferences;
pub use preset::Preset;
pub use skill::{InstalledSkill, Skill, SkillSelection, SkillSource, UnrecognizedSkill};
pub use tag::SkillTag;
pub use validate::{
    cli_agent_id, is_supported_agent, slugify, validate_agent_id, validate_group_id,
    validate_skill_id,
};

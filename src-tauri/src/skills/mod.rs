pub mod discovery;
pub mod url_parser;

pub use discovery::extract_front_matter;
pub use discovery::{
    discover_installed_agents, discover_installed_agents_globally, discover_installed_skills,
    discover_installed_skills_globally, discover_local_skills, discover_skills_in_directory,
};
pub mod catalog_git;
pub mod pack_import;
pub mod signature;
pub use url_parser::{SkillUrlParser, SkillsShUrlParser};

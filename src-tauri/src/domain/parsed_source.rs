use serde::{Deserialize, Serialize};

/// Output of `skills::url_parser::SkillUrlParser::parse`. `canonical_url` is the
/// normalized `https://www.skills.sh/<owner>/<repository>/<skill_name>` form,
/// independent of whatever casing/host variant the user typed in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedSkillSource {
    pub canonical_url: String,
    pub owner: String,
    pub repository: String,
    pub skill_name: String,
    pub repository_url: String,
}

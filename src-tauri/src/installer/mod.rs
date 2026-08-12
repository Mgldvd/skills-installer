pub mod dependency;
#[cfg(test)]
pub mod fake;
pub mod skills_cli;
pub mod traits;

pub use dependency::{DependencyResolver, ResolvedSkillsCli};
pub use skills_cli::SkillsCliInstaller;
pub use traits::{
    InstallBatch, Installer, ListOptions, ProgressSender, RemoveRequest, UpdateRequest,
};

#[cfg(test)]
pub use fake::{FakeInstaller, FakeProcessResult, FakeProcessRunner};

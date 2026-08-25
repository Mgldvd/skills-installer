//! Test doubles for `ProcessRunner` and `Installer`. Compiled only under
//! `#[cfg(test)]` (see `installer::mod`) so they never ship in the release
//! binary, but are reachable from any other module's test code within this
//! crate — used here, by `app::InstallationService` tests, and anywhere
//! else that needs to exercise install orchestration without a real
//! network call or child process (requirement: no live installs/network
//! access from the automated test suite).

use std::collections::VecDeque;
use std::sync::Mutex;

use async_trait::async_trait;
use tokio_util::sync::CancellationToken;

use crate::domain::{
    DependencyStatus, InstallProgressEvent, InstallResult, InstalledSkill, OutputStream,
    UninstallResult,
};
use crate::error::AppError;
use crate::process::{ProcessOutcome, ProcessOutputLine, ProcessRunner, ProcessSpec};

use super::traits::{
    InstallBatch, Installer, ListOptions, ProgressSender, RemoveRequest, UpdateRequest,
};

pub struct FakeProcessResult {
    pub outcome: Result<ProcessOutcome, AppError>,
    pub output_lines: Vec<ProcessOutputLine>,
}

impl FakeProcessResult {
    pub fn success() -> Self {
        Self {
            outcome: Ok(ProcessOutcome::Completed { exit_code: 0 }),
            output_lines: Vec::new(),
        }
    }

    pub fn failure(exit_code: i32) -> Self {
        Self {
            outcome: Ok(ProcessOutcome::Completed { exit_code }),
            output_lines: Vec::new(),
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            outcome: Err(AppError::Process(message.to_string())),
            output_lines: Vec::new(),
        }
    }

    pub fn cancelled() -> Self {
        Self {
            outcome: Ok(ProcessOutcome::Cancelled),
            output_lines: Vec::new(),
        }
    }

    pub fn with_output(mut self, stream: OutputStream, line: &str) -> Self {
        self.output_lines.push(ProcessOutputLine {
            stream,
            line: line.to_string(),
        });
        self
    }
}

/// Scripted, in-order responses for successive `run()` calls. Exhausting
/// the script falls back to a bare success rather than panicking, so tests
/// that don't care about a particular call's outcome don't need to script
/// every single one.
pub struct FakeProcessRunner {
    pub calls: Mutex<Vec<ProcessSpec>>,
    script: Mutex<VecDeque<FakeProcessResult>>,
}

impl FakeProcessRunner {
    pub fn new(script: Vec<FakeProcessResult>) -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            script: Mutex::new(script.into()),
        }
    }

    pub fn call_count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }
}

#[async_trait]
impl ProcessRunner for FakeProcessRunner {
    async fn run(
        &self,
        spec: ProcessSpec,
        output_tx: tokio::sync::mpsc::UnboundedSender<ProcessOutputLine>,
        _cancel: CancellationToken,
    ) -> Result<ProcessOutcome, AppError> {
        self.calls.lock().unwrap().push(spec);
        let next = self
            .script
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(FakeProcessResult::success);
        for line in next.output_lines {
            let _ = output_tx.send(line);
        }
        next.outcome
    }
}

/// A whole-`Installer` test double, for exercising `app::InstallationService`
/// (and CLI/command-layer code that depends on `Arc<dyn Installer>`) without
/// going through `SkillsCliInstaller` at all.
pub struct FakeInstaller {
    dependency_status: DependencyStatus,
    install_result: Mutex<Option<Result<InstallResult, AppError>>>,
    pub install_calls: Mutex<Vec<InstallBatch>>,
    remove_result: Mutex<Option<Result<UninstallResult, AppError>>>,
    pub remove_calls: Mutex<Vec<RemoveRequest>>,
}

impl FakeInstaller {
    pub fn new(dependency_status: DependencyStatus) -> Self {
        Self {
            dependency_status,
            install_result: Mutex::new(None),
            install_calls: Mutex::new(Vec::new()),
            remove_result: Mutex::new(None),
            remove_calls: Mutex::new(Vec::new()),
        }
    }

    pub fn with_install_result(self, result: Result<InstallResult, AppError>) -> Self {
        *self.install_result.lock().unwrap() = Some(result);
        self
    }

    pub fn with_remove_result(self, result: Result<UninstallResult, AppError>) -> Self {
        *self.remove_result.lock().unwrap() = Some(result);
        self
    }
}

#[async_trait]
impl Installer for FakeInstaller {
    async fn check_dependencies(&self) -> Result<DependencyStatus, AppError> {
        Ok(self.dependency_status.clone())
    }

    async fn list_installed(&self, _options: ListOptions) -> Result<Vec<InstalledSkill>, AppError> {
        Ok(Vec::new())
    }

    async fn install(
        &self,
        batch: InstallBatch,
        progress: ProgressSender,
        _cancel: CancellationToken,
    ) -> Result<InstallResult, AppError> {
        self.install_calls.lock().unwrap().push(batch.clone());
        let _ = progress.send(InstallProgressEvent::Start {
            total: batch.skills.len(),
        });

        match self.install_result.lock().unwrap().take() {
            Some(result) => result,
            None => Ok(InstallResult {
                requested: batch.skills.len(),
                installed: batch.skills.len(),
                ..Default::default()
            }),
        }
    }

    async fn remove(&self, request: RemoveRequest) -> Result<UninstallResult, AppError> {
        let requested = request.skills.len();
        self.remove_calls.lock().unwrap().push(request);
        match self.remove_result.lock().unwrap().take() {
            Some(result) => result,
            None => Ok(UninstallResult {
                requested,
                removed: requested,
                failed: 0,
                message: None,
            }),
        }
    }

    async fn update(&self, _request: UpdateRequest) -> Result<(), AppError> {
        Ok(())
    }
}

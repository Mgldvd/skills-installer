use std::path::PathBuf;
use std::process::Stdio;

use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;

use crate::domain::OutputStream;
use crate::error::AppError;

use super::ansi::strip_ansi;

/// Everything needed to start one child process. Arguments are always a
/// `Vec<String>` passed straight to `Command::args` — there is no code path
/// anywhere in this struct or its consumers that concatenates them into a
/// shell string. No `sh -c`, no `bash -c`, no `eval`.
#[derive(Debug, Clone)]
pub struct ProcessSpec {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: Vec<(String, String)>,
}

impl ProcessSpec {
    pub fn new(program: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            program: program.into(),
            args,
            cwd: None,
            env: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessOutputLine {
    pub stream: OutputStream,
    pub line: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessOutcome {
    Completed { exit_code: i32 },
    Cancelled,
}

/// The single process-execution primitive used everywhere a child process
/// needs to run. `Send + Sync` so it can live behind `Arc` and be shared
/// between the GUI (Tauri async commands) and the CLI.
#[async_trait]
pub trait ProcessRunner: Send + Sync {
    async fn run(
        &self,
        spec: ProcessSpec,
        output_tx: UnboundedSender<ProcessOutputLine>,
        cancel: CancellationToken,
    ) -> Result<ProcessOutcome, AppError>;
}

/// The real implementation, backed by `tokio::process::Command` for
/// non-blocking spawn/stream/cancel.
pub struct TokioProcessRunner;

#[async_trait]
impl ProcessRunner for TokioProcessRunner {
    async fn run(
        &self,
        spec: ProcessSpec,
        output_tx: UnboundedSender<ProcessOutputLine>,
        cancel: CancellationToken,
    ) -> Result<ProcessOutcome, AppError> {
        let mut command = tokio::process::Command::new(&spec.program);
        command.args(&spec.args);
        if let Some(cwd) = &spec.cwd {
            command.current_dir(cwd);
        }
        for (key, value) in &spec.env {
            command.env(key, value);
        }
        command.stdin(Stdio::null());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let mut child = command
            .spawn()
            .map_err(|e| AppError::Process(format!("failed to start \"{}\": {e}", spec.program)))?;

        let stdout = child
            .stdout
            .take()
            .expect("stdout was requested as piped at spawn time");
        let stderr = child
            .stderr
            .take()
            .expect("stderr was requested as piped at spawn time");

        let stdout_task = tokio::spawn(stream_lines(
            stdout,
            OutputStream::Stdout,
            output_tx.clone(),
        ));
        let stderr_task = tokio::spawn(stream_lines(stderr, OutputStream::Stderr, output_tx));

        tokio::select! {
            _ = cancel.cancelled() => {
                let _ = child.start_kill();
                let _ = child.wait().await;
                let _ = stdout_task.await;
                let _ = stderr_task.await;
                Ok(ProcessOutcome::Cancelled)
            }
            status = child.wait() => {
                let _ = stdout_task.await;
                let _ = stderr_task.await;
                let status = status.map_err(|e| {
                    AppError::Process(format!("failed to wait for \"{}\": {e}", spec.program))
                })?;
                Ok(ProcessOutcome::Completed { exit_code: status.code().unwrap_or(-1) })
            }
        }
    }
}

async fn stream_lines<R>(reader: R, stream: OutputStream, tx: UnboundedSender<ProcessOutputLine>)
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut lines = BufReader::new(reader).lines();
    while let Ok(Some(raw_line)) = lines.next_line().await {
        let clean = strip_ansi(&raw_line);
        if tx
            .send(ProcessOutputLine {
                stream,
                line: clean,
            })
            .is_err()
        {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    async fn drain(mut rx: mpsc::UnboundedReceiver<ProcessOutputLine>) -> Vec<ProcessOutputLine> {
        let mut lines = Vec::new();
        while let Some(line) = rx.recv().await {
            lines.push(line);
        }
        lines
    }

    #[tokio::test]
    async fn runs_a_real_process_and_streams_stdout() {
        let (tx, rx) = mpsc::unbounded_channel();
        let spec = ProcessSpec::new("printf", vec!["line-one\nline-two\n".to_string()]);
        let outcome = TokioProcessRunner
            .run(spec, tx, CancellationToken::new())
            .await
            .unwrap();

        assert_eq!(outcome, ProcessOutcome::Completed { exit_code: 0 });
        let lines: Vec<String> = drain(rx).await.into_iter().map(|l| l.line).collect();
        assert_eq!(lines, vec!["line-one", "line-two"]);
    }

    #[tokio::test]
    async fn captures_non_zero_exit_code() {
        let (tx, rx) = mpsc::unbounded_channel();
        let spec = ProcessSpec::new("sh", vec!["-c".to_string(), "exit 7".to_string()]);
        // Note: this test spawns `sh -c` itself only to *simulate* a
        // failing third-party tool for the purposes of exercising our exit
        // code capture — the application's own installer code never
        // constructs a shell invocation like this.
        let outcome = TokioProcessRunner
            .run(spec, tx, CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(outcome, ProcessOutcome::Completed { exit_code: 7 });
        drop(rx);
    }

    #[tokio::test]
    async fn cancellation_stops_a_long_running_process() {
        let (tx, rx) = mpsc::unbounded_channel();
        let spec = ProcessSpec::new("sleep", vec!["30".to_string()]);
        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();

        let handle =
            tokio::spawn(async move { TokioProcessRunner.run(spec, tx, cancel_clone).await });

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        cancel.cancel();

        let outcome = tokio::time::timeout(std::time::Duration::from_secs(5), handle)
            .await
            .expect("cancellation should not hang")
            .unwrap()
            .unwrap();
        assert_eq!(outcome, ProcessOutcome::Cancelled);
        drop(rx);
    }

    #[tokio::test]
    async fn reports_a_useful_error_for_a_missing_executable() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let spec = ProcessSpec::new("definitely-not-a-real-executable-xyz", vec![]);
        let result = TokioProcessRunner
            .run(spec, tx, CancellationToken::new())
            .await;
        assert!(result.is_err());
    }
}

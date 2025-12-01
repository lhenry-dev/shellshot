use portable_pty::{native_pty_system, ChildKiller, CommandBuilder, PtySize};
use std::{
    io::{self, BufReader, BufWriter, Read, Write},
    thread,
    time::Duration,
};
use termwiz::surface::Surface;
use thiserror::Error;
use tracing::{info, warn};

use crate::{
    constants::{MAX_HEIGHT, MAX_WIDTH},
    pty_executor::{
        dimension::Dimension,
        writer::{DetachableWriter, ThreadedWriter},
    },
    terminal_builder::TerminalBuilder,
};

pub mod dimension;
mod writer;

#[derive(Debug, Error)]
pub enum PtyExecutorError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Empty command provided")]
    EmptyCommand,

    #[error("Failed to open PTY: {0}")]
    PtyOpenFailed(String),

    #[error("Failed to clone PTY reader: {0}")]
    CloneReaderFailed(String),

    #[error("Failed to spawn child process: {0}")]
    SpawnChildFailed(String),

    #[error("Failed to take PTY writer: {0}")]
    TakeWriterFailed(String),

    #[error("Failed to join thread: {0}")]
    ThreadJoinFailed(String),

    #[error("Terminal builder error: {0}")]
    TerminalBuilderError(#[from] crate::terminal_builder::TerminalBuilderError),

    #[error("Command execution timed out")]
    Timeout,

    #[error("Child process panicked during execution")]
    ChildPanicked,
}

pub struct PtyIO {
    pub reader: BufReader<Box<dyn Read + Send>>,
    pub writer: DetachableWriter,
}

#[derive(Clone)]
pub struct PtyOptions {
    pub cols: Dimension,
    pub rows: Dimension,
}

pub struct PtyExecutor {}

impl PtyExecutor {
    pub fn run_command(
        pty_options: &PtyOptions,
        command: &[String],
    ) -> Result<Surface, PtyExecutorError> {
        if command.is_empty() {
            return Err(PtyExecutorError::EmptyCommand);
        }

        let cmd_name = &command[0];
        let args = &command[1..];

        info!("Executing command: {} {}", cmd_name, args.join(" "));

        let pty_system = native_pty_system();

        let pair = pty_system
            .openpty(PtySize {
                cols: pty_options.cols.to_u16(MAX_WIDTH),
                rows: pty_options.rows.to_u16(MAX_HEIGHT),
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| PtyExecutorError::PtyOpenFailed(e.to_string()))?;

        let mut cmd = CommandBuilder::new(cmd_name);
        cmd.args(args);

        if cmd.get_cwd().is_none() {
            cmd.cwd(".");
        }

        let reader = BufReader::new(
            pair.master
                .try_clone_reader()
                .map_err(|e| PtyExecutorError::CloneReaderFailed(e.to_string()))?,
        );
        let mut child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| PtyExecutorError::SpawnChildFailed(e.to_string()))?;
        let killer = child.clone_killer();

        let writer = pair
            .master
            .take_writer()
            .map_err(|e| PtyExecutorError::TakeWriterFailed(e.to_string()))?;
        let writer = ThreadedWriter::new(Box::new(BufWriter::new(writer)));
        let writer = DetachableWriter::new(Box::new(BufWriter::new(writer)));

        let pty_process = PtyIO {
            reader,
            writer: writer.clone(),
        };
        let cols = &pty_options.cols;
        let rows = &pty_options.rows;

        thread::scope(|s| -> Result<Surface, PtyExecutorError> {
            let handle = s.spawn(|| TerminalBuilder::run(pty_process, cols, rows));

            with_timeout(None, killer, s, || child.wait())??;

            writer.detach()?.flush()?;
            drop(child);
            drop(pair);

            let surface = handle
                .join()
                .map_err(|e| PtyExecutorError::ThreadJoinFailed(format!("{:?}", e)))??;

            Ok(surface)
        })
    }
}

fn with_timeout<'scope, R, F>(
    timeout: Option<Duration>,
    mut killer: Box<dyn ChildKiller + Send + Sync>,
    s: &'scope thread::Scope<'scope, '_>,
    f: F,
) -> Result<R, PtyExecutorError>
where
    F: FnOnce() -> R,
{
    if let Some(timeout) = timeout {
        let t = s.spawn(move || {
            thread::park_timeout(timeout);
            let _ = killer.kill();
            warn!("Command execution was terminated due to timeout");
        });

        let result = f();

        t.thread().unpark();
        t.join()
            .map_err(|e| PtyExecutorError::ThreadJoinFailed(format!("{:?}", e)))?;

        Ok(result)
    } else {
        Ok(f())
    }
}

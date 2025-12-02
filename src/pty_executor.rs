use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::{
    io::{self, BufReader, BufWriter, Read, Write},
    thread,
    time::Duration,
};
use termwiz::surface::Surface;
use thiserror::Error;
use tracing::info;

use crate::{
    constants::{SCREEN_MAX_HEIGHT, SCREEN_MAX_WIDTH},
    pty_executor::{
        dimension::Dimension,
        utils::with_timeout,
        writer::{DetachableWriter, ThreadedWriter},
    },
    terminal_builder::TerminalBuilder,
};

pub mod dimension;
mod utils;
pub mod writer;

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
    pub timeout: Option<Duration>,
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
                cols: pty_options.cols.to_u16(SCREEN_MAX_WIDTH),
                rows: pty_options.rows.to_u16(SCREEN_MAX_HEIGHT),
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
        let timeout = &pty_options.timeout;

        thread::scope(|s| -> Result<Surface, PtyExecutorError> {
            let handle = s.spawn(|| TerminalBuilder::run(pty_process, cols, rows));

            with_timeout(*timeout, killer, s, || child.wait())??;

            writer.detach()?.flush()?;
            drop(child);
            drop(pair);

            let surface = handle
                .join()
                .map_err(|e| PtyExecutorError::ThreadJoinFailed(format!("{e:?}")))??;

            Ok(surface)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pty_executor::dimension::Dimension;
    use std::time::Duration;

    fn default_options() -> PtyOptions {
        PtyOptions {
            cols: Dimension::Value(80),
            rows: Dimension::Value(24),
            timeout: Some(Duration::from_secs(5)),
        }
    }

    fn shell_command(cmd: &str) -> Vec<String> {
        if cfg!(windows) {
            vec!["cmd".to_string(), "/C".to_string(), cmd.to_string()]
        } else {
            vec!["sh".to_string(), "-c".to_string(), cmd.to_string()]
        }
    }

    #[test]
    fn test_run_command_basic() {
        let options = default_options();
        let command = shell_command("echo Hello World");

        let surface = PtyExecutor::run_command(&options, &command).expect("Failed to run command");

        let text = surface.screen_chars_to_string();
        println!("Captured output:\n{text}");
        assert!(text.contains("Hello World"));
    }

    #[test]
    fn test_run_command_ansi() {
        let options = default_options();

        let ansi_str = if cfg!(windows) {
            r"echo ^[[31mRed^[[0m ^[[32mGreen^[[0m ^[[1mBold^[[0m"
        } else {
            r#"echo -e "\e[31mRed\e[0m \e[32mGreen\e[0m \e[1mBold\e[0m""#
        };

        let command = shell_command(ansi_str);

        let surface =
            PtyExecutor::run_command(&options, &command).expect("Failed to run ANSI command");

        let text = surface.screen_chars_to_string();
        println!("Captured ANSI output:\n{text}");

        assert!(text.contains("Red"));
        assert!(text.contains("Green"));
        assert!(text.contains("Bold"));
    }

    #[test]
    fn test_empty_command_error() {
        let options = default_options();
        let command: Vec<String> = vec![];

        let result = PtyExecutor::run_command(&options, &command);
        assert!(matches!(result, Err(PtyExecutorError::EmptyCommand)));
    }

    #[test]
    fn test_timeout() {
        let options = PtyOptions {
            cols: Dimension::Value(80),
            rows: Dimension::Value(24),
            timeout: Some(Duration::from_millis(500)),
        };

        let command = if cfg!(windows) {
            shell_command("timeout /T 2")
        } else {
            shell_command("sleep 2")
        };

        PtyExecutor::run_command(&options, &command).unwrap();
    }
}

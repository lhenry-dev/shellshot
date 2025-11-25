use thiserror::Error;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use crate::os_command::windows::Windows as PlatformExecutor;

#[cfg(target_os = "linux")]
pub use crate::os_command::linux::Linux as PlatformExecutor;

#[derive(Debug, Error)]
pub enum ExecutorError {
    #[error("Failed to execute command: {0}")]
    CommandError(#[from] std::io::Error),
}

/// Platform-independent executor trait implemented per-OS to run shell commands.
///
/// Implementations should execute the provided `command` string on the host platform
/// and return the combined stdout/stderr (or appropriate output) as a `String` on success,
/// or an `ExecutorError` on failure.
pub trait Executor {
    /// Execute a shell command and return its output as a String.
    ///
    /// The `command` parameter is the full command to run; implementations are
    /// responsible for invoking the appropriate shell or process API for the
    /// target platform. On success return `Ok(output)` where `output` contains
    /// the command's stdout/stderr as a single `String`, otherwise return an
    /// `ExecutorError`.
    ///
    /// # Errors
    ///
    /// Returns an error if command execution fails.
    fn execute_command(command: &str) -> Result<String, ExecutorError>;
}

pub fn merge_output(stdout: &str, stderr: &str) -> String {
    [stdout, stderr]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use crate::{os_command::PlatformExecutor, Executor};

    #[test]
    fn test_execute_simple_command() {
        let command = r#"echo "test test test""#;
        let result = PlatformExecutor::execute_command(command).unwrap();

        assert!(
            result.contains("test test test"),
            "The output does not contain the expected text. Output: {result}"
        );
    }

    // #[test]
    // fn test_execute_command_accented_characters() {
    //     let command = r#"echo "&é~#[-è_çà@ôùê+=$£¤""#;
    //     let result = PlatformExecutor::execute_command(command).unwrap();
    //     assert!(
    //         result.contains("&é~#[-è_çà@ôùê+=$£¤"),
    //         "Accented characters are not decoded correctly. Output: {result}"
    //     );
    // }
}

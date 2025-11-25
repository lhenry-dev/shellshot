use std::process::Command;

use crate::os_command::{merge_output, Executor, ExecutorError};

pub struct Linux;

impl Executor for Linux {
    fn execute_command(command: &str) -> Result<String, ExecutorError> {
        let output = Command::new("sh").arg("-c").arg(command).output()?;

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let stderr_str = String::from_utf8_lossy(&output.stderr);

        Ok(merge_output(&stdout_str, &stderr_str))
    }
}

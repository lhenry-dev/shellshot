use std::process::Command;
use yore::code_pages::CP850;

use crate::os_command::{merge_output, Executor, ExecutorError};

pub struct Windows;

impl Executor for Windows {
    fn execute_command(command: &str) -> Result<String, ExecutorError> {
        let output = Command::new("cmd").arg("/C").arg(command).output()?;

        let stdout_str = CP850.decode(&output.stdout);
        let stderr_str = CP850.decode(&output.stderr);

        Ok(merge_output(&stdout_str, &stderr_str))
    }
}

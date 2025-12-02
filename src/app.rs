use std::time::Duration;

use clap::{ArgGroup, Parser};
use thiserror::Error;
use tracing::info;

use crate::{
    image_generator::{self, SaveError},
    image_renderer::{ImageRenderer, ImageRendererError},
    pty_executor::{dimension::Dimension, PtyExecutor, PtyExecutorError, PtyOptions},
    terminal_builder::TerminalBuilderError,
    window_decoration::{create_window_decoration, WindowDecorationType},
};

/// Errors that can occur in `shellshot`
#[derive(Error, Debug)]
pub enum ShellshotError {
    #[error("Failed to execute command: {0}")]
    CommandExecution(#[from] PtyExecutorError),

    #[error("Failed to build terminal from output: {0}")]
    TerminalBuild(#[from] TerminalBuilderError),

    #[error("Failed to render image: {0}")]
    ImageRender(#[from] ImageRendererError),

    #[error("Failed to save image to file: {0}")]
    Save(#[from] SaveError),
}

/// Command-line arguments for `shellshot`
#[derive(Parser, Debug)]
#[command(
    name = "shellshot",
    about = "Transform your command-line output into clean, shareable images with a single command.",
    version,
    long_about = None,
    group(ArgGroup::new("output_mode")
        .required(true)
        .args(&["output", "clipboard"])
    )
)]
pub struct Args {
    /// Command to execute
    #[arg(trailing_var_arg = true, required = true)]
    pub command: Vec<String>,

    /// Do not print anything to stdout
    #[arg(long, short = 'q')]
    pub quiet: bool,

    /// Do not draw window decorations
    #[arg(long)]
    pub no_decoration: bool,

    /// Specify decoration style
    #[arg(long, short = 'd', default_value = "classic")]
    pub decoration: WindowDecorationType,

    /// Specify output filename
    #[arg(long, short = 'o', conflicts_with = "clipboard")]
    pub output: Option<String>,

    /// Save to clipboard
    #[arg(long, conflicts_with = "output")]
    pub clipboard: bool,

    /// Final image width in pixels, or 'auto'
    #[arg(long, short = 'W', default_value = "auto")]
    pub width: Dimension,

    /// Final image height in pixels, or 'auto'
    #[arg(long, short = 'H', default_value = "auto")]
    pub height: Dimension,

    /// Timeout in seconds for command execution (0 = no timeout)
    #[arg(long, short = 't')]
    pub timeout: Option<u64>,
}

/// Main entry point for shellshot logic
///
/// # Errors
///
/// Returns an error if:
/// - Command execution fails
/// - Screen building fails
/// - Image rendering fails
/// - Saving the image fails
pub fn run_shellshot(args: Args) -> Result<(), ShellshotError> {
    info!("Starting shellshot v{}", env!("CARGO_PKG_VERSION"));

    let pty_options = PtyOptions {
        cols: args.width,
        rows: args.height,
        timeout: args.timeout.map(Duration::from_secs),
    };

    let screen = PtyExecutor::run_command(&pty_options, &args.command)?;

    let decoration = (!args.no_decoration).then_some(args.decoration);
    let window_decoration = create_window_decoration(decoration.as_ref());

    let image_data = ImageRenderer::render_image(&args.command, &screen, window_decoration)?;

    if args.clipboard {
        image_generator::save_to_clipboard(&image_data)?;
        info!("✅ Screenshot saved to clipboard");
    }

    if let Some(output) = args.output {
        image_generator::save_to_file(&image_data, &output)?;
        info!("✅ Screenshot saved to {output}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_command_with_file() {
        let base_command = vec!["echo".to_string(), "hello".to_string()];

        let command: Vec<String> = if cfg!(windows) && base_command[0] == "echo" {
            vec!["cmd".into(), "/C".into(), base_command[1..].join(" ")]
        } else {
            base_command.clone()
        };

        let args = Args {
            command,
            quiet: false,
            no_decoration: false,
            decoration: WindowDecorationType::Classic,
            output: Some("test.png".into()),
            clipboard: false,
            width: Dimension::Auto,
            height: Dimension::Auto,
            timeout: None,
        };

        let result = run_shellshot(args);
        assert!(result.is_ok());
    }
}

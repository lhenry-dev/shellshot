use clap::Parser;
use thiserror::Error;

use crate::{
    image_generator::{self, SaveError},
    image_renderer::{ImageRenderer, ImageRendererError},
    pty_executor::PtyExecutor,
    terminal_builder::TerminalBuilder,
    window_decoration::{create_window_decoration, WindowDecorationType},
};

/// Errors that can occur in `shellshot`
#[derive(Error, Debug)]
pub enum ShellshotError {
    // #[error("Failed to execute command: {0}")]
    // CommandExecution(#[from] ExecutorError),

    // #[error("Failed to build screen from output: {0}")]
    // ScreenBuild(#[from] ScreenBuilderError),
    #[error("Failed to render image: {0}")]
    ImageRender(#[from] ImageRendererError),

    #[error("Failed to save image to file: {0}")]
    Save(#[from] SaveError),
}

/// Command-line arguments for `shellshot`
#[derive(Parser, Debug)]
#[command(
    name = "shellshot",
    about = "Transform command-line output into stunning, shareable images",
    version,
    long_about = None
)]
pub struct Args {
    /// Command to execute
    #[arg(trailing_var_arg = true)]
    pub command: Vec<String>,

    /// Do not draw window decorations
    #[arg(long)]
    pub no_decoration: bool,

    /// Specify decoration style
    #[arg(long, short = 'd', default_value = "classic")]
    pub decoration: WindowDecorationType,

    /// Specify output filename (default: out.png)
    #[arg(long, short = 'f', default_value = "out.png")]
    pub filename: Option<String>,

    /// Save to clipboard
    #[arg(long)]
    pub clipboard: bool,
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
    println!("Starting shellshot v{}", env!("CARGO_PKG_VERSION"));

    let rows = 60;
    let cols = 180;

    println!("Executing command: {:?}", args.command);

    let executor = PtyExecutor::new(rows, cols).unwrap();
    let pty_process = executor.run_command(&args.command).unwrap();
    let screen = TerminalBuilder::run(pty_process, rows, cols).unwrap();
    // let output = PlatformExecutor::execute_command(&command_str)?;

    let decoration = (!args.no_decoration).then_some(args.decoration);
    let window_decoration = create_window_decoration(decoration.as_ref());

    // let screen = ScreenBuilder::from_output(&output, &command_str, window_decoration.as_ref())?;
    let image_data = ImageRenderer::render_image(&args.command, &screen, window_decoration)?;

    if args.clipboard {
        image_generator::save_to_clipboard(&image_data)?;
        println!("✅ Screenshot saved to clipboard");
    }

    let filename = args.filename.unwrap_or_else(|| "out.png".to_string());
    image_generator::save_to_file(&image_data, &filename)?;
    println!("✅ Screenshot saved to {filename}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_command_with_file() {
        let args = Args {
            command: vec!["echo".into(), "hello".into()],
            no_decoration: false,
            decoration: WindowDecorationType::Classic,
            filename: Some("test.png".into()),
            clipboard: false,
        };

        let result = run_shellshot(args);
        assert!(result.is_ok());
    }
}

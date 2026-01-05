#![crate_type = "lib"]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

mod app;
mod constants;
mod image_generator;
mod image_renderer;
mod pty_executor;
mod terminal_builder;
mod window_decoration;

pub use app::{Args, run_shellshot};
pub use window_decoration::WindowDecorationType;

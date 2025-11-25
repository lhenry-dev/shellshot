#![crate_type = "lib"]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

mod app;
mod constants;
mod image_generator;
mod image_renderer;
mod os_command;
mod screen_builder;
mod window_decoration;

pub use crate::window_decoration::WindowDecorationType;
pub use app::{run_shellshot, Args};
pub use image_generator::{save_to_clipboard, save_to_file};
pub use image_renderer::ImageRenderer;
pub use os_command::Executor;

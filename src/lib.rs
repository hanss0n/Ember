//! Shared code for the Night Light settings app and panel applet.
//!
//! The `nightlight` CLI (`~/.local/bin/nightlight`) does the DDC/CI work.
//! The GUI reads and writes the same JSON config and calls the CLI to apply it.

pub mod backend;
pub mod config;
pub mod paths;
pub mod schedule;
pub mod status;

pub use config::{Config, Mode, Monitor};
pub use schedule::STEPS;

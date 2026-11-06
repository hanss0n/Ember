//! Shared code for the Ember settings app and panel applet.
//!
//! The `ember` CLI (`~/.local/bin/ember`) does the DDC/CI work and owns the
//! schedule logic, including sunrise and sunset. The GUIs read and write the
//! same JSON config, call the CLI to apply it, and show the status it reports.

pub mod backend;
pub mod clock;
pub mod config;
pub mod paths;
pub mod status;

pub use config::{Config, Mode, Monitor, Schedule};
pub use status::Status;

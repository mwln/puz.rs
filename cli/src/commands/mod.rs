//! Subcommand implementations for the `puz` CLI.
//!
//! Each module implements one top-level command: it defines the command's
//! clap arguments and a `run` entry point. `main` dispatches to these; shared
//! presentation lives in [`crate::render`].

pub(crate) mod dump;
pub(crate) mod inspect;
pub(crate) mod parse_json;
pub(crate) mod validate;

//! gsbt - Gameserver Backup Tool.
//!
//! A Rust port of the original Go implementation. The crate is organised into
//! modules that mirror the original package layout:
//!
//! - [`config`] - YAML configuration loading, discovery and env substitution.
//! - [`connector`] - pluggable FTP/SFTP/Nitrado connectors.
//! - [`backup`] - archive creation and backup orchestration.
//! - [`log`] - structured logging with markup support.
//! - [`progress`] - progress reporting.
//! - [`cli`] - command line interface.

pub mod backup;
pub mod cli;
pub mod config;
pub mod connector;
pub mod log;
pub mod progress;
pub mod restore;

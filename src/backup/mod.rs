//! Archive creation and backup orchestration.

pub mod archive;
pub mod manager;

pub use archive::{create_archive, timestamped_filename};
pub use manager::{transfer_log_path, Manager, Stats};

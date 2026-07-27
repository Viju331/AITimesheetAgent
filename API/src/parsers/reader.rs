use std::path::{Path, PathBuf};
use crate::errors::AppResult;
use super::session_model::NormalizedSession;

/// Common contract every AI tool session reader must implement (P3-001).
pub trait SessionReader {
    fn tool_name(&self) -> &'static str;

    /// Locate candidate session sources (files or databases) under the configured tool folder.
    fn discover(&self, tool_folder: &Path) -> AppResult<Vec<PathBuf>>;

    /// Parse a single discovered source into a normalized session.
    /// Returns `Ok(None)` when the source contains no usable conversation data.
    fn read(&self, source: &Path) -> AppResult<Option<NormalizedSession>>;
}

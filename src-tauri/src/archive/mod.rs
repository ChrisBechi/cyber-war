//! Archive operations use bytes and the selected VFS only. No host paths or unpack APIs.
pub mod cli;
mod codec;
pub mod downloads;
pub mod ipc;
pub mod jobs;
#[cfg(debug_assertions)]
pub mod qa;
mod service;
#[cfg(test)]
mod tests;

use crate::{error::GameResult, vfs::domain};
use serde::{Deserialize, Serialize};
pub use service::*;
use std::collections::BTreeMap;

/// Generic byte-level boundary for virtual package containers. No package rules.
pub(crate) fn encode_entries(
    entries: &[StoredEntry],
    format: ArchiveFormat,
) -> GameResult<Vec<u8>> {
    codec::encode(entries, format, None, &ArchiveSafetyLimits::default())
}
pub(crate) fn decode_entries(bytes: &[u8], format: ArchiveFormat) -> GameResult<Vec<StoredEntry>> {
    codec::decode(bytes, format, None, true, &ArchiveSafetyLimits::default())
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ArchiveFormat {
    Zip,
    Tar,
    Gzip,
    Bzip2,
    Xz,
    TarGzip,
    TarBzip2,
    TarXz,
}
impl ArchiveFormat {
    pub fn mime(self) -> &'static str {
        match self {
            Self::Zip => "application/zip",
            Self::Tar => "application/x-tar",
            Self::Gzip | Self::TarGzip => "application/gzip",
            Self::Bzip2 | Self::TarBzip2 => "application/x-bzip2",
            Self::Xz | Self::TarXz => "application/x-xz",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::Zip => "Zip archive data",
            Self::Tar => "POSIX tar archive",
            Self::Gzip | Self::TarGzip => "gzip compressed data",
            Self::Bzip2 | Self::TarBzip2 => "bzip2 compressed data",
            Self::Xz | Self::TarXz => "XZ compressed data",
        }
    }
    pub fn is_stream(self) -> bool {
        matches!(self, Self::Gzip | Self::Bzip2 | Self::Xz)
    }
    pub fn stream(self) -> Self {
        match self {
            Self::TarGzip => Self::Gzip,
            Self::TarBzip2 => Self::Bzip2,
            Self::TarXz => Self::Xz,
            f => f,
        }
    }
    pub fn suffix(self) -> &'static str {
        match self {
            Self::Gzip => ".gz",
            Self::Bzip2 => ".bz2",
            Self::Xz => ".xz",
            _ => "",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ArchiveIntegrity {
    Valid,
    Partial,
    Corrupted,
    Encrypted,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveEntry {
    pub path: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub original_size: u64,
    pub compressed_size: Option<u64>,
    pub permissions: u16,
    pub modified_at: u64,
    pub owner: Option<String>,
    pub group: Option<String>,
    pub crc: Option<u32>,
    pub link_target: Option<String>,
    pub encrypted: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryMetadata {
    pub logical_size: u64,
    pub modified_at: u64,
    pub owner: String,
    pub group: String,
    pub mime: Option<String>,
    pub archive_depth: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_source: Option<String>,
}
#[derive(Debug, Clone)]
pub(crate) struct StoredEntry {
    pub info: ArchiveEntry,
    pub data: Vec<u8>,
    pub metadata: Option<EntryMetadata>,
}

#[derive(Debug, Clone)]
pub struct ArchiveSafetyLimits {
    pub max_entries: usize,
    pub max_uncompressed_size: u64,
    pub max_compression_ratio: u64,
    pub max_depth: usize,
    pub max_filename_length: usize,
    pub max_storage_size: u64,
}
impl Default for ArchiveSafetyLimits {
    fn default() -> Self {
        Self {
            max_entries: 4096,
            max_uncompressed_size: 16 * 1024 * 1024 * 1024,
            max_compression_ratio: 2000,
            max_depth: 32,
            max_filename_length: 255,
            max_storage_size: 32 * 1024 * 1024,
        }
    }
}
impl ArchiveSafetyLimits {
    pub fn path(&self, path: &str) -> GameResult<String> {
        let path = path.trim_end_matches('/');
        if path.is_empty()
            || path.starts_with('/')
            || path.len() > 4096
            || path.contains(['\\', ':'])
            || path.chars().any(char::is_control)
        {
            return Err(domain("archive: unsafe entry path"));
        }
        let parts: Vec<_> = path.split('/').collect();
        if parts.len() > self.max_depth
            || parts.iter().any(|p| {
                p.is_empty() || *p == ".." || *p == "." || p.len() > self.max_filename_length
            })
        {
            return Err(domain("archive: unsafe entry path or path limit exceeded"));
        }
        Ok(path.into())
    }
    pub(crate) fn entries(&self, entries: &[StoredEntry], compressed: u64) -> GameResult<()> {
        if entries.len() > self.max_entries {
            return Err(domain("archive: entry count limit exceeded"));
        }
        let mut paths = BTreeMap::new();
        let mut total = 0u64;
        let mut storage = 0u64;
        for e in entries {
            let p = self.path(&e.info.path)?;
            if paths.insert(p, e.info.kind.as_str()).is_some() {
                return Err(domain("archive: duplicate filename"));
            }
            if !matches!(e.info.kind.as_str(), "file" | "directory" | "symlink") {
                return Err(domain("archive: unsupported entry type"));
            }
            if let Some(link) = &e.info.link_target {
                // Conservative relative links only; never follow them while extracting.
                self.path(link)?;
            }
            total = total
                .checked_add(e.info.original_size)
                .ok_or_else(|| domain("archive: size overflow"))?;
            storage = storage
                .checked_add(e.data.len() as u64)
                .ok_or_else(|| domain("archive: size overflow"))?;
            if total > self.max_uncompressed_size || storage > self.max_storage_size {
                return Err(domain("archive: uncompressed size limit exceeded"));
            }
        }
        // Sparse logical bytes are bounded separately; ratio limits apply to actual decompression.
        if storage > compressed.max(1).saturating_mul(self.max_compression_ratio) {
            return Err(domain("archive: compression ratio limit exceeded"));
        }
        for e in entries {
            let mut p = e.info.path.as_str();
            while let Some((parent, _)) = p.rsplit_once('/') {
                if paths.get(parent).is_some_and(|kind| *kind != "directory") {
                    return Err(domain("archive: entry descends through a file or symlink"));
                }
                p = parent;
            }
        }
        Ok(())
    }
}

pub fn detect(bytes: &[u8]) -> Option<ArchiveFormat> {
    if bytes.starts_with(b"PK\x03\x04") || bytes.starts_with(b"PK\x05\x06") {
        Some(ArchiveFormat::Zip)
    } else if bytes.starts_with(&[0x1f, 0x8b]) {
        Some(ArchiveFormat::Gzip)
    } else if bytes.starts_with(b"BZh") {
        Some(ArchiveFormat::Bzip2)
    } else if bytes.starts_with(b"\xfd7zXZ\0") {
        Some(ArchiveFormat::Xz)
    } else if bytes.get(257..262) == Some(b"ustar")
        || (bytes.len() >= 1024 && bytes.iter().all(|b| *b == 0))
    {
        Some(ArchiveFormat::Tar)
    } else {
        None
    }
}

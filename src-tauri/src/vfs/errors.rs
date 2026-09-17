//! Stable virtual errno vocabulary. Never contains host I/O errors.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Errno {
    NotFound,
    Exists,
    NotDirectory,
    IsDirectory,
    Access,
    NotEmpty,
    Loop,
    Invalid,
    NameTooLong,
    ReadOnly,
    NoSpace,
    NotPermitted,
    BadDescriptor,
    CrossDevice,
}
impl Errno {
    pub fn code(self) -> &'static str {
        match self {
            Self::NotFound => "ENOENT",
            Self::Exists => "EEXIST",
            Self::NotDirectory => "ENOTDIR",
            Self::IsDirectory => "EISDIR",
            Self::Access => "EACCES",
            Self::NotEmpty => "ENOTEMPTY",
            Self::Loop => "ELOOP",
            Self::Invalid => "EINVAL",
            Self::NameTooLong => "ENAMETOOLONG",
            Self::ReadOnly => "EROFS",
            Self::NoSpace => "ENOSPC",
            Self::NotPermitted => "EPERM",
            Self::BadDescriptor => "EBADF",
            Self::CrossDevice => "EXDEV",
        }
    }
}
impl std::fmt::Display for Errno {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::NotFound => "No such file or directory",
            Self::Exists => "File exists",
            Self::NotDirectory => "Not a directory",
            Self::IsDirectory => "Is a directory",
            Self::Access => "Permission denied",
            Self::NotEmpty => "Directory not empty",
            Self::Loop => "Too many levels of symbolic links",
            Self::Invalid => "Invalid argument",
            Self::NameTooLong => "File name too long",
            Self::ReadOnly => "Read-only virtual filesystem",
            Self::NoSpace => "No space left on virtual device",
            Self::NotPermitted => "Operation not permitted",
            Self::BadDescriptor => "Bad virtual file descriptor",
            Self::CrossDevice => "Invalid cross-device link",
        })
    }
}
pub fn error(errno: Errno) -> crate::error::GameError {
    crate::error::GameError::Vfs(errno)
}

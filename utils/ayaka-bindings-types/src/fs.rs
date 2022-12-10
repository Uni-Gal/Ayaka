use std::io::SeekFrom;

use serde::{Deserialize, Serialize};
use vfs::*;

/// Type of a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum FileType {
    /// A file.
    File,
    /// A directory.
    Dir,
}

impl From<VfsFileType> for FileType {
    fn from(value: VfsFileType) -> Self {
        match value {
            VfsFileType::File => Self::File,
            VfsFileType::Directory => Self::Dir,
        }
    }
}

impl From<FileType> for VfsFileType {
    fn from(value: FileType) -> Self {
        match value {
            FileType::File => Self::File,
            FileType::Dir => Self::Directory,
        }
    }
}

/// File metadata.
#[derive(Debug, Deserialize, Serialize)]
pub struct FileMetadata {
    /// Type of file.
    pub file_type: FileType,
    /// Length of the file.
    pub len: u64,
}

impl From<VfsMetadata> for FileMetadata {
    fn from(value: VfsMetadata) -> Self {
        Self {
            file_type: value.file_type.into(),
            len: value.len,
        }
    }
}

impl From<FileMetadata> for VfsMetadata {
    fn from(value: FileMetadata) -> Self {
        Self {
            file_type: value.file_type.into(),
            len: value.len,
        }
    }
}

/// A [`SeekFrom`] type which implements `serde`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum FileSeekFrom {
    /// Seek from start.
    Start(u64),
    /// Seek from end.
    End(i64),
    /// Seek from current position.
    Current(i64),
}

impl From<SeekFrom> for FileSeekFrom {
    fn from(value: SeekFrom) -> Self {
        match value {
            SeekFrom::Start(p) => Self::Start(p),
            SeekFrom::End(p) => Self::End(p),
            SeekFrom::Current(p) => Self::Current(p),
        }
    }
}

impl From<FileSeekFrom> for SeekFrom {
    fn from(value: FileSeekFrom) -> Self {
        match value {
            FileSeekFrom::Start(p) => Self::Start(p),
            FileSeekFrom::End(p) => Self::End(p),
            FileSeekFrom::Current(p) => Self::Current(p),
        }
    }
}

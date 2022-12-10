use std::io::SeekFrom;

use serde::{Deserialize, Serialize};
use vfs::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum FileType {
    File,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct FileMetadata {
    pub file_type: FileType,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum FileSeekFrom {
    Start(u64),
    End(i64),
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

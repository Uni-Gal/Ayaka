use frfs::FRFS;
use std::{
    fmt::Debug,
    io::{Result, Write},
    path::Path,
};
use vfs::{error::VfsErrorKind, *};

#[derive(Debug)]
pub struct PackFS {
    fs: FRFS,
}

impl PackFS {
    pub fn new(p: impl AsRef<Path> + Copy) -> Result<Self> {
        Ok(Self { fs: FRFS::new(p)? })
    }
}

impl FileSystem for PackFS {
    fn read_dir(&self, path: &str) -> VfsResult<Box<dyn Iterator<Item = String>>> {
        Ok(Box::new(
            self.fs
                .read_dir(path)?
                .into_iter()
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path().to_string_lossy().into_owned()),
        ))
    }

    fn create_dir(&self, _path: &str) -> VfsResult<()> {
        Err(VfsErrorKind::NotSupported.into())
    }

    fn open_file(&self, path: &str) -> VfsResult<Box<dyn SeekAndRead>> {
        Ok(Box::new(self.fs.open(path)?))
    }

    fn create_file(&self, _path: &str) -> VfsResult<Box<dyn Write>> {
        Err(VfsErrorKind::NotSupported.into())
    }

    fn append_file(&self, _path: &str) -> VfsResult<Box<dyn Write>> {
        Err(VfsErrorKind::NotSupported.into())
    }

    fn metadata(&self, path: &str) -> VfsResult<VfsMetadata> {
        Ok(self.fs.metadata(path).map(|meta| VfsMetadata {
            file_type: if meta.is_file() {
                VfsFileType::File
            } else {
                VfsFileType::Directory
            },
            len: meta.len(),
        })?)
    }

    fn exists(&self, path: &str) -> VfsResult<bool> {
        match self.metadata(path) {
            Ok(_) => Ok(true),
            Err(err) => match err.kind() {
                VfsErrorKind::FileNotFound => Ok(false),
                _ => Err(err),
            },
        }
    }

    fn remove_file(&self, _path: &str) -> VfsResult<()> {
        Err(VfsErrorKind::NotSupported.into())
    }

    fn remove_dir(&self, _path: &str) -> VfsResult<()> {
        Err(VfsErrorKind::NotSupported.into())
    }
}

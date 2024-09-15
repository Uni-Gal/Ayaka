use crate::import;
use ayaka_bindings_types::{FileMetadata, FileSeekFrom};
use std::io::{Read, Result, Seek, SeekFrom};
use vfs::{error::VfsErrorKind, *};

#[import("fs")]
extern "C" {
    fn __read_dir(path: &str) -> Option<Vec<String>>;
    fn __metadata(path: &str) -> Option<FileMetadata>;
    fn __exists(path: &str) -> bool;

    fn __open_file(path: &str) -> Option<u64>;
    fn __close_file(fd: u64);

    fn __file_read(fd: u64, ptr: i32, len: i32) -> Option<usize>;
    fn __file_seek(fd: u64, pos: FileSeekFrom) -> Option<u64>;
}

#[derive(Debug, Default)]
pub struct HostFS;

impl FileSystem for HostFS {
    fn read_dir(&self, path: &str) -> VfsResult<Box<dyn Iterator<Item = String> + Send>> {
        match __read_dir(path) {
            Some(paths) => Ok(Box::new(paths.into_iter())),
            None => Err(VfsErrorKind::FileNotFound.into()),
        }
    }

    fn create_dir(&self, _path: &str) -> VfsResult<()> {
        Err(VfsErrorKind::NotSupported.into())
    }

    fn open_file(&self, path: &str) -> VfsResult<Box<dyn SeekAndRead + Send>> {
        match __open_file(path) {
            Some(fd) => Ok(Box::new(HostFile { fd })),
            None => Err(VfsErrorKind::FileNotFound.into()),
        }
    }

    fn create_file(&self, _path: &str) -> VfsResult<Box<dyn SeekAndWrite + Send>> {
        Err(VfsErrorKind::NotSupported.into())
    }

    fn append_file(&self, _path: &str) -> VfsResult<Box<dyn SeekAndWrite + Send>> {
        Err(VfsErrorKind::NotSupported.into())
    }

    fn metadata(&self, path: &str) -> VfsResult<VfsMetadata> {
        match __metadata(path) {
            Some(meta) => Ok(meta.into()),
            None => Err(VfsErrorKind::FileNotFound.into()),
        }
    }

    fn exists(&self, path: &str) -> VfsResult<bool> {
        Ok(__exists(path))
    }

    fn remove_file(&self, _path: &str) -> VfsResult<()> {
        Err(VfsErrorKind::NotSupported.into())
    }

    fn remove_dir(&self, _path: &str) -> VfsResult<()> {
        Err(VfsErrorKind::NotSupported.into())
    }
}

#[derive(Debug)]
struct HostFile {
    fd: u64,
}

impl Read for HostFile {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        __file_read(self.fd, buf.as_mut_ptr() as _, buf.len() as _)
            .ok_or_else(|| std::io::ErrorKind::NotFound.into())
    }
}

impl Seek for HostFile {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        __file_seek(self.fd, pos.into()).ok_or_else(|| std::io::ErrorKind::NotFound.into())
    }
}

impl Drop for HostFile {
    fn drop(&mut self) {
        __close_file(self.fd)
    }
}

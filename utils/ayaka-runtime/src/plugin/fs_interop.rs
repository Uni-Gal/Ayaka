use anyhow::Result;
use ayaka_bindings_types::{FileMetadata, FileSeekFrom};
use ayaka_plugin::{Linker, LinkerHandle, RawModule};
use slab::Slab;
use std::{
    collections::HashMap,
    io::SeekFrom,
    sync::{Arc, Mutex},
};
use vfs::*;

#[derive(Default)]
struct FDMap {
    map: Slab<Box<dyn SeekAndRead + Send>>,
}

impl FDMap {
    pub fn open(&mut self, file: Box<dyn SeekAndRead + Send>) -> u64 {
        self.map.insert(file) as u64
    }

    pub fn close(&mut self, fd: u64) {
        self.map.remove(fd as usize);
    }

    pub fn read(&mut self, fd: u64, buf: &mut [u8]) -> std::io::Result<usize> {
        self.map
            .get_mut(fd as usize)
            .ok_or(std::io::ErrorKind::NotFound)?
            .read(buf)
    }

    pub fn seek(&mut self, fd: u64, pos: SeekFrom) -> std::io::Result<u64> {
        self.map
            .get_mut(fd as usize)
            .ok_or(std::io::ErrorKind::NotFound)?
            .seek(pos)
    }
}

pub fn register<M: RawModule>(store: &mut impl Linker<M>, root_path: &VfsPath) -> Result<()> {
    let p = root_path.clone();
    let read_dir_func = store.wrap(move |(path,): (String,)| {
        Ok(p.join(&path[1..])?
            .read_dir()
            .map(|iter| iter.map(|p| p.as_str().to_string()).collect::<Vec<_>>())
            .ok())
    });
    let p = root_path.clone();
    let metadata_func = store.wrap(move |(path,): (String,)| {
        Ok(p.join(&path[1..])?.metadata().map(FileMetadata::from).ok())
    });
    let p = root_path.clone();
    let exists_func = store.wrap(move |(path,): (String,)| Ok(p.join(&path[1..])?.exists()?));

    let fd_map = Arc::new(Mutex::new(FDMap::default()));
    let p = root_path.clone();
    let map = fd_map.clone();
    let open_file_func = store.wrap(move |(path,): (String,)| {
        let file = p.join(&path[1..])?.open_file();
        Ok(file.map(|file| map.lock().unwrap().open(file)).ok())
    });
    let map = fd_map.clone();
    let close_file_func = store.wrap(move |(fd,): (u64,)| {
        map.lock().unwrap().close(fd);
        Ok(())
    });
    let map = fd_map.clone();
    let file_read_func = store.wrap_with(move |mut handle, (fd, ptr, len): (u64, i32, i32)| {
        Ok(handle
            .slice_mut(ptr, len, |buf| map.lock().unwrap().read(fd, buf))
            .ok())
    });
    let map = fd_map;
    let file_seek_func = store.wrap(move |(fd, pos): (u64, FileSeekFrom)| {
        Ok(map.lock().unwrap().seek(fd, pos.into()).ok())
    });
    store.import(
        "fs",
        HashMap::from([
            ("__read_dir".to_string(), read_dir_func),
            ("__metadata".to_string(), metadata_func),
            ("__exists".to_string(), exists_func),
            ("__open_file".to_string(), open_file_func),
            ("__close_file".to_string(), close_file_func),
            ("__file_read".to_string(), file_read_func),
            ("__file_seek".to_string(), file_seek_func),
        ]),
    )?;
    Ok(())
}

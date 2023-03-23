use objc::{rc::StrongPtr, runtime::Object, *};
use stable_deref_trait::{CloneStableDeref, StableDeref};
use std::{
    ffi::{c_char, c_void, CString},
    ops::Deref,
};
use tokio::sync::watch;
use tokio_stream::{wrappers::WatchStream, Stream, StreamExt};

#[link(name = "Foundation", kind = "framework")]
#[link(name = "UIKit", kind = "framework")]
#[link(name = "UniformTypeIdentifiers", kind = "framework")]
#[link(name = "picker", kind = "static")]
extern "C" {
    fn show_browser(
        controller: *mut Object,
        extensions: *const *const c_char,
        types_len: usize,
        allow_multiple: bool,
        closure: unsafe extern "C" fn(*mut Object, *mut c_void),
        closure_data: *mut c_void,
    );
}

#[derive(Clone)]
pub struct FileHandle(StrongPtr);

impl Deref for FileHandle {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe {
            let bytes: *const c_void = msg_send![*self.0, bytes];
            let length: usize = msg_send![*self.0, length];
            std::slice::from_raw_parts(bytes as *const u8, length)
        }
    }
}

// SAFETY: Apple
unsafe impl Send for FileHandle {}
unsafe impl Sync for FileHandle {}

// SAFETY: NSData
unsafe impl StableDeref for FileHandle {}
unsafe impl CloneStableDeref for FileHandle {}

unsafe extern "C" fn pick_files_closure(data: *mut Object, closure_data: *mut c_void) {
    let sender = Box::from_raw(closure_data as *mut watch::Sender<Option<FileHandle>>);
    if !data.is_null() {
        let file_handle = FileHandle(StrongPtr::retain(data));
        sender.send(Some(file_handle)).ok();
        std::mem::forget(sender);
    }
}

pub fn pick_files(controller: *mut Object, extensions: &[&str]) -> impl Stream<Item = FileHandle> {
    let extensions = extensions
        .iter()
        .map(|s| CString::new(*s).unwrap())
        .collect::<Vec<_>>();
    let extension_ptrs = extensions.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();
    let (tx, rx) = watch::channel(None::<FileHandle>);
    unsafe {
        show_browser(
            controller,
            extension_ptrs.as_ptr(),
            extension_ptrs.len(),
            true,
            pick_files_closure,
            Box::leak(Box::new(tx)) as *mut _ as *mut c_void,
        );
    }
    WatchStream::new(rx).skip(1).filter_map(|f| f)
}

use objc::{rc::StrongPtr, runtime::Object, *};
use pin_project::pin_project;
use stable_deref_trait::{CloneStableDeref, StableDeref};
use std::{
    ffi::{c_char, c_void, CString},
    fmt::Debug,
    ops::Deref,
    pin::{pin, Pin},
    task::{Context, Poll},
};
use tokio::sync::watch;
use tokio_stream::{wrappers::WatchStream, Stream, StreamExt};

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
    ) -> *mut Object;
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

impl Debug for FileHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FileHandle")
            .field(&(*self.0 as *mut ()))
            .finish()
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

pub fn pick_files(
    controller: *mut Object,
    extensions: &[&str],
) -> impl Stream<Item = FileHandle> + Send + Sync {
    let extensions = extensions
        .iter()
        .map(|s| CString::new(*s).unwrap())
        .collect::<Vec<_>>();
    let extension_ptrs = extensions.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();
    let (tx, rx) = watch::channel(None::<FileHandle>);
    let delegate = unsafe {
        StrongPtr::retain(show_browser(
            controller,
            extension_ptrs.as_ptr(),
            extension_ptrs.len(),
            true,
            pick_files_closure,
            Box::into_raw(Box::new(tx)) as *mut _ as *mut c_void,
        ))
    };
    let s = WatchStream::new(rx).filter_map(|f| f);
    PickFilesStream { s, delegate }
}

#[pin_project]
struct PickFilesStream<S: Stream<Item = FileHandle> + Send + Sync> {
    #[pin]
    s: S,
    delegate: StrongPtr,
}

unsafe impl<S: Stream<Item = FileHandle> + Send + Sync> Send for PickFilesStream<S> {}
unsafe impl<S: Stream<Item = FileHandle> + Send + Sync> Sync for PickFilesStream<S> {}

impl<S: Stream<Item = FileHandle> + Send + Sync> Stream for PickFilesStream<S> {
    type Item = FileHandle;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().s.poll_next(cx)
    }
}

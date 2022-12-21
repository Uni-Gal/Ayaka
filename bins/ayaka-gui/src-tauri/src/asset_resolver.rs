use axum::{
    body::{Bytes, HttpBody},
    extract::Path,
    http::{header::CONTENT_TYPE, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router, Server,
};
use pin_project::pin_project;
use std::{
    io::Read,
    net::TcpListener,
    pin::Pin,
    sync::OnceLock,
    task::{Context, Poll},
};
use stream_future::{try_stream, Stream};
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Runtime,
};
use tower_http::cors::{Any, CorsLayer};
use vfs::*;

pub(crate) static ROOT_PATH: OnceLock<VfsPath> = OnceLock::new();
const BUFFER_LEN: usize = 65536;

struct VfsFile(Box<dyn SeekAndRead>);

#[allow(unsafe_code)]
unsafe impl Send for VfsFile {}
#[allow(unsafe_code)]
unsafe impl Sync for VfsFile {}

impl Read for VfsFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}

#[try_stream(Bytes)]
fn file_stream(mut file: VfsFile, length: usize) -> Result<(), axum::Error> {
    let length = length.min(BUFFER_LEN);
    loop {
        let mut buffer = vec![0; length];
        let read_bytes = file.read(&mut buffer).map_err(|e| axum::Error::new(e))?;
        buffer.truncate(read_bytes);
        if read_bytes > 0 {
            yield Bytes::from(buffer);
        } else {
            break;
        }
    }
    Ok(())
}

#[pin_project]
struct StreamBody<S: Stream<Item = Result<Bytes, axum::Error>> + Send> {
    #[pin]
    stream: S,
}

impl<S: Stream<Item = Result<Bytes, axum::Error>> + Send + 'static> IntoResponse for StreamBody<S> {
    fn into_response(self) -> Response {
        Response::new(axum::body::boxed(self))
    }
}

impl<S: Stream<Item = Result<Bytes, axum::Error>> + Send> HttpBody for StreamBody<S> {
    type Data = Bytes;

    type Error = axum::Error;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        self.project().stream.poll_next(cx)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }
}

async fn fs_resolver(Path(path): Path<String>) -> Response {
    let path = ROOT_PATH.get().unwrap().join(path).unwrap();
    if let Ok(file) = path.open_file() {
        let file = VfsFile(file);
        let length = path
            .metadata()
            .map(|meta| meta.len as usize)
            .unwrap_or(BUFFER_LEN);
        let mime = mime_guess::from_path(path.as_str()).first_or_octet_stream();
        (
            [(CONTENT_TYPE, mime.to_string())],
            StreamBody {
                stream: file_stream(file, length),
            },
        )
            .into_response()
    } else {
        (StatusCode::NOT_FOUND, ()).into_response()
    }
}

async fn resolver<R: Runtime>(app: AppHandle<R>, Path(path): Path<String>) -> Response {
    if let Some(asset) = app.asset_resolver().get(path) {
        ([(CONTENT_TYPE, asset.mime_type)], asset.bytes).into_response()
    } else {
        (StatusCode::NOT_FOUND, ()).into_response()
    }
}

pub fn init<R: Runtime>(listener: TcpListener) -> TauriPlugin<R> {
    Builder::new("asset_resolver")
        .setup(move |app| {
            let app = app.clone();
            tauri::async_runtime::spawn(async {
                let cors = CorsLayer::new().allow_methods(Any).allow_origin(Any);
                let app = Router::new()
                    .route("/fs/*.path", get(fs_resolver))
                    .route("/*path", get(move |path| resolver(app, path)))
                    .layer(cors);
                Server::from_tcp(listener)
                    .unwrap()
                    .serve(app.into_make_service())
                    .await
                    .unwrap()
            });
            Ok(())
        })
        .build()
}

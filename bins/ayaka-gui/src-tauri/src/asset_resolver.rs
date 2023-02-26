use axum::{
    body::{Body, Bytes, StreamBody},
    extract::Path,
    http::{header::CONTENT_TYPE, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router, Server,
};
use ayaka_model::vfs::{error::VfsErrorKind, *};
use std::{
    fmt::Display,
    io::{BorrowedBuf, Read},
    net::TcpListener,
    sync::OnceLock,
};
use stream_future::try_stream;
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Runtime,
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

pub(crate) static ROOT_PATH: OnceLock<VfsPath> = OnceLock::new();

#[derive(Debug)]
struct ResolverError(StatusCode, String);

impl Display for ResolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {}", self.0.as_u16(), self.1)
    }
}

impl std::error::Error for ResolverError {}

impl IntoResponse for ResolverError {
    fn into_response(self) -> Response {
        (self.0, self.1).into_response()
    }
}

impl From<VfsError> for ResolverError {
    fn from(err: VfsError) -> Self {
        let msg = err.to_string();
        let code = match err.kind() {
            VfsErrorKind::IoError(_) | VfsErrorKind::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
            VfsErrorKind::FileNotFound => StatusCode::NOT_FOUND,
            VfsErrorKind::InvalidPath => StatusCode::BAD_REQUEST,
            VfsErrorKind::DirectoryExists | VfsErrorKind::FileExists => StatusCode::CONFLICT,
            VfsErrorKind::NotSupported => StatusCode::NOT_IMPLEMENTED,
        };
        Self(code, msg)
    }
}

impl From<std::io::Error> for ResolverError {
    fn from(err: std::io::Error) -> Self {
        let err: VfsError = err.into();
        Self::from(err)
    }
}

const BUFFER_LEN: usize = 1048576;

fn read_buf_vec(mut file: impl Read, vec: &mut Vec<u8>) -> std::io::Result<usize> {
    let old_len = vec.len();
    let mut read_buf = BorrowedBuf::from(vec.spare_capacity_mut());
    let mut cursor = read_buf.unfilled();
    file.read_buf(cursor.reborrow())?;
    let written = cursor.written();
    unsafe {
        vec.set_len(old_len + written);
    }
    Ok(written)
}

#[try_stream(Bytes)]
fn file_stream(mut file: Box<dyn SeekAndRead + Send>, length: usize) -> std::io::Result<()> {
    let length = length.min(BUFFER_LEN);
    loop {
        let mut buffer = Vec::with_capacity(length);
        let read_bytes = read_buf_vec(&mut file, &mut buffer)?;
        if read_bytes > 0 {
            yield Bytes::from(buffer);
        } else {
            break;
        }
    }
    Ok(())
}

async fn fs_resolver(Path(path): Path<String>) -> Result<impl IntoResponse, ResolverError> {
    let path = ROOT_PATH.get().expect("cannot get ROOT_PATH").join(path)?;
    let file = path.open_file()?;
    let mime = mime_guess::from_path(path.as_str()).first_or_octet_stream();
    let length = path
        .metadata()
        .map(|meta| meta.len as usize)
        .unwrap_or(BUFFER_LEN);
    Ok((
        [(CONTENT_TYPE, mime.to_string())],
        StreamBody::new(file_stream(file, length)),
    ))
}

async fn resolver<R: Runtime>(app: AppHandle<R>, req: Request<Body>) -> impl IntoResponse {
    if let Some(asset) = app.asset_resolver().get(req.uri().path().to_string()) {
        Ok(([(CONTENT_TYPE, asset.mime_type)], asset.bytes))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub fn init<R: Runtime>(listener: TcpListener) -> TauriPlugin<R> {
    Builder::new("asset_resolver")
        .setup(move |app| {
            let app = app.clone();
            tauri::async_runtime::spawn(async {
                let app = Router::new()
                    .route("/fs/*path", get(fs_resolver))
                    .fallback(move |req| resolver(app, req))
                    .layer(
                        TraceLayer::new_for_http()
                            .on_request(())
                            .on_response(())
                            .on_body_chunk(())
                            .on_eos(()),
                    )
                    .layer(CorsLayer::new().allow_methods(Any).allow_origin(Any));
                Server::from_tcp(listener)
                    .expect("cannot create server")
                    .serve(app.into_make_service())
                    .await
                    .expect("cannot serve server")
            });
            Ok(())
        })
        .build()
}

use axum::{
    body::Body,
    extract::Path,
    http::{header::CONTENT_TYPE, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router, Server,
};
use ayaka_model::{
    log,
    vfs::{error::VfsErrorKind, *},
};
use std::{fmt::Display, io::Read, net::TcpListener, sync::OnceLock};
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Runtime,
};
use tower_http::cors::{Any, CorsLayer};

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
        log::error!("{}", self);
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

async fn fs_resolver(Path(path): Path<String>) -> Result<impl IntoResponse, ResolverError> {
    let path = ROOT_PATH.get().unwrap().join(path)?;
    let mut file = path.open_file()?;
    let mime = mime_guess::from_path(path.as_str()).first_or_octet_stream();
    // We choose to read_to_end, because in most release cases, the files should be in a TAR.
    // In that case, we use mmap and the file is simply a byte slice.
    // It is a simple copy from the source to buffer.
    let length = path
        .metadata()
        .map(|meta| meta.len as usize)
        .unwrap_or(65536);
    let mut buffer = Vec::with_capacity(length);
    file.read_to_end(&mut buffer)?;
    Ok(([(CONTENT_TYPE, mime.to_string())], buffer))
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
                let cors = CorsLayer::new().allow_methods(Any).allow_origin(Any);
                let app = Router::new()
                    .route("/fs/*path", get(fs_resolver))
                    .fallback(move |req| resolver(app, req))
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

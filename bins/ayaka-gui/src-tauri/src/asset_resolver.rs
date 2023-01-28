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
use std::{io::Read, net::TcpListener, sync::OnceLock};
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Runtime,
};
use tower_http::cors::{Any, CorsLayer};

pub(crate) static ROOT_PATH: OnceLock<VfsPath> = OnceLock::new();

fn vfs_error_response(err: VfsError) -> (StatusCode, String) {
    let msg = err.to_string();
    let code = match err.kind() {
        VfsErrorKind::IoError(_) | VfsErrorKind::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        VfsErrorKind::FileNotFound => StatusCode::NOT_FOUND,
        VfsErrorKind::InvalidPath => StatusCode::BAD_REQUEST,
        VfsErrorKind::DirectoryExists | VfsErrorKind::FileExists => StatusCode::CONFLICT,
        VfsErrorKind::NotSupported => StatusCode::NOT_IMPLEMENTED,
    };
    (code, msg)
}

async fn fs_resolver(Path(path): Path<String>) -> Response {
    let path = ROOT_PATH.get().unwrap().join(path).unwrap();
    match path.open_file() {
        Ok(mut file) => {
            log::debug!("Get FS {} 200", path.as_str());
            let mime = mime_guess::from_path(path.as_str()).first_or_octet_stream();
            // We choose to read_to_end, because in most release cases, the files should be in a TAR.
            // In that case, we use mmap and the file is simply a byte slice.
            // It is a simple copy from the source to buffer.
            let length = path
                .metadata()
                .map(|meta| meta.len as usize)
                .unwrap_or(65536);
            let mut buffer = Vec::with_capacity(length);
            match file.read_to_end(&mut buffer) {
                Ok(_) => ([(CONTENT_TYPE, mime.to_string())], buffer).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            }
        }
        Err(err) => {
            let (code, msg) = vfs_error_response(err);
            log::debug!("Get FS {} {}", path.as_str(), code.as_u16());
            (code, msg).into_response()
        }
    }
}

async fn resolver<R: Runtime>(app: AppHandle<R>, req: Request<Body>) -> Response {
    if let Some(asset) = app.asset_resolver().get(req.uri().path().to_string()) {
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

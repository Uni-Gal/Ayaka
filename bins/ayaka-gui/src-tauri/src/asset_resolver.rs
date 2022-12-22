use axum::{
    body::{Body, Bytes, StreamBody},
    extract::Path,
    http::{header::CONTENT_TYPE, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router, Server,
};
use ayaka_runtime::{log, vfs::*};
use std::{
    io::{Read, Result},
    net::TcpListener,
    sync::OnceLock,
};
use stream_future::try_stream;
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Runtime,
};
use tower_http::cors::{Any, CorsLayer};

pub(crate) static ROOT_PATH: OnceLock<VfsPath> = OnceLock::new();
const BUFFER_LEN: usize = 65536;

#[try_stream(Bytes)]
fn file_stream(mut file: Box<dyn SeekAndRead + Send>, length: usize) -> Result<()> {
    let length = length.min(BUFFER_LEN);
    loop {
        let mut buffer = vec![0; length];
        let read_bytes = file.read(&mut buffer)?;
        buffer.truncate(read_bytes);
        if read_bytes > 0 {
            yield Bytes::from(buffer);
        } else {
            break;
        }
    }
    Ok(())
}

async fn fs_resolver(Path(path): Path<String>) -> Response {
    let path = ROOT_PATH.get().unwrap().join(path).unwrap();
    if let Ok(file) = path.open_file() {
        log::debug!("Get FS {} 200", path.as_str());
        let length = path
            .metadata()
            .map(|meta| meta.len as usize)
            .unwrap_or(BUFFER_LEN);
        let mime = mime_guess::from_path(path.as_str()).first_or_octet_stream();
        (
            [(CONTENT_TYPE, mime.to_string())],
            StreamBody::new(file_stream(file, length)),
        )
            .into_response()
    } else {
        log::debug!("Get FS {} 404", path.as_str());
        (StatusCode::NOT_FOUND, ()).into_response()
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

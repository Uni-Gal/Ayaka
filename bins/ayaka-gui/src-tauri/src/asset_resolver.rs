use actix_cors::Cors;
use actix_web::{
    http::header::CONTENT_TYPE,
    middleware::Logger,
    web::{self, Bytes},
    App, HttpRequest, HttpResponse, HttpServer, Responder, Scope,
};
use std::{io::Result, net::TcpListener, sync::OnceLock};
use stream_future::try_stream;
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Runtime,
};
use vfs::*;

pub(crate) static ROOT_PATH: OnceLock<VfsPath> = OnceLock::new();
const BUFFER_LEN: usize = 65536;

#[try_stream(Bytes)]
fn file_stream(mut file: Box<dyn SeekAndRead>, length: usize) -> Result<()> {
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

async fn fs_resolver(req: HttpRequest) -> impl Responder {
    let url = req.uri().path().strip_prefix("/fs/").unwrap_or_default();
    let path = ROOT_PATH.get().unwrap().join(url).unwrap();
    if let Ok(file) = path.open_file() {
        let length = path
            .metadata()
            .map(|meta| meta.len as usize)
            .unwrap_or(BUFFER_LEN);
        let mime = mime_guess::from_path(path.as_str()).first_or_octet_stream();
        HttpResponse::Ok()
            .content_type(mime)
            .streaming(file_stream(file, length))
    } else {
        HttpResponse::NotFound().finish()
    }
}

async fn resolver<R: Runtime>(app: AppHandle<R>, req: HttpRequest) -> impl Responder {
    let url = req.uri().path();
    if let Some(asset) = app.asset_resolver().get(url.to_string()) {
        HttpResponse::Ok()
            .append_header((CONTENT_TYPE, asset.mime_type.as_str()))
            .body(asset.bytes)
    } else {
        HttpResponse::NotFound().finish()
    }
}

pub fn init<R: Runtime>(listener: TcpListener) -> TauriPlugin<R> {
    Builder::new("asset_resolver")
        .setup(move |app| {
            let app = app.clone();
            tauri::async_runtime::spawn_blocking(move || {
                actix_web::rt::System::new().block_on(async move {
                    HttpServer::new(move || {
                        let app = app.clone();
                        App::new()
                            .service(Scope::new("/fs").default_service(web::to(fs_resolver)))
                            .default_service(web::to(move |req| resolver(app.clone(), req)))
                            .wrap(Logger::new("\"%r\" %s").log_target(module_path!()))
                            .wrap(Cors::permissive())
                    })
                    .listen(listener)
                    .unwrap()
                    .run()
                    .await
                })
            });
            Ok(())
        })
        .build()
}

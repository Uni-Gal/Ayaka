use actix_cors::Cors;
use actix_files::NamedFile;
use actix_web::{
    http::header::CONTENT_TYPE, middleware::Logger, web, App, HttpRequest, HttpResponse,
    HttpServer, Responder, Scope,
};
use std::{net::TcpListener, path::PathBuf, sync::OnceLock};
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Runtime,
};

pub(crate) static ROOT_PATH: OnceLock<PathBuf> = OnceLock::new();

async fn fs_resolver(req: HttpRequest) -> impl Responder {
    let url = req.uri().path().strip_prefix("/fs/").unwrap_or_default();
    let path = ROOT_PATH.get().unwrap().join(url);
    if let Ok(file) = NamedFile::open_async(&path).await {
        file.into_response(&req)
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

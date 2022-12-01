use ayaka_runtime::log;
use std::{path::PathBuf, sync::OnceLock};
use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};
use tiny_http::{Header, Server};

pub(crate) static ROOT_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn init<R: Runtime>(port: u16) -> TauriPlugin<R> {
    Builder::new("asset_resolver")
        .setup(move |app| {
            let asset_resolver = app.asset_resolver();
            std::thread::spawn(move || {
                let server = Server::http(format!("127.0.0.1:{port}"))
                    .expect("Unable to start local server");
                for req in server.incoming_requests() {
                    let url = req.url();
                    log::warn!("Acquiring {}", url);
                    if url.starts_with("/fs/") {
                        let path = ROOT_PATH
                            .get()
                            .unwrap()
                            .join(url.strip_prefix("/fs/").unwrap());
                        if path.is_file() {
                            let file = std::fs::File::open(&path).unwrap();
                            let mut resp = tiny_http::Response::from_file(file);
                            if let Some(mime) = mime_guess::from_path(path).first() {
                                resp.add_header(
                                    Header::from_bytes("Content-Type", mime.essence_str())
                                        .expect("Unable to convert mime_type to Content-Type"),
                                );
                            }
                            req.respond(resp).expect("Unable to setup response");
                        } else {
                            req.respond(tiny_http::Response::empty(404))
                                .expect("Unable to setup response");
                        };
                    } else if let Some(asset) = asset_resolver.get(url.to_string()) {
                        let mut resp = tiny_http::Response::from_data(asset.bytes);
                        resp.add_header(
                            Header::from_bytes("Content-Type", asset.mime_type)
                                .expect("Unable to convert mime_type to Content-Type"),
                        );
                        req.respond(resp).expect("Unable to setup response");
                    } else {
                        req.respond(tiny_http::Response::empty(404))
                            .expect("Unable to setup response")
                    }
                }
            });
            Ok(())
        })
        .build()
}

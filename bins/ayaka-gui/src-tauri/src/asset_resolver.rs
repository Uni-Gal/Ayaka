use std::{path::PathBuf, sync::OnceLock};

use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};
use tiny_http::{Header, Server};

pub(crate) static ROOT_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn init<R: Runtime>(dev_url: String, port: u16) -> TauriPlugin<R> {
    Builder::new("asset_resolver")
        .setup(move |app| {
            let asset_resolver = app.asset_resolver();
            std::thread::spawn(move || {
                let server = Server::http(format!("127.0.0.1:{port}"))
                    .expect("Unable to start local server");
                for req in server.incoming_requests() {
                    let url = req.url().to_string();
                    if cfg!(debug_assertions) {
                        let _ = asset_resolver;
                    } else if url.starts_with("/assets/")
                        || url == "/"
                        || url == "/live2d.min.js"
                        || url == "/live2dcubismcore.min.js"
                    {
                        let asset = asset_resolver.get(url).unwrap();
                        let mut resp = tiny_http::Response::from_data(asset.bytes);
                        resp.add_header(
                            Header::from_bytes("Content-Type", asset.mime_type)
                                .expect("Unable to convert mime_type to Content-Type"),
                        );
                        req.respond(resp).expect("Unable to setup response");
                        continue;
                    }
                    if url.starts_with("/fs/") {
                        let path = ROOT_PATH
                            .get()
                            .unwrap()
                            .clone()
                            .join(url.strip_prefix("/fs/").unwrap());
                        let file = if path.is_file() || path.is_symlink() {
                            std::fs::File::open(&path).unwrap()
                        } else if path.is_dir() {
                            let mut path = path.clone();
                            path.push("index.html");
                            match std::fs::File::open(path) {
                                Ok(file) => file,
                                Err(_) => {
                                    req.respond(tiny_http::Response::empty(404))
                                        .expect("Unable to setup response");
                                    continue;
                                }
                            }
                        } else {
                            req.respond(tiny_http::Response::empty(404))
                                .expect("Unable to setup response");
                            continue;
                        };
                        let mut resp = tiny_http::Response::from_file(file);
                        if let Some(mime) = mime_guess::from_path(url).first() {
                            resp.add_header(
                                Header::from_bytes("Content-Type", mime.essence_str())
                                    .expect("Unable to convert mime_type to Content-Type"),
                            );
                        }
                        req.respond(resp).expect("Unable to setup response");
                    } else if cfg!(debug_assertions) {
                        let path = if url.ends_with('/') {
                            url + "index.html"
                        } else {
                            url
                        };
                        let resp = minreq::get(dev_url.trim_end_matches('/').to_string() + &path)
                            .send()
                            .expect("Unable to send request");
                        req.respond(tiny_http::Response::new(
                            resp.status_code.into(),
                            resp.headers
                                .iter()
                                .map(|(k, v)| {
                                    Header::from_bytes(k.as_bytes(), v.as_bytes())
                                        .expect("Unable to convert Header")
                                })
                                .collect(),
                            resp.as_bytes(),
                            None,
                            None,
                        ))
                        .expect("Unable to setup response")
                    } else {
                        let _ = dev_url;
                        req.respond(tiny_http::Response::empty(404))
                            .expect("Unable to setup response")
                    }
                }
            });
            Ok(())
        })
        .build()
}

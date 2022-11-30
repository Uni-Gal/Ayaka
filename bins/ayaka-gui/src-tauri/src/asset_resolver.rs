use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};
use tiny_http::{Header, Server};

pub fn init<R: Runtime>(dev_url: String, port: u16) -> TauriPlugin<R> {
    Builder::new("asset_resolver")
        .setup(move |app| {
            let asset_resolver = app.asset_resolver();
            std::thread::spawn(move || {
                let server = Server::http(format!("127.0.0.1:{port}"))
                    .expect("Unable to start local server");
                for req in server.incoming_requests() {
                    #[cfg(debug_assertions)]
                    let _ = asset_resolver;
                    #[cfg(not(debug_assertions))]
                    if req.url().starts_with("/assets/")
                        || req.url() == "/"
                        || req.url() == "/live2d.min.js"
                        || req.url() == "/live2dcubismcore.min.js"
                    {
                        let asset = asset_resolver.get(req.url().into()).unwrap();
                        let mut resp = if let Some(csp) = asset.csp_header {
                            #[cfg(target_os = "linux")]
                            let mut resp = {
                                let html = String::from_utf8_lossy(&asset.bytes);
                                let body = html.replacen(tauri::utils::html::CSP_TOKEN, &csp, 1);
                                tiny_http::Response::from_data(body)
                            };
                            #[cfg(not(target_os = "linux"))]
                            let mut resp = Response::from_data(asset.bytes);
                            resp.add_header(
                                Header::from_bytes("Content-Security-Policy", csp).expect(
                                    "Unable to convert csp_header to Content-Security-Policy",
                                ),
                            );
                            resp
                        } else {
                            tiny_http::Response::from_data(asset.bytes)
                        };
                        resp.add_header(
                            Header::from_bytes("Content-Type", asset.mime_type)
                                .expect("Unable to convert mime_type to Content-Type"),
                        );
                        req.respond(resp).expect("Unable to setup response");
                        continue;
                    }
                    match (req.url(), std::fs::canonicalize(req.url())) {
                        ("/", _) | (_, Err(_)) => {
                            #[cfg(debug_assertions)]
                            {
                                let path = if req.url().ends_with('/') {
                                    req.url().to_string() + "index.html"
                                } else {
                                    req.url().to_string()
                                };
                                let resp =
                                    minreq::get(dev_url.trim_end_matches('/').to_string() + &path)
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
                            }
                            #[cfg(not(debug_assertions))]
                            {
                                let _ = dev_url;
                                req.respond(tiny_http::Response::empty(404))
                                    .expect("Unable to setup response")
                            }
                        }
                        (_, Ok(path)) => {
                            let file = std::fs::File::open(&path).unwrap();
                            let file = if file.metadata().unwrap().is_dir() {
                                let mut path = path.clone();
                                path.push("index.html");
                                std::fs::File::open(path).unwrap()
                            } else {
                                file
                            };
                            let mut resp = tiny_http::Response::from_file(file);
                            if let Some(mime) = mime_guess::from_path(req.url()).first() {
                                resp.add_header(
                                    Header::from_bytes("Content-Type", mime.essence_str())
                                        .expect("Unable to convert mime_type to Content-Type"),
                                );
                            }
                            req.respond(resp).expect("Unable to setup response")
                        }
                    };
                }
            });
            Ok(())
        })
        .build()
}

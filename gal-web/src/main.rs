use actix_web::{get, main, web, App, HttpServer, Responder};
use clap::Parser;
use gal_runtime::{
    anyhow::{anyhow, Result},
    log::info,
    Context, Game,
};
use serde_json::json;
use std::{
    ffi::OsString,
    sync::{Arc, Mutex},
};

#[derive(Debug, Parser)]
#[clap(about, version, author)]
pub struct Options {
    input: OsString,
    #[clap(short, long)]
    port: Option<u16>,
}

#[get("/info")]
async fn hello(data: web::Data<Mutex<Context>>) -> impl Responder {
    info!("GET info");
    let ctx = data.lock().unwrap();
    web::Json(json! ({
        "title": ctx.game.title,
        "author": ctx.game.author,
    }))
}

#[main]
async fn main() -> Result<()> {
    let opts = Options::parse();
    env_logger::try_init()?;
    let game = Arc::new(Game::open(&opts.input)?);
    let ctx_data = web::Data::new(Mutex::new(Context::new(game)?));
    let port = opts
        .port
        .or_else(|| portpicker::pick_unused_port())
        .ok_or_else(|| anyhow!("failed to find unused port"))?;
    let url = format!("http://127.0.0.1:{}/", port);
    info!("Listening {}", url);
    webbrowser::open(&url)?;
    HttpServer::new(move || {
        App::new()
            .service(web::scope("/api").app_data(ctx_data.clone()).service(hello))
            .service(actix_files::Files::new("/", ".").prefer_utf8(true))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await?;
    Ok(())
}

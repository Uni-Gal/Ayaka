use actix_web::{get, main, web, App, HttpServer, Responder};
use clap::Parser;
use gal_runtime::{
    anyhow::{anyhow, Result},
    log::info,
    Context, Game,
};
use serde_json::json;
use std::{ffi::OsString, path::PathBuf, sync::Mutex};

#[derive(Debug, Parser)]
#[clap(about, version, author)]
pub struct Options {
    input: OsString,
    #[clap(long)]
    dist: OsString,
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
    let (game, runtime) = Game::open(&opts.input)?;
    let ctx_data = web::Data::new(Mutex::new(Context::new(game, runtime)?));
    let port = opts
        .port
        .or_else(|| portpicker::pick_unused_port())
        .ok_or_else(|| anyhow!("failed to find unused port"))?;
    let url = format!("http://127.0.0.1:{}/", port);
    info!("Listening {}", url);
    webbrowser::open(&url)?;
    HttpServer::new(move || {
        let dist = PathBuf::from(&opts.dist);
        let index = dist.join("index.html");
        App::new()
            .service(web::scope("/api").app_data(ctx_data.clone()).service(hello))
            .service(
                actix_web_lab::web::spa()
                    .static_resources_location(dist.to_string_lossy().into_owned())
                    .index_file(index.to_string_lossy().into_owned())
                    .finish(),
            )
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await?;
    Ok(())
}

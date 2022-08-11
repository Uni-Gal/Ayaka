use clap::Parser;
use gal_runtime::{
    anyhow::{Ok, Result},
    log::LevelFilter,
    Context, FrontendType, LocaleBuf,
};
use std::ffi::OsString;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Parser)]
#[clap(about, version, author)]
pub struct Options {
    input: OsString,
    #[clap(short, long)]
    output: OsString,
    #[clap(short, long)]
    locale: Option<LocaleBuf>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let opts = Options::parse();
    env_logger::Builder::from_default_env()
        .filter_module("wasmer", LevelFilter::Warn)
        .try_init()?;
    let context = Context::open(&opts.input, FrontendType::Text);
    let mut ctx = context.await?;

    let mut output = tokio::fs::File::create(&opts.output).await?;
    output.write_all(b"\\documentclass{ctexart}\n").await?;
    output.write_all(b"\\usepackage{lua-ul}\n").await?;
    output
        .write_all(format!("\\title{{{}}}\n", ctx.game.title).as_bytes())
        .await?;
    output
        .write_all(format!("\\author{{{}}}\n", ctx.game.author).as_bytes())
        .await?;
    output.write_all(b"\\begin{document}\n").await?;

    output.write_all(b"\\maketitle\n").await?;
    output.write_all(b"\\tableofcontents\n").await?;

    ctx.init_new();
    if let Some(loc) = opts.locale {
        ctx.set_locale(loc);
    }
    while let Some(action) = ctx.next_run() {
        if let Some(name) = &action.character {
            output
                .write_all(format!("\\paragraph{{{}}}", name).as_bytes())
                .await?;
        }
        for s in action.line {
            output.write_all(s.as_str().as_bytes()).await?;
        }
        output.write_all(b"\n").await?;
        if !action.switches.is_empty() {
            output.write_all(b"\\begin{itemize}\n").await?;
            for s in action.switches.iter() {
                output.write_all(b"\\item ").await?;
                if s.enabled {
                    output.write_all(s.text.as_bytes()).await?;
                } else {
                    output
                        .write_all(format!("\\strikeThrough{{{}}}", s.text).as_bytes())
                        .await?;
                }
                output.write_all(b"\n").await?;
            }
            output.write_all(b"\\end{itemize}\n").await?;
        }
    }

    output.write_all(b"\\end{document}\n").await?;
    Ok(())
}

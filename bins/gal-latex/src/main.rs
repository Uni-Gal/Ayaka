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
    output.write(b"\\documentclass{ctexart}\n").await?;
    output.write(b"\\usepackage{lua-ul}\n").await?;
    output
        .write(format!("\\title{{{}}}\n", ctx.game.title).as_bytes())
        .await?;
    output
        .write(format!("\\author{{{}}}\n", ctx.game.author).as_bytes())
        .await?;
    output.write(b"\\begin{document}\n").await?;

    output.write(b"\\maketitle\n").await?;
    output.write(b"\\tableofcontents\n").await?;

    ctx.init_new();
    if let Some(loc) = opts.locale {
        ctx.set_locale(loc);
    }
    while let Some(action) = ctx.next_run() {
        if let Some(name) = &action.character {
            output
                .write(format!("\\paragraph{{{}}}", name).as_bytes())
                .await?;
        }
        for s in action.line {
            output.write(s.as_str().as_bytes()).await?;
        }
        output.write(b"\n").await?;
        if !action.switches.is_empty() {
            output.write(b"\\begin{itemize}\n").await?;
            for s in action.switches.iter() {
                output.write(b"\\item ").await?;
                if s.enabled {
                    output.write(s.text.as_bytes()).await?;
                } else {
                    output
                        .write(format!("\\strikeThrough{{{}}}", s.text).as_bytes())
                        .await?;
                }
                output.write(b"\n").await?;
            }
            output.write(b"\\end{itemize}\n").await?;
        }
    }

    output.write(b"\\end{document}\n").await?;
    Ok(())
}

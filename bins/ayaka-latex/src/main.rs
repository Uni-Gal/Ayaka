mod writer;

use ayaka_runtime::{anyhow::Result, log::LevelFilter, Context, FrontendType, Locale};
use clap::Parser;
use std::ffi::OsString;
use writer::LaTeXWriter;

#[derive(Debug, Parser)]
#[clap(about, version, author)]
pub struct Options {
    input: OsString,
    #[clap(short, long)]
    output: OsString,
    #[clap(short, long)]
    locale: Option<Locale>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let opts = Options::parse();
    env_logger::Builder::from_default_env()
        .filter_module("wasmer", LevelFilter::Warn)
        .try_init()?;
    let context = Context::open(&opts.input, FrontendType::Text);
    let mut ctx = context.await?;

    let output = tokio::fs::File::create(&opts.output).await?;
    let mut output = LaTeXWriter::new(output);
    output.command("documentclass", ["ctexart"]).await?;
    output.command("usepackage", ["lua-ul"]).await?;
    output.command("title", [&ctx.game.title]).await?;
    output.command("author", [&ctx.game.author]).await?;
    output
        .environment("document", |output| async move {
            output.command0("maketitle").await?;
            output.command0("tableofcontents").await?;

            ctx.init_new();
            ctx.set_locale(opts.locale.unwrap_or_else(Locale::current));
            while let Some(action) = ctx.next_run() {
                if let Some(name) = &action.character {
                    output.command("paragraph", [name]).await?;
                }
                for s in action.line {
                    output.write(s.as_str()).await?;
                }
                output.write("\n").await?;
                if !action.switches.is_empty() {
                    output
                        .environment("itemize", |output| async move {
                            for s in action.switches.iter() {
                                output.command0("item").await?;
                                if s.enabled {
                                    output.write(&s.text).await?;
                                } else {
                                    output.command("strikeThrough", [&s.text]).await?;
                                }
                                output.write("\n").await?;
                            }
                            Ok(output)
                        })
                        .await?;
                }
            }
            Ok(output)
        })
        .await?;
    Ok(())
}

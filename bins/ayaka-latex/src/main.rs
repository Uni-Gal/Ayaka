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
    let context = Context::open(&opts.input, FrontendType::Latex);
    let mut ctx = context.await?;

    let output = tokio::fs::File::create(&opts.output).await?;
    let mut output = LaTeXWriter::new(output);
    output.command("documentclass", ["ctexart"]).await?;
    output.command("usepackage", ["graphicx"]).await?;
    output.command("usepackage", ["lua-ul"]).await?;
    output.command("usepackage", ["luatexja-ruby"]).await?;
    output.command("usepackage", ["verbatim"]).await?;
    output.command("title", [&ctx.game.title]).await?;
    output.command("author", [&ctx.game.author]).await?;
    output
        .environment("document", |output| async move {
            output.command0("maketitle").await?;
            output.command0("tableofcontents").await?;

            ctx.init_new();
            ctx.set_locale(opts.locale.unwrap_or_else(Locale::current));

            let mut current_para = None;
            let mut current_bg = None;

            while let Some(action) = ctx.next_run() {
                if action.para_title != current_para {
                    current_para = action.para_title.clone();
                    output
                        .command("section", [action.para_title.unwrap_or_default()])
                        .await?;
                }
                let bg = action.props.get("bg");
                if current_bg.as_ref() != bg {
                    current_bg = bg.cloned();
                    if let Some(bg) = &current_bg {
                        output
                            .environment_attr("figure", "!htbp", |output| async move {
                                output.command0("centering").await?;
                                output
                                    .command_attr(
                                        "includegraphics",
                                        "width=1\\linewidth",
                                        [bg.replace('\\', "/")],
                                    )
                                    .await?;
                                Ok(output)
                            })
                            .await?;
                    }
                }
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

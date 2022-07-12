use clap::Parser;
use gal_runtime::{anyhow::Result, tokio_stream::StreamExt, Context, FrontendType, OpenStatus};
use std::ffi::OsString;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Parser)]
#[clap(about, version, author)]
pub struct Options {
    input: OsString,
    #[clap(short, long)]
    output: OsString,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let opts = Options::parse();
    env_logger::try_init()?;
    let open = Context::open(&opts.input, FrontendType::Text);
    tokio::pin!(open);
    while let Some(status) = open.try_next().await? {
        match status {
            OpenStatus::LoadProfile => println!("Loading profile..."),
            OpenStatus::CreateRuntime => println!("Creating runtime..."),
            OpenStatus::LoadPlugin(name, i, len) => {
                println!("Loading plugin \"{}\" ({}/{})", name, i + 1, len)
            }
            OpenStatus::Loaded(mut ctx) => {
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
                while let Some(action) = ctx.next_run() {
                    if let Some(name) = &action.data.character {
                        output
                            .write(format!("\\paragraph{{{}}}", name).as_bytes())
                            .await?;
                    }
                    output.write(action.data.line.as_bytes()).await?;
                    output.write(b"\n").await?;
                    if !action.data.switches.is_empty() {
                        output.write(b"\\begin{itemize}\n").await?;
                        for s in action.data.switches.iter() {
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
            }
        }
    }
    Ok(())
}

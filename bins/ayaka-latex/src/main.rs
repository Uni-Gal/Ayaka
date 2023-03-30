mod writer;

use ayaka_plugin_wasmi::{WasmiLinker, WasmiModule};
use ayaka_runtime::{anyhow::Result, vfs::VfsPath, *};
use clap::Parser;
use flexi_logger::{LogSpecification, Logger};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};
use writer::LaTeXWriter;

#[derive(Debug, Parser)]
#[clap(about, version, author)]
pub struct Options {
    #[clap(required = true)]
    input: Vec<OsString>,
    #[clap(short, long)]
    output: PathBuf,
    #[clap(short, long)]
    locale: Option<Locale>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let opts = Options::parse();
    let opt_root_path = opts.output.parent().expect("cannot get output parent dir");
    let spec = LogSpecification::parse("warn")?;
    let _log_handle = Logger::with(spec)
        .log_to_stdout()
        .set_palette("b1;3;2;4;6".to_string())
        .use_utc()
        .start()?;
    let linker = WasmiLinker::new(())?;
    let context = ContextBuilder::<WasmiModule>::new(FrontendType::Latex, linker)
        .with_paths(&opts.input)?
        .open();
    let mut ctx = context.await?;

    let output = tokio::fs::File::create(&opts.output).await?;
    let mut output = LaTeXWriter::new(output);
    output.command("documentclass", ["ctexart"]).await?;
    output.command("usepackage", ["graphicx"]).await?;
    output.command("usepackage", ["lua-ul"]).await?;
    output.command("usepackage", ["luatexja-ruby"]).await?;
    output.command("usepackage", ["verbatim"]).await?;
    output.command("title", [&ctx.game().config.title]).await?;
    output
        .command("author", [&ctx.game().config.author])
        .await?;
    output
        .environment("document", |output| async move {
            output.command0("maketitle").await?;
            output.command0("tableofcontents").await?;

            ctx.set_start_context();
            let loc = opts.locale.unwrap_or_else(Locale::current);

            let mut current_para = None;
            let mut current_bg = None;

            while let Some(raw_ctx) = ctx.next_run() {
                let action = ctx.get_action(&loc, &raw_ctx)?;
                let para_title = ctx.current_paragraph_title(&loc);
                if para_title != current_para.as_ref() {
                    output
                        .command(
                            "section",
                            [para_title.map(|s| s.as_str()).unwrap_or_default()],
                        )
                        .await?;
                    current_para = para_title.cloned();
                }
                let bg = raw_ctx
                    .locals
                    .get("bg")
                    .map(|value| value.get_str().into_owned());
                if current_bg != bg {
                    current_bg = bg;
                    if let Some(bg) = &current_bg {
                        let bg = bg.strip_prefix('/').unwrap_or(bg);
                        let bg_file = ctx.root_path().join(bg)?;
                        let out_bg = opt_root_path.join(bg);
                        copy_vfs(&bg_file, &out_bg).await?;
                        output
                            .environment_attr("figure", "!htbp", |output| async move {
                                output.command0("centering").await?;
                                output
                                    .command_attr(
                                        "includegraphics",
                                        "width=1\\linewidth",
                                        [out_bg.to_string_lossy().replace('\\', "/")],
                                    )
                                    .await?;
                                Ok(output)
                            })
                            .await?;
                    }
                }
                match action {
                    Action::Empty | Action::Custom(_) => {}
                    Action::Text(action) => {
                        if let Some(name) = &action.character {
                            output.command("paragraph", [name]).await?;
                        }
                        for s in action.text {
                            output.write(s.as_str()).await?;
                        }
                        output.write("\n\n").await?;
                    }
                    Action::Switches(switches) => {
                        output
                            .environment("itemize", |output| async move {
                                for s in switches.iter() {
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
            }
            Ok(output)
        })
        .await?;
    Ok(())
}

async fn copy_vfs(source: &VfsPath, target: &Path) -> Result<()> {
    use tokio::io::AsyncWriteExt;
    let mut source = source.open_file()?;
    tokio::fs::create_dir_all(target.parent().expect("cannot get target parent dir")).await?;
    let mut target = tokio::fs::File::create(target).await?;
    let mut buffer = [0u8; 65536];
    loop {
        let count = source.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        target.write_all(&buffer[..count]).await?;
    }
    Ok(())
}

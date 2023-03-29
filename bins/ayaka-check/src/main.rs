use ayaka_plugin_wasmi::{WasmiLinker, WasmiModule};
use ayaka_runtime::{anyhow::Result, *};
use clap::Parser;
use flexi_logger::{LogSpecification, Logger};
use std::{
    ffi::OsString,
    io::{stdin, stdout, Write},
    pin::pin,
};

#[derive(Debug, Parser)]
#[clap(about, version, author)]
pub struct Options {
    #[clap(required = true)]
    input: Vec<OsString>,
    #[clap(long)]
    auto: bool,
    #[clap(short, long)]
    locale: Option<Locale>,
}

fn read_line() -> Result<String> {
    stdout().flush()?;
    let mut s = String::new();
    stdin().read_line(&mut s)?;
    Ok(s)
}

fn pause(auto: bool) -> Result<()> {
    if auto {
        println!();
    } else {
        read_line()?;
    }
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let opts = Options::parse();
    let spec = LogSpecification::parse("warn,ayaka=debug")?;
    let _log_handle = Logger::with(spec)
        .log_to_stdout()
        .set_palette("b1;3;2;4;6".to_string())
        .use_utc()
        .start()?;
    let linker = WasmiLinker::new(())?;
    let context = ContextBuilder::<WasmiModule>::new(FrontendType::Text, linker)
        .with_paths(&opts.input)?
        .open();
    let mut context = pin!(context);
    while let Some(status) = context.next().await {
        match status {
            OpenStatus::LoadProfile => println!("Loading profile..."),
            OpenStatus::CreateRuntime => println!("Creating runtime..."),
            OpenStatus::LoadPlugin(name, i, len) => {
                println!("Loading plugin {} ({}/{})", name, i + 1, len)
            }
            OpenStatus::GamePlugin => println!("Preprocessing game..."),
            OpenStatus::LoadResource => println!("Loading resources..."),
            OpenStatus::LoadParagraph => println!("Loading paragraphs..."),
        }
    }
    let mut ctx = context.await?;
    ctx.set_start_context();
    let loc = opts.locale.unwrap_or_else(Locale::current);
    while let Some(raw_ctx) = ctx.next_run() {
        let action = ctx.get_action(&loc, &raw_ctx)?;
        match action {
            Action::Empty | Action::Custom(_) => {}
            Action::Text(action) => {
                if let Some(name) = &action.character {
                    print!("_{}_", name);
                }
                for s in &action.text {
                    print!("{}", s.as_str());
                }
                pause(opts.auto)?;
            }
            Action::Switches(switches) => {
                for (i, s) in switches.iter().enumerate() {
                    if s.enabled {
                        print!("\n-{}- {}", i + 1, s.text);
                    } else {
                        print!("\n-x- {}", s.text);
                    }
                }
                println!();
                loop {
                    let s = read_line()?;
                    if let Ok(i) = s.trim().parse::<usize>() {
                        let valid = i > 0 && i <= switches.len() && switches[i - 1].enabled;
                        if valid {
                            ctx.switch(i - 1);
                            break;
                        }
                    }
                    println!("Invalid switch, enter again!");
                }
            }
        }
    }
    Ok(())
}

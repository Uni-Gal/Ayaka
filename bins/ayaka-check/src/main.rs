use ayaka_runtime::{
    anyhow::{bail, Result},
    log::LevelFilter,
    *,
};
use clap::Parser;
use std::{
    ffi::OsString,
    io::{stdin, stdout, Write},
};

#[derive(Debug, Parser)]
#[clap(about, version, author)]
pub struct Options {
    input: OsString,
    #[clap(long)]
    check: bool,
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
    env_logger::Builder::from_default_env()
        .filter_module("wasmer", LevelFilter::Warn)
        .try_init()?;
    let context = Context::open(&opts.input, FrontendType::Text);
    pin_mut!(context);
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
    if opts.check && !ctx.check() {
        bail!("Check failed.");
    }
    ctx.init_new();
    ctx.set_settings(Settings {
        lang: opts.locale.unwrap_or_else(Locale::current),
    });
    while let Some(params) = ctx.next_run() {
        let action = ctx.get_action(params.clone());
        if let Some(name) = &action.character {
            print!("_{}_", name);
        }
        for s in action.line {
            print!("{}", s.as_str());
        }
        if !action.switches.is_empty() {
            for (i, s) in action.switches.iter().enumerate() {
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
                    let valid =
                        i > 0 && i <= action.switches.len() && action.switches[i - 1].enabled;
                    if valid {
                        ctx.call(&params.switches[i - 1].action);
                        break;
                    }
                }
                println!("Invalid switch, enter again!");
            }
        } else {
            pause(opts.auto)?;
        }
    }
    Ok(())
}

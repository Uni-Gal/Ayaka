use clap::Parser;
use gal_runtime::{
    anyhow::{bail, Result},
    Command, Context, Game, Line,
};
use std::{
    ffi::OsString,
    io::{stdin, stdout, Write},
    path::PathBuf,
};

#[derive(Debug, Parser)]
#[clap(about, version, author)]
pub struct Options {
    input: OsString,
    #[clap(long)]
    check: bool,
    #[clap(long)]
    auto: bool,
}

fn open_game(input: &OsString) -> Result<Game> {
    let reader = std::fs::File::open(input)?;
    let mut game: Game = serde_yaml::from_reader(reader)?;
    game.root_path = PathBuf::from(input).parent().unwrap().into();
    Ok(game)
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

fn main() -> Result<()> {
    let opts = Options::parse();
    let game = open_game(&opts.input)?;
    let mut ctx = Context::new(&game)?;
    if opts.check {
        if !ctx.check() {
            bail!("Check failed.");
        }
    }
    while let Some(text) = ctx.next_run() {
        let mut item_index = 0;
        let mut item_actions = vec![];
        for line in text.0 {
            match line {
                Line::Str(s) => print!("{}", s),
                Line::Cmd(c) => match c {
                    Command::Pause => {
                        pause(opts.auto)?;
                    }
                    Command::Par => println!(),
                    Command::Exec(p) => print!("{}", ctx.call(&p).get_str()),
                    Command::Switch {
                        text: stext,
                        action,
                        enabled,
                    } => {
                        let enabled = enabled.map(|p| ctx.call(&p).get_bool()).unwrap_or(true);
                        if enabled {
                            print!("\n-{}- {}", item_index + 1, stext);
                        } else {
                            print!("\n-x- {}", stext);
                        }
                        item_actions.push(action);
                        item_index += 1;
                    }
                },
            }
        }
        if item_index > 0 {
            println!();
            loop {
                let s = read_line()?;
                let i = s.trim().parse::<usize>()?;
                let valid = i > 0 && i <= item_index;
                if valid {
                    ctx.call(&item_actions[i - 1]);
                    break;
                } else {
                    println!("Invalid switch, enter again!");
                }
            }
        } else {
            pause(opts.auto)?;
        }
    }
    Ok(())
}

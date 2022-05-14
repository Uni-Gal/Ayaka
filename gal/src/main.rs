use anyhow::{anyhow, Result};
use clap::Parser;
use gal_runtime::{Command, Context, Game, Line};
use std::{ffi::OsString, io::stdin, path::PathBuf};

#[derive(Debug, Parser)]
#[clap(about, version, author)]
pub struct Options {
    input: OsString,
    #[clap(short, long)]
    check: bool,
}

fn open_game(input: &OsString) -> Result<Game> {
    let reader = std::fs::File::open(input)?;
    let mut game: Game = serde_yaml::from_reader(reader)?;
    game.root_path = PathBuf::from(input).parent().unwrap().into();
    Ok(game)
}

fn main() -> Result<()> {
    let opts = Options::parse();
    let game = open_game(&opts.input)?;
    let mut ctx = Context::new(&game);
    if opts.check {
        if !ctx.check() {
            return Err(anyhow!("Check failed."));
        }
    }
    while let Some(text) = ctx.next_run() {
        let mut item_index = 0;
        let mut item_actions = vec![];
        for line in text.0 {
            match line {
                Line::Str(s) => print!("{}", s),
                Line::Cmd(c) => match c {
                    Command::Pause => println!(),
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
        println!();
        if item_index > 0 {
            loop {
                let mut s = String::default();
                stdin().read_line(&mut s)?;
                let i = s.trim().parse::<usize>()?;
                let valid = i > 0 && i <= item_index;
                if valid {
                    ctx.call(&item_actions[i - 1]);
                    break;
                } else {
                    println!("Invalid switch, enter again!");
                }
            }
        }
    }
    Ok(())
}

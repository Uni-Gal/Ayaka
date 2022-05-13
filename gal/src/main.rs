use gal_runtime::{Command, Context, Game, Line};
use std::{io::stdin, path::PathBuf};

fn main() {
    let filename = std::env::args().nth(1).unwrap();
    let reader = std::fs::File::open(&filename).unwrap();
    let game: Game = serde_yaml::from_reader(reader).unwrap();
    let mut ctx = Context::new(PathBuf::from(filename).parent().unwrap(), &game);
    while let Some(text) = ctx.next() {
        let mut item_index = 0;
        let mut item_actions = vec![];
        for line in text.0 {
            match line {
                Line::Str(s) => print!("{}", s),
                Line::Cmd(c) => match c {
                    Command::Pause => println!(),
                    Command::Exec(p) => print!("{}", ctx.call(&p).get_str()),
                    Command::Switch(stext, action, enabled) => {
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
                stdin().read_line(&mut s).unwrap();
                let i = s.trim().parse::<usize>().unwrap();
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
}

use gal_runtime::{Action, Context, Game};
use std::{io::stdin, path::PathBuf};

fn main() {
    let filename = std::env::args().nth(1).unwrap();
    let reader = std::fs::File::open(&filename).unwrap();
    let game: Game = serde_yaml::from_reader(reader).unwrap();
    let mut ctx = Context::new(PathBuf::from(filename).parent().unwrap(), &game);
    while let Some(e) = ctx.next() {
        match e {
            Action::Text(s) => println!("{}", ctx.call(s).get_str()),
            Action::Switch {
                allow_default,
                items,
                default_action,
            } => {
                println!("Switches:");
                for (i, item) in items.iter().enumerate() {
                    if ctx.call(&item.enabled).get_bool() {
                        println!("-{}- {}", i + 1, item.text);
                    } else {
                        println!("-x- {}", item.text);
                    }
                }
                println!("Pleese choose one:");
                loop {
                    let mut s = String::default();
                    stdin().read_line(&mut s).unwrap();
                    let i = s.trim().parse::<usize>().unwrap();
                    let valid = if *allow_default {
                        i <= items.len()
                    } else {
                        i > 0 && i <= items.len()
                    };
                    if valid {
                        if i == 0 {
                            ctx.call(default_action);
                        } else {
                            ctx.call(&items[i - 1].action);
                        }
                        break;
                    } else {
                        println!("Invalid switch, enter again!");
                    }
                }
            }
        }
    }
}

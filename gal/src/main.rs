use gal_runtime::{Context, Event, Game};
use std::{io::stdin, path::PathBuf};

fn main() {
    let filename = std::env::args().nth(1).unwrap();
    let reader = std::fs::File::open(&filename).unwrap();
    let game: Game = serde_yaml::from_reader(reader).unwrap();
    let mut ctx = Context::new(PathBuf::from(filename).parent().unwrap(), &game);
    while let Some(e) = ctx.next() {
        match e {
            Event::Text(s) => println!("{}", s),
            Event::Switch {
                allow_default,
                items,
            } => {
                println!("Switches:");
                for (i, item) in items.iter().enumerate() {
                    if item.enabled {
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
                    let valid = if allow_default {
                        i <= items.len()
                    } else {
                        i > 0 && i <= items.len()
                    };
                    if valid {
                        ctx.switch(i as i64);
                        break;
                    } else {
                        println!("Invalid switch, enter again!");
                    }
                }
            }
        }
    }
}

use std::io::stdin;

use gal_runtime::{Context, Event, Game};

fn main() {
    let filename = std::env::args().nth(1).unwrap();
    let reader = std::fs::File::open(filename).unwrap();
    let game: Game = serde_yaml::from_reader(reader).unwrap();
    let mut ctx = Context::new(game);
    while let Some(e) = ctx.next() {
        match e {
            Event::Text(s) => println!("{}", s),
            Event::Switch {
                allow_default,
                items,
            } => {
                println!("Switches: {:?}, please choose one:", items);
                let mut s = String::default();
                loop {
                    stdin().read_line(&mut s).unwrap();
                    let i = s.parse::<usize>().unwrap();
                    let valid = if allow_default {
                        i <= items.len()
                    } else {
                        i > 0 && i <= items.len()
                    };
                    if valid {
                        ctx.switch(i as i64);
                    } else {
                        println!("Invalid switch, enter again!");
                    }
                }
            }
        }
    }
}

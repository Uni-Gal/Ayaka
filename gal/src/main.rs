use gal_runtime::{Context, Event, Game};

fn main() {
    let filename = std::env::args().nth(1).unwrap();
    let reader = std::fs::File::open(filename).unwrap();
    let game: Game = serde_yaml::from_reader(reader).unwrap();
    let mut ctx = Context::new(game);
    while let Some(e) = ctx.next() {
        match e {
            Event::Text(s) => println!("{}", s),
            Event::Switch(items) => println!("Switches: {:?}", items),
        }
    }
}

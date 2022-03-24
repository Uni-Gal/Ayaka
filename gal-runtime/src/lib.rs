use gal_primitive::*;
use std::collections::HashMap;

pub mod runner;
pub mod script;

#[derive(Debug)]
pub struct Context<'a> {
    pub game: &'a Game,
    pub ctx: RawContext,
    pub locals: HashMap<String, Value>,
}

impl<'a> Context<'a> {
    pub fn new(game: &'a Game) -> Self {
        Self {
            game,
            ctx: RawContext::default(),
            locals: HashMap::new(),
        }
    }

    pub fn current_paragraph(&self) -> Option<&Paragraph> {
        self.game.find_para(&self.ctx.cur_para)
    }

    pub fn current_act(&self) -> Option<&Action> {
        self.current_paragraph()
            .map(|p| &p.actions[self.ctx.cur_act])
    }
}

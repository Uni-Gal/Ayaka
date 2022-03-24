use crate::script::Evaluable;
use crate::*;

pub struct Runner {
    game: Game,
}

impl Runner {
    pub fn new(game: Game) -> Self {
        Self { game }
    }

    pub fn run(&self) {
        let mut ctx = Context::new(&self.game);
        let mut current_para = self.game.paras.first();
        while let Some(cur_para) = current_para {
            ctx.ctx.cur_para = cur_para.tag.clone();
            for act in &cur_para.actions {
                match act {
                    Action::Text(_) => unimplemented!(),
                    Action::Switch(_) => unimplemented!(),
                }
            }
            current_para = self.game.find_para(&cur_para.next.eval_str(&mut ctx));
        }
    }
}

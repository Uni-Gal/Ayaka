pub mod script;

pub use gal_primitive::{Action, Game, Paragraph, RawContext, Value, VarMap};
use script::Evaluable;

#[derive(Debug)]
pub enum Event {
    Text(String),
    // TODO: enabled
    Switch(Vec<String>),
}

#[derive(Debug, Default)]
pub struct VarTable {
    pub vars: VarMap,
    pub locals: VarMap,
}

impl VarTable {
    pub fn with_vars(vars: VarMap) -> Self {
        Self {
            vars,
            locals: VarMap::default(),
        }
    }
}

#[derive(Debug)]
pub struct Context {
    pub game: Game,
    pub ctx: RawContext,
    pub table: VarTable,
}

impl Context {
    pub fn new(game: Game) -> Self {
        let mut ctx = RawContext::default();
        ctx.cur_para = game
            .paras
            .first()
            .map(|p| p.tag.clone())
            .unwrap_or_default();
        Self::with_context(game, ctx, VarMap::default())
    }

    pub fn with_context(game: Game, ctx: RawContext, vars: VarMap) -> Self {
        Self {
            game,
            ctx,
            table: VarTable::with_vars(vars),
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

impl Iterator for Context {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cur_para) = self.game.find_para(&self.ctx.cur_para) {
            if self.ctx.cur_act < cur_para.actions.len() {
                let res = match &cur_para.actions[self.ctx.cur_act] {
                    Action::Text(s) => Some(Event::Text(s.eval_str(&mut self.table))),
                    Action::Switch(items) => Some(Event::Switch(
                        items.iter().map(|item| item.text.clone()).collect(),
                    )),
                };
                self.ctx.cur_act += 1;
                res
            } else {
                self.ctx.cur_para = cur_para.next.eval_str(&mut self.table);
                self.ctx.cur_act = 0;
                self.next()
            }
        } else {
            None
        }
    }
}

pub mod script;

pub use gal_primitive::{Action, Game, Paragraph, RawContext, Value, VarMap};
use script::Evaluable;

#[derive(Debug)]
pub enum Event {
    Text(String),
    // TODO: enabled
    Switch {
        allow_default: bool,
        items: Vec<SwitchItem>,
    },
}

#[derive(Debug)]
pub struct SwitchItem {
    pub text: String,
    pub enabled: bool,
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
    cur_switch_bind: Option<gal_script::Ref>,
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
            cur_switch_bind: None,
        }
    }

    pub fn current_paragraph(&self) -> Option<&Paragraph> {
        self.game.find_para(&self.ctx.cur_para)
    }

    pub fn current_act(&self) -> Option<&Action> {
        self.current_paragraph()
            .map(|p| &p.actions[self.ctx.cur_act])
    }

    pub fn switch(&mut self, i: i64) {
        use gal_script::Ref;
        match self.cur_switch_bind.as_ref().unwrap() {
            Ref::Ctx(n) => self.table.vars.insert(n.clone(), Value::Num(i)),
            _ => unreachable!(),
        };
    }
}

impl Iterator for Context {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cur_para) = self.game.find_para(&self.ctx.cur_para) {
            if self.ctx.cur_act < cur_para.actions.len() {
                let res = match &cur_para.actions[self.ctx.cur_act] {
                    Action::Text(s) => Some(Event::Text(s.eval_str(&mut self.table))),
                    Action::Switch {
                        bind,
                        allow_default,
                        items,
                    } => {
                        self.cur_switch_bind = gal_script::gal::RefParser::new().parse(bind).ok();
                        Some(Event::Switch {
                            allow_default: *allow_default,
                            items: items
                                .iter()
                                .map(|item| SwitchItem {
                                    text: item.text.clone(),
                                    enabled: item.enabled.eval_bool(&mut self.table),
                                })
                                .collect(),
                        })
                    }
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

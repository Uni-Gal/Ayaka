pub mod plugin;
pub mod script;

pub use gal_primitive::*;

use plugin::*;
use script::*;
use std::{collections::HashMap, path::Path};

type RuntimeMap = HashMap<String, Runtime>;

#[derive(Debug)]
pub enum Event {
    Text(String),
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

pub struct Context<'a> {
    pub game: &'a Game,
    pub ctx: RawContext,
    pub res: VarMap,
    // TODO: it's too ugly
    cur_switch_bind: Option<gal_script::Ref>,
    modules: RuntimeMap,
}

impl<'a> Context<'a> {
    pub fn new(path: impl AsRef<Path>, game: &'a Game) -> Self {
        let mut ctx = RawContext::default();
        ctx.cur_para = game
            .paras
            .first()
            .map(|p| p.tag.clone())
            .unwrap_or_default();
        Self::with_context(path, game, ctx)
    }

    pub fn with_context(path: impl AsRef<Path>, game: &'a Game, ctx: RawContext) -> Self {
        Self {
            game,
            ctx,
            // TODO: load resources
            res: VarMap::default(),
            cur_switch_bind: None,
            modules: load_plugins(&game.plugins, path),
        }
    }

    fn table(&mut self) -> VarTable {
        VarTable::new(&mut self.ctx.locals, &self.res, &self.modules)
    }

    pub fn current_paragraph(&self) -> Option<&'a Paragraph> {
        self.game.find_para(&self.ctx.cur_para)
    }

    pub fn current_act(&self) -> Option<&'a Action> {
        self.current_paragraph()
            .and_then(|p| p.actions.get(self.ctx.cur_act))
    }

    pub fn switch(&mut self, i: i64) {
        use gal_script::Ref;
        match self.cur_switch_bind.as_ref().unwrap() {
            Ref::Ctx(n) => self.ctx.locals.insert(n.clone(), RawValue::Num(i)),
            _ => unreachable!(),
        };
    }
}

impl Iterator for Context<'_> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cur_para) = self.current_paragraph() {
            if let Some(act) = self.current_act() {
                let res = match act {
                    Action::Text(s) => Some(Event::Text(self.table().call(s).get_str().into())),
                    Action::Switch {
                        bind,
                        allow_default,
                        items,
                    } => {
                        self.cur_switch_bind = gal_script::RefParser::new().parse(bind).ok();
                        Some(Event::Switch {
                            allow_default: *allow_default,
                            items: items
                                .iter()
                                .map(|item| SwitchItem {
                                    text: item.text.clone(),
                                    enabled: self.table().call(&item.enabled).get_bool(),
                                })
                                .collect(),
                        })
                    }
                };
                self.ctx.cur_act += 1;
                res
            } else {
                self.ctx.cur_para = self.table().call(&cur_para.next).get_str().into();
                self.ctx.cur_act = 0;
                self.next()
            }
        } else {
            None
        }
    }
}

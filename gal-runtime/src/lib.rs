pub mod config;
pub mod plugin;
pub mod script;

pub use config::*;
pub use gal_script::{Expr, RawValue};

use plugin::*;
use script::*;
use std::{collections::HashMap, path::Path};
use wit_bindgen_wasmtime::wasmtime::Store;

pub struct Runtime {
    store: Store<()>,
    modules: HashMap<String, Host>,
}

pub struct Context<'a> {
    pub game: &'a Game,
    pub ctx: RawContext,
    pub res: VarMap,
    runtime: Runtime,
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
        let runtime = load_plugins(&game.plugins, path);
        Self {
            game,
            ctx,
            // TODO: load resources
            res: VarMap::default(),
            runtime,
        }
    }

    fn table(&mut self) -> VarTable {
        VarTable::new(&mut self.ctx.locals, &self.res, &mut self.runtime)
    }

    pub fn current_paragraph(&self) -> Option<&'a Paragraph> {
        self.game.find_para(&self.ctx.cur_para)
    }

    pub fn current_act(&self) -> Option<&'a Action> {
        self.current_paragraph()
            .and_then(|p| p.actions.get(self.ctx.cur_act))
    }

    pub fn call(&mut self, expr: &impl Callable) -> RawValue {
        self.table().call(expr)
    }
}

impl<'a> Iterator for Context<'a> {
    type Item = &'a Action;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cur_para) = self.current_paragraph() {
            if let Some(act) = self.current_act() {
                self.ctx.cur_act += 1;
                Some(act)
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

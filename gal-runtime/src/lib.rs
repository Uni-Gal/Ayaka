pub mod config;
pub mod plugin;
pub mod script;

pub use config::*;
use gal_script::TextParser;
pub use gal_script::{Command, Expr, Line, RawValue, Text};

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

    pub fn current_text(&self) -> Option<&'a String> {
        self.current_paragraph()
            .and_then(|p| p.actions.get(self.ctx.cur_act))
    }

    pub fn call(&mut self, expr: &impl Callable) -> RawValue {
        self.table().call(expr)
    }
}

fn parse_text_rich_error(text: &str) -> Text {
    match TextParser::new(text).parse() {
        Ok(t) => t,
        Err(e) => {
            use std::iter::repeat;
            const FREE_LEN: usize = 20;

            let loc = e.loc();
            let pre_len = loc.0.min(FREE_LEN);
            let post_len = (text.len() - loc.1).min(FREE_LEN);
            eprintln!(
                "Parse error:\n    {}\n    {}\n{}",
                &text[loc.0 - pre_len..loc.1 + post_len],
                repeat(' ')
                    .take(pre_len)
                    .chain(repeat('^').take(loc.1 - loc.0))
                    .collect::<String>(),
                e
            );
            panic!("{}", e);
        }
    }
}

impl<'a> Iterator for Context<'a> {
    type Item = Text;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cur_para) = self.current_paragraph() {
            if let Some(act) = self.current_text() {
                self.ctx.cur_act += 1;
                Some(parse_text_rich_error(act))
            } else {
                self.ctx.cur_para = cur_para
                    .next
                    .as_ref()
                    .map(|next| {
                        parse_text_rich_error(next)
                            .call(&mut self.table())
                            .get_str()
                            .into()
                    })
                    .unwrap_or_default();
                self.ctx.cur_act = 0;
                self.next()
            }
        } else {
            None
        }
    }
}

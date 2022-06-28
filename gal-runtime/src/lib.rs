#![feature(round_char_boundary)]

pub mod config;
pub mod plugin;
pub mod script;

pub use config::*;
pub use gal_locale::Locale;
pub use gal_script::{log, Command, Expr, Line, RawValue, Text};
pub use wit_bindgen_wasmtime::anyhow;

use gal_script::{Loc, ParseError, TextParser};
use log::{error, warn};
use plugin::*;
use script::*;
use std::{collections::HashMap, path::Path};
use unicode_width::UnicodeWidthStr;
use wit_bindgen_wasmtime::wasmtime::Store;

pub struct Runtime {
    store: Store<()>,
    modules: HashMap<String, Host>,
}

pub type LocaleMap = HashMap<String, Locale>;

pub struct Context<'a> {
    pub game: &'a Game,
    pub ctx: RawContext,
    loc: Locale,
    runtime: Runtime,
}

impl<'a> Context<'a> {
    fn default_ctx(game: &Game) -> RawContext {
        let mut ctx = RawContext::default();
        ctx.cur_para = game
            .paras
            .get(&game.base_lang)
            .and_then(|paras| paras.first().map(|p| p.tag.clone()))
            .unwrap_or_else(|| {
                warn!("There is no paragraph in the game.");
                Default::default()
            });
        ctx
    }

    pub fn new(game: &'a Game) -> anyhow::Result<Self> {
        Self::with_context(game, Self::default_ctx(game))
    }

    pub fn with_context(game: &'a Game, ctx: RawContext) -> anyhow::Result<Self> {
        let runtime = load_plugins(&game.plugins, &game.root_path)?;
        Ok(Self {
            game,
            ctx,
            loc: Locale::current(),
            runtime,
        })
    }

    fn table(&mut self) -> VarTable {
        VarTable::new(
            &self.game,
            &self.loc,
            &mut self.ctx.locals,
            &mut self.runtime,
        )
    }

    fn current_paragraph(&self) -> Fallback<'a, Paragraph> {
        self.game.find_para_fallback(&self.loc, &self.ctx.cur_para)
    }

    fn current_text(&self) -> Option<&'a String> {
        self.current_paragraph().and_then(|p| {
            p.texts.get(self.ctx.cur_act).and_then(|s| {
                if s.is_empty() || s == "~" {
                    None
                } else {
                    Some(s)
                }
            })
        })
    }

    pub fn set_locale(&mut self, loc: Locale) {
        self.loc = loc;
    }

    pub fn locale(&self) -> &Locale {
        &self.loc
    }

    pub fn call(&mut self, expr: &impl Callable) -> RawValue {
        self.table().call(expr)
    }

    fn rich_error(&self, text: &str, e: &ParseError) -> String {
        use std::iter::repeat;
        const FREE_LEN: usize = 20;

        let loc = e.loc();
        let loc = Loc(
            text.floor_char_boundary(loc.0),
            text.ceil_char_boundary(loc.1),
        );
        let pre = text.floor_char_boundary(loc.0 - loc.0.min(FREE_LEN));
        let post = text.ceil_char_boundary(loc.1 + (text.len() - loc.1).min(FREE_LEN));

        let para_name = self
            .current_paragraph()
            .and_then(|p| p.title.as_ref())
            .map(|s| s.escape_default().to_string())
            .unwrap_or_default();
        let act_num = self.ctx.cur_act + 1;
        let show_code = &text[pre..post];
        let pre_code = &text[pre..loc.0];
        let error_code = &text[loc.0..loc.1];
        format!(
            "Parse error on paragraph \"{para_name}\", act {act_num}:\n    {show_code}\n    {}\n{e}\n",
            repeat(' ')
                .take(UnicodeWidthStr::width_cjk(pre_code))
                .chain(repeat('^').take(UnicodeWidthStr::width_cjk(error_code)))
                .collect::<String>(),
        )
    }

    // Translate character names
    // TODO: reduce allocation
    fn exact_text(&self, t: Text) -> Text {
        let mut lines = vec![];
        for line in t.0.into_iter() {
            let line = match line {
                Line::Cmd(Command::Character(key, mut alter)) => {
                    if alter.is_empty() {
                        alter = self
                            .game
                            .find_res_fallback(&self.loc)
                            .and_then(|map| map.get(&format!("ch_{}", key)))
                            .map(|v| v.get_str().into_owned())
                            .unwrap_or_default();
                    }
                    Line::Cmd(Command::Character(key, alter))
                }
                _ => line,
            };
            lines.push(line);
        }
        Text(lines)
    }

    fn parse_text_rich_error(&self, text: &str) -> Text {
        match TextParser::new(text).parse() {
            Ok(t) => self.exact_text(t),
            Err(e) => {
                error!("{}", self.rich_error(text, &e));
                Text::default()
            }
        }
    }

    fn check_text_rich_error(&self, text: &str) -> bool {
        if let Err(e) = TextParser::new(text).parse() {
            eprintln!("{}", self.rich_error(text, &e));
            false
        } else {
            true
        }
    }

    pub fn next_run(&mut self) -> Option<Text> {
        let cur_para = self.current_paragraph();
        if cur_para.is_some() {
            if let Some(act) = self.current_text() {
                self.ctx.cur_act += 1;
                Some(self.parse_text_rich_error(act))
            } else {
                self.ctx.cur_para = cur_para
                    .and_then(|p| {
                        p.next.as_ref().map(|next| {
                            self.call(&self.parse_text_rich_error(next))
                                .get_str()
                                .into()
                        })
                    })
                    .unwrap_or_default();
                self.ctx.cur_act = 0;
                self.next_run()
            }
        } else {
            None
        }
    }

    pub fn check(&mut self) -> bool {
        let mut succeed = true;
        for (_, paras) in &self.game.paras {
            for para in paras {
                self.ctx.cur_para = para.tag.clone();
                for (index, act) in para.texts.iter().enumerate() {
                    self.ctx.cur_act = index;
                    succeed &= self.check_text_rich_error(act);
                }
                if let Some(next) = &para.next {
                    succeed &= self.check_text_rich_error(next);
                }
            }
        }
        self.ctx = Self::default_ctx(self.game);
        succeed
    }
}

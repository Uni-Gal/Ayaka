#![feature(absolute_path)]
#![feature(extend_one)]
#![feature(round_char_boundary)]

pub mod config;
pub mod plugin;
pub mod script;

pub use config::*;
pub use gal_locale::Locale;
pub use gal_script::{log, Command, Expr, Line, RawValue, Text};
pub use wit_bindgen_wasmtime::anyhow;

use gal_script::{Loc, ParseError, Program, TextParser};
use log::{error, warn};
use script::*;
use std::{collections::HashMap, path::Path, sync::Arc};
use unicode_width::UnicodeWidthStr;

pub type LocaleMap = HashMap<String, Locale>;

pub struct Context {
    pub game: Arc<Game>,
    pub ctx: RawContext,
    loc: Locale,
}

impl Context {
    fn default_ctx(game: &Game) -> RawContext {
        let mut ctx = RawContext::default();
        ctx.cur_para = game
            .paras()
            .get(&game.base_lang())
            .and_then(|paras| paras.first().map(|p| p.tag.clone()))
            .unwrap_or_else(|| {
                warn!("There is no paragraph in the game.");
                Default::default()
            });
        ctx
    }

    pub fn new(game: Arc<Game>, loc: Locale) -> anyhow::Result<Self> {
        let ctx = Self::default_ctx(&game);
        Self::with_context(game, loc, ctx)
    }

    pub fn with_context(game: Arc<Game>, loc: Locale, ctx: RawContext) -> anyhow::Result<Self> {
        Ok(Self { game, ctx, loc })
    }

    fn table(&mut self) -> VarTable {
        VarTable::new(&self.game, &self.loc, &mut self.ctx.locals)
    }

    fn current_paragraph(&self) -> Fallback<&Paragraph> {
        self.game.find_para_fallback(&self.loc, &self.ctx.cur_para)
    }

    fn current_text(&self) -> Fallback<&String> {
        self.current_paragraph()
            .map(|p| {
                p.texts.get(self.ctx.cur_act).and_then(|s| {
                    if s.is_empty() || s == "~" {
                        None
                    } else {
                        Some(s)
                    }
                })
            })
            .flatten()
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

    fn exact_text(&mut self, t: Text) -> Action {
        let mut lines = String::new();
        let mut chname = None;
        let mut switches = vec![];
        let mut switch_actions = vec![];
        let mut bgm = None;
        for line in t.0.into_iter() {
            match line {
                Line::Str(s) => lines.push_str(&s),
                Line::Cmd(cmd) => match cmd {
                    Command::Par => lines.push('\n'),
                    Command::Character(key, alter) => {
                        chname = if alter.is_empty() {
                            // TODO: reduce allocation
                            let res_key = format!("ch_{}", key);
                            self.game
                                .find_res_fallback(&self.loc)
                                .and_then(|map| map.get(&res_key))
                                .map(|v| v.get_str().into_owned())
                        } else {
                            Some(alter)
                        }
                    }
                    Command::Exec(p) => lines.push_str(&self.call(&p).get_str()),
                    Command::Switch {
                        text,
                        action,
                        enabled,
                    } => {
                        // unwrap: when enabled is None, it means true.
                        let enabled = enabled.map(|p| self.call(&p).get_bool()).unwrap_or(true);
                        switches.push(Switch { text, enabled });
                        switch_actions.push(action);
                    }
                    Command::Bgm(index) => bgm = Some(index),
                },
            }
        }
        let bgm = bgm
            .map(|index| self.game.bgm_dir().join(format!("{}.mp3", index)))
            .map(|path| {
                std::path::absolute(path)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
            });
        Action {
            data: ActionData {
                line: lines,
                character: chname,
                switches,
                bgm,
            },
            switch_actions,
        }
    }

    fn merge_action(&self, actions: Fallback<Action>) -> Option<Action> {
        if actions.is_some() {
            let data = {
                let datas = actions.as_ref().map(|act| &act.data);
                let line = datas
                    .as_ref()
                    .and_then(|data| {
                        if data.line.is_empty() {
                            None
                        } else {
                            Some(&data.line)
                        }
                    })
                    .cloned()
                    .unwrap_or_default();
                let character = datas
                    .as_ref()
                    .and_then(|data| data.character.as_ref())
                    .cloned();
                let switches = datas
                    .as_ref()
                    .map(|data| data.switches.clone())
                    .into_iter()
                    .map(|s| {
                        s.merge(|mut s, bases| {
                            if s.text.is_empty() {
                                s.text = bases.text;
                            }
                            if s.enabled != bases.enabled {
                                s.enabled = bases.enabled;
                            }
                            s
                        })
                    })
                    .collect();
                let bgm = datas.as_ref().and_then(|data| data.bgm.as_ref()).cloned();
                ActionData {
                    line,
                    character,
                    switches,
                    bgm,
                }
            };
            let switch_actions = {
                actions
                    .map(|act| act.switch_actions)
                    .into_iter()
                    .map(|act| act.and_then(|p| if p.0.is_empty() { None } else { Some(p) }))
                    .map(|p| p.unwrap_or(Program(vec![])))
                    .collect()
            };
            Some(Action {
                data,
                switch_actions,
            })
        } else {
            None
        }
    }

    fn parse_text_rich_error(&self, text: &str) -> Text {
        match TextParser::new(text).parse() {
            Ok(t) => t,
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

    pub fn next_run(&mut self) -> Option<Action> {
        let cur_para = self.current_paragraph();
        if cur_para.is_some() {
            let cur_text = self.current_text();
            if cur_text.is_some() {
                let text = cur_text.map(|act| self.parse_text_rich_error(act));
                self.ctx.cur_act += 1;
                let actions = text.map(|t| self.exact_text(t));
                self.merge_action(actions)
            } else {
                self.ctx.cur_para = cur_para
                    .and_then(|p| p.next.as_ref())
                    .map(|next| self.parse_text_rich_error(next))
                    .map(|text| self.call(&text).get_str().into())
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
        for (_, paras) in self.game.paras() {
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
        self.ctx = Self::default_ctx(&self.game);
        succeed
    }
}

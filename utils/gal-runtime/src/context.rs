pub use gal_bindings_types::FrontendType;

use crate::{
    plugin::{LoadStatus, Runtime},
    progress_future::ProgressFuture,
    *,
};
use anyhow::{anyhow, bail, Result};
use gal_bindings_types::{
    ActionLines, ActionProcessContextRef, GameProcessContextRef, TextProcessContextRef,
};
use gal_script::{Command, Line, Loc, ParseError, Program, Text, TextParser};
use log::{error, warn};
use script::*;
use std::{
    collections::HashMap,
    future::Future,
    path::{Path, PathBuf},
};
use tokio_stream::StreamExt;
use unicode_width::UnicodeWidthStr;

/// The game running context.
pub struct Context {
    /// The inner [`Game`] object.
    pub game: Game,
    frontend: FrontendType,
    root_path: PathBuf,
    runtime: Runtime,
    settings: Settings,
    global_record: GlobalRecord,
    /// The inner current record.
    pub ctx: RawContext,
}

/// The open status when creating [`Context`].
#[derive(Debug, Clone)]
pub enum OpenStatus {
    /// Start loading config file.
    LoadProfile,
    /// Start creating plugin runtime.
    CreateRuntime,
    /// Loading the plugin.
    LoadPlugin(
        /// Plugin name.
        String,
        /// Plugin index.
        usize,
        /// Plugin total count.
        usize,
    ),
}

impl Context {
    /// Open a config file with frontend type.
    pub fn open<'a>(
        path: impl AsRef<Path> + 'a,
        frontend: FrontendType,
    ) -> ProgressFuture<impl Future<Output = Result<Self>> + 'a, OpenStatus> {
        ProgressFuture::new(async move |tx| {
            tx.send(OpenStatus::LoadProfile)?;
            let file = tokio::fs::read(&path).await?;
            let mut game: Game = serde_yaml::from_slice(&file)?;
            let root_path = path
                .as_ref()
                .parent()
                .ok_or_else(|| anyhow!("Cannot get parent from input path."))?;
            let root_path = std::path::absolute(root_path)?;
            let runtime = {
                let runtime = Runtime::load(&game.plugins.dir, &root_path, &game.plugins.modules);
                tokio::pin!(runtime);
                while let Some(load_status) = runtime.next().await {
                    match load_status {
                        LoadStatus::CreateEngine => tx.send(OpenStatus::CreateRuntime)?,
                        LoadStatus::LoadPlugin(name, i, len) => {
                            tx.send(OpenStatus::LoadPlugin(name, i, len))?
                        }
                    }
                }
                runtime.await?
            };
            for m in &runtime.game_modules {
                let module = &runtime.modules[m];
                let ctx = GameProcessContextRef {
                    title: &game.title,
                    author: &game.author,
                    root_path: &root_path,
                    props: &game.props,
                };
                let res = module.process_game(ctx)?;
                for (key, value) in res.props {
                    game.props.insert(key, value);
                }
            }
            Ok(Self {
                game,
                frontend,
                root_path,
                runtime,
                settings: Settings::new(),
                global_record: GlobalRecord::default(),
                ctx: RawContext::default(),
            })
        })
    }

    /// Initialize the [`RawContext`] to the start of the game.
    pub fn init_new(&mut self) {
        let mut ctx = RawContext::default();
        ctx.cur_para = self
            .game
            .paras
            .get(&self.game.base_lang)
            .and_then(|paras| paras.first().map(|p| p.tag.clone()))
            .unwrap_or_else(|| {
                warn!("There is no paragraph in the game.");
                Default::default()
            });
        self.init_context(ctx)
    }

    /// Initialize the [`RawContext`] with given record.
    pub fn init_context(&mut self, ctx: RawContext) {
        self.ctx = ctx;
    }

    fn table(&mut self) -> VarTable {
        VarTable::new(
            &self.runtime,
            self.game.find_res_fallback(&self.locale()),
            &mut self.ctx.locals,
        )
    }

    fn current_paragraph(&self) -> Fallback<&Paragraph> {
        self.game
            .find_para_fallback(&self.locale(), &self.ctx.cur_para)
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

    /// Set the current locale.
    pub fn set_locale(&mut self, loc: impl Into<LocaleBuf>) {
        self.settings.lang = loc.into();
    }

    /// Get the current locale.
    pub fn locale(&self) -> &Locale {
        &self.settings.lang
    }

    /// Set all settings.
    pub fn set_settings(&mut self, s: Settings) {
        self.settings = s;
    }

    /// Get all settings.
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Set global record.
    pub fn set_global_record(&mut self, r: GlobalRecord) {
        self.global_record = r;
    }

    /// Get global record.
    pub fn global_record(&self) -> &GlobalRecord {
        &self.global_record
    }

    /// Determine if an [`Action`] has been visited,
    /// by the paragraph tag and action index.
    pub fn visited(&self, para: &str, act: usize) -> bool {
        if let Some(max_act) = self.global_record.record.get(para) {
            log::info!("Test act: {}, max act: {}", act, max_act);
            *max_act >= act
        } else {
            false
        }
    }

    /// Call the part of script with this context.
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

    fn exact_text(&mut self, para_title: Option<String>, t: Text) -> Result<Action> {
        let mut action_line = ActionLines::default();
        let mut chname = None;
        let mut switches = vec![];
        let mut props = HashMap::new();
        let mut switch_actions = vec![];
        for line in t.0.into_iter() {
            match line {
                Line::Str(s) => action_line.push_back_chars(s),
                Line::Cmd(cmd) => match cmd {
                    Command::Character(key, alter) => {
                        chname = if alter.is_empty() {
                            // TODO: reduce allocation
                            let res_key = format!("ch_{}", key);
                            self.game
                                .find_res_fallback(&self.locale())
                                .and_then(|map| map.get(&res_key))
                                .map(|v| v.get_str().into_owned())
                        } else {
                            Some(alter)
                        }
                    }
                    Command::Exec(p) => action_line.push_back_chars(self.call(&p).into_str()),
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
                    Command::Other(name, args) => {
                        if let Some(m) = self.runtime.text_modules.get(&name) {
                            let game_context = TextProcessContextRef {
                                root_path: &self.root_path,
                                game_props: &self.game.props,
                                frontend: self.frontend,
                            };
                            let mut res = self.runtime.modules.get(m).unwrap().dispatch_command(
                                &name,
                                &args,
                                game_context,
                            )?;
                            action_line.append(&mut res.line);
                            for (key, value) in res.props.into_iter() {
                                props.insert(key, value);
                            }
                        } else {
                            bail!("Invalid command {}", name);
                        }
                    }
                },
            }
        }
        Ok(Action {
            cur_para: self.ctx.cur_para.clone(),
            cur_act: self.ctx.cur_act,
            locals: self.ctx.locals.clone(),
            line: action_line,
            character: chname,
            para_title,
            switches,
            props,
            switch_actions,
        })
    }

    fn merge_action(&self, actions: Fallback<Action>) -> Option<Action> {
        if actions.is_some() {
            let actions = actions.spec();

            let cur_para = actions.cur_para.and_any().unwrap_or_default();
            let cur_act = actions.cur_act.fallback().unwrap_or_default();
            let locals = actions.locals.and_any().unwrap_or_default();
            let line = actions.line.and_any().unwrap_or_default();
            let character = actions.character.flatten().and_any();
            let para_title = actions.para_title.flatten().and_any();
            let switches = actions
                .switches
                .into_iter()
                .map(|s| {
                    let s = s.spec();
                    let text = s.text.and_any().unwrap_or_default();
                    let (enabled, base_enabled) = s.enabled.unzip();
                    let enabled = base_enabled.or_else(|| enabled).unwrap_or(true);
                    Switch { text, enabled }
                })
                .collect();
            let (props, base_props) = actions.props.unzip();
            let (mut props, base_props) =
                (props.unwrap_or_default(), base_props.unwrap_or_default());
            for (key, value) in base_props.into_iter() {
                props.entry(key).or_insert(value);
            }
            let switch_actions = actions
                .switch_actions
                .into_iter()
                .map(|act| act.map(|p| p.0).and_any().map(Program))
                .map(|p| p.unwrap_or_default())
                .collect();
            Some(Action {
                cur_para,
                cur_act,
                locals,
                line,
                character,
                para_title,
                switches,
                props,
                switch_actions,
            })
        } else {
            None
        }
    }

    fn process_action(&mut self, mut action: Action) -> Result<Action> {
        let last_action = self.ctx.history.last();
        for action_module in &self.runtime.action_modules {
            let module = &self.runtime.modules[action_module];
            let ctx = ActionProcessContextRef {
                root_path: &self.root_path,
                game_props: &self.game.props,
                frontend: self.frontend,
                last_action,
                action: &action,
            };
            action = module.process_action(ctx)?;
        }
        while let Some(act) = action.line.back() {
            if act.as_str().trim().is_empty() {
                action.line.pop_back();
            } else {
                break;
            }
        }
        while let Some(act) = action.line.front() {
            if act.as_str().trim().is_empty() {
                action.line.pop_front();
            } else {
                break;
            }
        }
        if !action.line.is_empty() || action.character.is_some() {
            self.ctx.history.push(action.clone());
        }
        Ok(action)
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

    /// Step to next line.
    pub fn next_run(&mut self) -> Option<Action> {
        if let Some(action) = self.ctx.history.last() {
            self.global_record
                .record
                .entry(action.cur_para.clone())
                .and_modify(|act| *act = (*act).max(action.cur_act))
                .or_insert(action.cur_act);
        }
        let cur_para = self.current_paragraph();
        if cur_para.is_some() {
            let cur_text = self.current_text();
            if cur_text.is_some() {
                let text = cur_text.map(|act| self.parse_text_rich_error(act));
                let para_title = cur_para.and_then(|p| p.title.as_ref()).cloned();
                let actions = text.map(|t| {
                    self.exact_text(para_title.clone(), t).unwrap_or_else(|e| {
                        error!("Exact text error: {}", e);
                        Action::default()
                    })
                });
                self.ctx.cur_act += 1;
                self.merge_action(actions).map(|act| {
                    self.process_action(act).unwrap_or_else(|e| {
                        error!("Error when processing action: {}", e);
                        Action::default()
                    })
                })
            } else {
                self.ctx.cur_para = cur_para
                    .and_then(|p| p.next.as_ref())
                    .map(|next| self.parse_text_rich_error(next))
                    .map(|text| self.call(&text).into_str())
                    .unwrap_or_default();
                self.ctx.cur_act = 0;
                self.next_run()
            }
        } else {
            None
        }
    }

    /// Step back to the last run.
    pub fn next_back_run(&mut self) -> Option<Action> {
        self.ctx.history.pop();
        if let Some(history_action) = self.ctx.history.last() {
            self.ctx.cur_act = history_action.cur_act;
            self.ctx.cur_para = history_action.cur_para.clone();
            self.ctx.locals = history_action.locals.clone();
            Some(history_action.clone())
        } else {
            None
        }
    }

    /// Check all paragraphs to find grammer errors.
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
        succeed
    }
}

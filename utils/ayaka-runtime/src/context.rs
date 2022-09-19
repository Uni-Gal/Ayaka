use crate::{
    plugin::{LoadStatus, Runtime},
    *,
};
use anyhow::{anyhow, bail, Result};
use ayaka_bindings_types::*;
use ayaka_script::{Loc, ParseError, TextParser};
use ayaka_script_types::{Command, Line, Text};
use fallback::Fallback;
use log::error;
use script::*;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use stream_future::stream;
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
    /// The inner raw context.
    pub ctx: RawContext,
    /// The inner record.
    pub record: ActionRecord,
}

/// The open status when creating [`Context`].
#[derive(Debug, Clone)]
pub enum OpenStatus {
    /// Start loading config file.
    LoadProfile,
    /// Start creating plugin runtime.
    CreateRuntime,
    /// Loading the plugin.
    LoadPlugin(String, usize, usize),
    /// Executing game plugins.
    GamePlugin,
    /// Loading the resources.
    LoadResource,
    /// Loading the paragraphs.
    LoadParagraph,
}

impl Context {
    /// Open a config file with frontend type.
    #[stream(OpenStatus, lifetime = "'a")]
    pub async fn open<'a>(path: impl AsRef<Path> + 'a, frontend: FrontendType) -> Result<Self> {
        yield OpenStatus::LoadProfile;
        let file = std::fs::read(&path)?;
        let mut config: GameConfig = serde_yaml::from_slice(&file)?;
        let root_path = path
            .as_ref()
            .parent()
            .ok_or_else(|| anyhow!("Cannot get parent from input path."))?;
        let root_path = std::path::absolute(root_path)?;
        let runtime = {
            let runtime = Runtime::load(&config.plugins.dir, &root_path, &config.plugins.modules);
            pin_mut!(runtime);
            while let Some(load_status) = runtime.next().await {
                match load_status {
                    LoadStatus::CreateEngine => yield OpenStatus::CreateRuntime,
                    LoadStatus::LoadPlugin(name, i, len) => {
                        yield OpenStatus::LoadPlugin(name, i, len)
                    }
                };
            }
            runtime.await?
        };

        yield OpenStatus::GamePlugin;
        for m in &runtime.game_modules {
            let module = &runtime.modules[m];
            let ctx = GameProcessContextRef {
                title: &config.title,
                author: &config.author,
                root_path: &root_path,
                props: &config.props,
            };
            let res = module.process_game(ctx)?;
            for (key, value) in res.props {
                config.props.insert(key, value);
            }
        }

        yield OpenStatus::LoadResource;
        let mut res = HashMap::new();
        if let Some(res_path) = &config.res {
            let res_path = root_path.join(res_path);
            for rl in std::fs::read_dir(res_path)? {
                let p = rl?.path();
                if p.is_file() && p.extension().unwrap_or_default() == "yaml" {
                    let loc = p
                        .file_stem()
                        .and_then(|s| s.to_string_lossy().parse::<Locale>().ok())
                        .unwrap_or_default();
                    let r = std::fs::read(p)?;
                    let r = serde_yaml::from_slice(&r)?;
                    res.insert(loc, r);
                }
            }
        }

        yield OpenStatus::LoadParagraph;
        let mut paras = HashMap::new();
        let paras_path = root_path.join(&config.paras);
        for pl in std::fs::read_dir(paras_path)? {
            let p = pl?.path();
            if p.is_dir() {
                let loc = p
                    .file_name()
                    .and_then(|s| s.to_string_lossy().parse::<Locale>().ok())
                    .unwrap_or_default();
                let mut paras_map = HashMap::new();
                for p in std::fs::read_dir(p)? {
                    let p = p?.path();
                    if p.is_file() && p.extension().unwrap_or_default() == "yaml" {
                        let key = p
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned();
                        let para = std::fs::read(p)?;
                        let para = serde_yaml::from_slice(&para)?;
                        paras_map.insert(key, para);
                    }
                }
                paras.insert(loc, paras_map);
            }
        }
        Ok(Self {
            game: Game { config, paras, res },
            frontend,
            root_path,
            runtime,
            settings: Settings::new(),
            global_record: GlobalRecord::default(),
            ctx: RawContext::default(),
            record: ActionRecord::default(),
        })
    }

    /// Initialize the [`RawContext`] to the start of the game.
    pub fn init_new(&mut self) {
        self.init_context(ActionRecord { history: vec![] })
    }

    /// Initialize the [`ActionRecord`] with given record.
    pub fn init_context(&mut self, record: ActionRecord) {
        self.ctx = record.last_ctx_with_game(&self.game);
        log::debug!("Context: {:?}", self.ctx);
        self.record = record;
        if !self.record.history.is_empty() {
            // If the record is not empty,
            // we need to set current context to the next one.
            self.ctx.cur_act += 1;
        }
    }

    fn table(&mut self) -> VarTable {
        VarTable::new(
            &self.runtime,
            self.game.find_res_fallback(&self.settings.lang),
            &mut self.ctx.locals,
        )
    }

    fn current_paragraph(&self, loc: &Locale) -> Option<&Paragraph> {
        self.game
            .find_para(loc, &self.ctx.cur_base_para, &self.ctx.cur_para)
    }

    fn current_paragraph_fallback(&self) -> Fallback<&Paragraph> {
        self.game.find_para_fallback(
            &self.settings.lang,
            &self.ctx.cur_base_para,
            &self.ctx.cur_para,
        )
    }

    fn current_text(&self, loc: &Locale) -> Option<&String> {
        self.current_paragraph(loc).and_then(|p| {
            p.texts.get(self.ctx.cur_act).and_then(|s| {
                if s.is_empty() || s == "~" {
                    None
                } else {
                    Some(s)
                }
            })
        })
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
    pub fn visited(&self, action: &ActionParams) -> bool {
        if let Some(max_act) = self.global_record.record.get(&action.ctx.cur_para) {
            log::debug!("Test act: {}, max act: {}", action.ctx.cur_act, max_act);
            *max_act >= action.ctx.cur_act
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
            .current_paragraph(&self.game.config.base_lang)
            .and_then(|p| p.title.as_deref())
            .unwrap_or_default()
            .escape_default();
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

    fn parse_action_params(&mut self, t: Text) -> Result<ActionParams> {
        let mut action_line_params = vec![];
        let mut chkey = None;
        let mut switches = vec![];
        let mut props = HashMap::new();
        for line in t.0.into_iter() {
            match line {
                Line::Str(_) => {}
                Line::Cmd(cmd) => match cmd {
                    Command::Character(key, _) => {
                        chkey = Some(key);
                    }
                    Command::Exec(p) => {
                        let param = self.call(&p);
                        action_line_params.push({
                            let mut lines = ActionLines::default();
                            lines.push_back_chars(param.into_str());
                            lines
                        });
                    }
                    Command::Switch {
                        text: _,
                        action,
                        enabled,
                    } => {
                        // unwrap: when enabled is None, it means true.
                        let enabled = enabled.map(|p| self.call(&p).get_bool()).unwrap_or(true);
                        switches.push(SwitchParams { action, enabled });
                    }
                    Command::Other(name, args) => {
                        if let Some(m) = self.runtime.text_modules.get(&name) {
                            let game_context = TextProcessContextRef {
                                root_path: &self.root_path,
                                game_props: &self.game.config.props,
                                frontend: self.frontend,
                            };
                            let res = self.runtime.modules.get(m).unwrap().dispatch_command(
                                &name,
                                &args,
                                game_context,
                            )?;
                            action_line_params.push(res.line);
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
        Ok(ActionParams {
            ctx: self.ctx.clone(),
            line_params: action_line_params,
            ch_key: chkey,
            switches,
            props,
        })
    }

    fn parse_action(&self, t: Text) -> Result<Action> {
        let mut action_line = ActionLines::default();
        let mut action_line_param_index = 0;
        let mut chname = None;
        let mut switches = vec![];
        for line in t.0.into_iter() {
            match line {
                Line::Str(s) => action_line.push_back_chars(s),
                Line::Cmd(cmd) => match cmd {
                    Command::Character(_, alias) => {
                        if !alias.is_empty() && alias != "~" {
                            chname = Some(alias);
                        }
                    }
                    Command::Exec(_) | Command::Other(_, _) => {
                        action_line.push_back_chars(format!("{{{}}}", action_line_param_index));
                        action_line_param_index += 1;
                    }
                    Command::Switch {
                        text,
                        action: _,
                        enabled: _,
                    } => {
                        switches.push(Switch {
                            text,
                            enabled: false,
                        });
                    }
                },
            }
        }
        Ok(Action {
            ctx: RawContext::default(),
            line: action_line,
            ch_key: None,
            character: chname,
            switches,
            props: HashMap::default(),
        })
    }

    fn merge_action(&self, action: Fallback<Action>, mut params: ActionParams) -> Action {
        let mut action = {
            let action = action.spec();

            let line = action.line.and_any().unwrap_or_default();
            let switches = action.switches.and_any().unwrap_or_default();
            Action {
                ctx: params.ctx,
                line,
                ch_key: None,
                character: None,
                switches,
                props: params.props,
            }
        };

        let mut new_line = ActionLines::default();
        for line in action.line {
            use rustc_parse_format::*;

            let parser = Parser::new(line.as_str(), None, None, false, ParseMode::Format);
            for piece in parser {
                match piece {
                    Piece::String(s) => match &line {
                        ActionLine::Block(_) => new_line.push_back_block(s),
                        ActionLine::Chars(_) => new_line.push_back_chars(s),
                    },
                    Piece::NextArgument(arg) => match arg.position {
                        Position::ArgumentImplicitlyIs(i) | Position::ArgumentIs(i) => {
                            new_line.append(&mut params.line_params[i])
                        }
                        _ => unimplemented!(),
                    },
                }
            }
        }
        action.line = new_line;

        action.ch_key = params.ch_key;
        if action.character.is_none() {
            if let Some(key) = &action.ch_key {
                let res_key = format!("ch_{}", key);
                action.character = self
                    .game
                    .find_res_fallback(&self.settings.lang)
                    .and_then(|map| map.get(&res_key))
                    .map(|v| v.get_str().into_owned())
            }
        }

        for (sw, swp) in action.switches.iter_mut().zip(params.switches.iter()) {
            sw.enabled = swp.enabled;
        }

        action
    }

    pub fn get_action(&self, params: ActionParams) -> Action {
        let cur_text = self
            .game
            .find_para_fallback(
                &self.settings.lang,
                &params.ctx.cur_base_para,
                &params.ctx.cur_para,
            )
            .map(|p| {
                p.texts.get(params.ctx.cur_act).and_then(|s| {
                    if s.is_empty() || s == "~" {
                        None
                    } else {
                        Some(s)
                    }
                })
            })
            .flatten();

        let action = cur_text.map(|act| self.parse_text_rich_error(act));

        let action = action.map(|t| {
            self.parse_action(t).unwrap_or_else(|e| {
                error!("Exact text error: {}", e);
                Action::default()
            })
        });

        let act = self.merge_action(action, params);
        self.process_action(act).unwrap_or_else(|e| {
            error!("Error when processing action: {}", e);
            Action::default()
        })
    }

    fn process_action(&self, mut action: Action) -> Result<Action> {
        let last_action = self.record.history.last();
        for action_module in &self.runtime.action_modules {
            let module = &self.runtime.modules[action_module];
            let ctx = ActionProcessContextRef {
                root_path: &self.root_path,
                game_props: &self.game.config.props,
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
        Ok(action)
    }

    fn push_history(&mut self, action: &ActionParams) {
        self.record.history.push(action.clone());
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
    pub fn next_run(&mut self) -> Option<ActionParams> {
        if let Some(action) = self.record.history.last() {
            self.global_record
                .record
                .entry(action.ctx.cur_para.clone())
                .and_modify(|act| *act = (*act).max(action.ctx.cur_act))
                .or_insert(action.ctx.cur_act);
        }
        let cur_text_base = loop {
            let cur_para = self.current_paragraph(&self.game.config.base_lang);
            let cur_text = self.current_text(&self.game.config.base_lang);
            match (cur_para.is_some(), cur_text.is_some()) {
                (true, true) => break cur_text,
                (true, false) => {
                    self.ctx.cur_para = cur_para
                        .and_then(|p| p.next.as_ref())
                        .map(|next| self.parse_text_rich_error(next))
                        .map(|text| self.call(&text).into_str())
                        .unwrap_or_default();
                    self.ctx.cur_act = 0;
                }
                (false, _) => {
                    if self.ctx.cur_base_para == self.ctx.cur_para {
                        if !self.ctx.cur_para.is_empty() {
                            error!(
                                "Cannot find paragraph \"{}\"",
                                self.ctx.cur_para.escape_default()
                            );
                        }
                        return None;
                    } else {
                        self.ctx.cur_base_para = self.ctx.cur_para.clone();
                    }
                }
            }
        };
        let action_base = cur_text_base.map(|act| self.parse_text_rich_error(act));

        let params = action_base.map(|t| {
            self.parse_action_params(t).unwrap_or_else(|e| {
                error!("Parse action params error: {}", e);
                ActionParams::default()
            })
        });
        if let Some(action) = &params {
            self.push_history(action);
        }
        self.ctx.cur_act += 1;
        params
    }

    /// Get (again) then current run.
    pub fn current_run(&self) -> Option<ActionParams> {
        self.record.history.last().cloned()
    }

    /// Step back to the last run.
    pub fn next_back_run(&mut self) -> Option<ActionParams> {
        if self.record.history.len() <= 1 {
            None
        } else {
            if let Some(last_action) = self.record.history.pop() {
                self.ctx = last_action.ctx;
                log::debug!(
                    "Back to para {}, act {}",
                    self.ctx.cur_para,
                    self.ctx.cur_act
                );
            }
            self.current_run()
        }
    }

    /// Get current paragraph title.
    pub fn current_paragraph_title(&self) -> Option<&String> {
        self.current_paragraph_fallback()
            .and_then(|p| p.title.as_ref())
    }

    /// Check all paragraphs to find grammer errors.
    pub fn check(&mut self) -> bool {
        let mut succeed = true;
        for paras in self.game.paras.values() {
            for (base_tag, paras) in paras {
                self.ctx.cur_base_para = base_tag.clone();
                for para in paras.iter() {
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
        }
        succeed
    }
}

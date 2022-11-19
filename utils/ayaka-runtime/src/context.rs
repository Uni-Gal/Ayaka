use crate::{
    plugin::{LoadStatus, Runtime},
    *,
};
use anyhow::{anyhow, bail, Result};
use ayaka_bindings_types::*;
use ayaka_script::{Command, Line, Text};
use fallback::Fallback;
use log::error;
use script::*;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use stream_future::stream;
use trylog::TryLog;

/// The game running context.
pub struct Context {
    /// The inner [`Game`] object.
    pub game: Game,
    frontend: FrontendType,
    root_path: PathBuf,
    runtime: Runtime,
    /// The inner raw context.
    pub ctx: RawContext,
    /// The inner record.
    pub record: ActionRecord,
    switches: Vec<(bool, Program)>,
    vars: VarMap,
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
        for module in runtime.game_modules() {
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
                    if let Some(loc) = p
                        .file_stem()
                        .and_then(|s| s.to_string_lossy().parse::<Locale>().ok())
                    {
                        let r = std::fs::read(p)?;
                        let r = serde_yaml::from_slice(&r)?;
                        res.insert(loc, r);
                    }
                }
            }
        }

        yield OpenStatus::LoadParagraph;
        let mut paras = HashMap::new();
        let paras_path = root_path.join(&config.paras);
        for pl in std::fs::read_dir(paras_path)? {
            let p = pl?.path();
            if p.is_dir() {
                if let Some(loc) = p
                    .file_name()
                    .and_then(|s| s.to_string_lossy().parse::<Locale>().ok())
                {
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
        }
        Ok(Self {
            game: Game { config, paras, res },
            frontend,
            root_path,
            runtime,
            ctx: RawContext::default(),
            record: ActionRecord::default(),
            switches: vec![],
            vars: VarMap::default(),
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
        VarTable::new(&self.runtime, &mut self.ctx.locals)
    }

    fn current_paragraph(&self, loc: &Locale) -> Option<&Paragraph> {
        self.game
            .find_para(loc, &self.ctx.cur_base_para, &self.ctx.cur_para)
    }

    fn current_paragraph_fallback(&self, loc: &Locale) -> Fallback<&Paragraph> {
        self.game
            .find_para_fallback(loc, &self.ctx.cur_base_para, &self.ctx.cur_para)
    }

    fn current_text(&self, loc: &Locale) -> Option<&Line> {
        self.current_paragraph(loc)
            .and_then(|p| p.texts.get(self.ctx.cur_act))
    }

    fn find_res(&self, loc: &Locale, key: &str) -> Option<&RawValue> {
        self.game
            .find_res_fallback(loc)
            .and_then(|map| map.get(key))
    }

    /// Call the part of script with this context.
    pub fn call(&mut self, expr: &impl Callable) -> RawValue {
        self.table().call(expr)
    }

    /// Choose a switch item by index, start by 0.
    pub fn switch(&mut self, i: usize) -> RawValue {
        assert!((0..self.switches.len()).contains(&i));
        assert!(self.switches[i].0);
        self.call(&self.switches[i].1.clone())
    }

    fn parse_text(&self, loc: &Locale, text: &Text) -> Result<ActionText> {
        let mut action = ActionText::default();
        for subtext in &text.0 {
            match subtext {
                SubText::Str(s) => action.push_back_chars(s),
                SubText::Cmd(cmd) => match cmd {
                    Command::Character(key, alias) => {
                        action.ch_key = Some(key.clone());
                        action.character = if alias.is_empty() {
                            self.find_res(loc, &format!("ch_{}", key))
                                .map(|value| value.get_str().into_owned())
                        } else {
                            Some(alias.clone())
                        }
                    }
                    Command::Res(n) => {
                        if let Some(value) = self.find_res(loc, n) {
                            action.push_back_block(value.get_str())
                        }
                    }
                    Command::Ctx(n) => {
                        if let Some(value) = self.ctx.locals.get(n) {
                            action.push_back_block(value.get_str())
                        }
                    }
                    Command::Other(cmd, args) => {
                        if let Some(module) = self.runtime.text_module(cmd) {
                            let ctx = TextProcessContextRef {
                                root_path: &self.root_path,
                                game_props: &self.game.config.props,
                                frontend: self.frontend,
                            };
                            let res = module.dispatch_text(cmd, args, ctx)?;
                            action.text.extend(res.text.text);
                            action.vars.extend(res.text.vars);
                        }
                    }
                },
            }
        }
        Ok(action)
    }

    fn parse_switches(&self, s: &[SwitchItem]) -> Vec<Switch> {
        s.iter()
            .zip(&self.switches)
            .map(|(item, (enabled, _))| Switch {
                text: item.text.clone(),
                enabled: *enabled,
            })
            .collect()
    }

    fn process_line(&mut self, t: Line) -> Result<()> {
        match t {
            Line::Empty | Line::Text(_) => {}
            Line::Exec { exec } => {
                self.call(&exec);
            }
            Line::Switch { switches } => {
                self.switches.clear();
                for item in switches {
                    let enabled = item
                        .enabled
                        .as_ref()
                        .map(|p| self.call(p).get_bool())
                        .unwrap_or(true);
                    self.switches.push((enabled, item.action));
                }
            }
            Line::Custom(props) => {
                self.vars.clear();
                let cmd = props.iter().next().map(|(key, _)| key);
                if let Some(cmd) = cmd {
                    if let Some(module) = self.runtime.line_module(cmd) {
                        let ctx = LineProcessContextRef {
                            root_path: &self.root_path,
                            game_props: &self.game.config.props,
                            frontend: self.frontend,
                            ctx: &self.ctx,
                            props: &props,
                        };
                        let res = module.dispatch_line(cmd, ctx)?;
                        self.ctx.locals.extend(res.locals);
                        self.vars.extend(res.vars);
                    } else {
                        bail!("Cannot find command {}", cmd)
                    }
                }
            }
        }
        Ok(())
    }

    fn merge_action(&self, action: Fallback<Action>) -> Result<Action> {
        match action.unzip() {
            (None, None) => Ok(Action::default()),
            (Some(action), None) | (None, Some(action)) => Ok(action),
            (Some(action), Some(action_base)) => match (action, action_base) {
                (Action::Text(action), Action::Text(action_base)) => {
                    let action = Fallback::new(Some(action), Some(action_base));
                    let action = action.spec();
                    Ok(Action::Text(ActionText {
                        text: action.text.and_any().unwrap_or_default(),
                        ch_key: action.ch_key.flatten().fallback(),
                        character: action.character.flatten().fallback(),
                        vars: action.vars.and_any().unwrap_or_default(),
                    }))
                }
                (Action::Switches(mut switches), Action::Switches(switches_base)) => {
                    for (item, item_base) in switches.iter_mut().zip(switches_base) {
                        item.enabled = item_base.enabled;
                    }
                    Ok(Action::Switches(switches))
                }
                (Action::Custom(mut vars), Action::Custom(vars_base)) => {
                    vars.extend(vars_base);
                    Ok(Action::Custom(vars))
                }
                _ => bail!("Mismatching action type"),
            },
        }
    }

    fn process_action_text(&self, ctx: &RawContext, action: &mut ActionText) -> Result<()> {
        for module in self.runtime.action_modules() {
            let ctx = ActionProcessContextRef {
                root_path: &self.root_path,
                game_props: &self.game.config.props,
                frontend: self.frontend,
                ctx,
                action,
            };
            *action = module.process_action(ctx)?.action;
        }
        while let Some(act) = action.text.back() {
            if act.as_str().trim().is_empty() {
                action.text.pop_back();
            } else {
                break;
            }
        }
        while let Some(act) = action.text.front() {
            if act.as_str().trim().is_empty() {
                action.text.pop_front();
            } else {
                break;
            }
        }
        Ok(())
    }

    /// Get the [`Action`] from [`Locale`] and [`ActionParams`].
    pub fn get_action(&self, loc: &Locale, ctx: &RawContext) -> Result<Action> {
        let cur_text = self
            .game
            .find_para_fallback(loc, &ctx.cur_base_para, &ctx.cur_para)
            .map(|p| p.texts.get(ctx.cur_act))
            .flatten();

        let action = cur_text
            .map(|t| match t {
                Line::Text(t) => self.parse_text(loc, t).map(Action::Text).ok(),
                Line::Switch { switches } => Some(Action::Switches(self.parse_switches(switches))),
                // The real vars will be filled in `merge_action`.
                Line::Custom(_) => Some(Action::Custom(self.vars.clone())),
                _ => None,
            })
            .flatten();

        let mut act = self.merge_action(action)?;
        if let Action::Text(act) = &mut act {
            self.process_action_text(ctx, act)?;
        }
        Ok(act)
    }

    fn push_history(&mut self) {
        let ctx = &self.ctx;
        let cur_text = self
            .game
            .find_para(
                &self.game.config.base_lang,
                &ctx.cur_base_para,
                &ctx.cur_para,
            )
            .and_then(|p| p.texts.get(ctx.cur_act));
        let is_text = cur_text
            .map(|line| matches!(line, Line::Text(_)))
            .unwrap_or_default();
        if is_text {
            self.record.history.push(ctx.clone());
        }
    }

    /// Step to next line.
    pub fn next_run(&mut self) -> Option<RawContext> {
        let cur_text_base = loop {
            let cur_para = self.current_paragraph(&self.game.config.base_lang);
            let cur_text = self.current_text(&self.game.config.base_lang);
            match (cur_para.is_some(), cur_text.is_some()) {
                (true, true) => break cur_text,
                (true, false) => {
                    self.ctx.cur_para = cur_para
                        .and_then(|p| p.next.as_ref())
                        // TODO: reduce clone
                        .cloned()
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

        let ctx = cur_text_base.cloned().map(|t| {
            self.process_line(t)
                .unwrap_or_default_log("Parse line error");
            self.push_history();
            self.ctx.clone()
        });
        self.ctx.cur_act += 1;
        ctx
    }

    /// Step back to the last run.
    pub fn next_back_run(&mut self) -> Option<&RawContext> {
        if self.record.history.len() <= 1 {
            None
        } else {
            if let Some(ctx) = self.record.history.pop() {
                self.ctx = ctx;
                log::debug!(
                    "Back to para {}, act {}",
                    self.ctx.cur_para,
                    self.ctx.cur_act
                );
            }
            self.record.history.last()
        }
    }

    /// Get current paragraph title.
    pub fn current_paragraph_title(&self, loc: &Locale) -> Option<&String> {
        self.current_paragraph_fallback(loc)
            .and_then(|p| p.title.as_ref())
    }
}

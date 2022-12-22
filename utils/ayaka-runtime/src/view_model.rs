use crate::{settings::*, *};
use anyhow::Result;
use serde::Serialize;
use std::path::Path;
use stream_future::stream;
use trylog::macros::*;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "t", content = "data")]
pub enum OpenGameStatus {
    LoadProfile,
    CreateRuntime,
    LoadPlugin(String, usize, usize),
    GamePlugin,
    LoadResource,
    LoadParagraph,
    LoadSettings,
    LoadGlobalRecords,
    LoadRecords,
    Loaded,
}

impl From<OpenStatus> for OpenGameStatus {
    fn from(value: OpenStatus) -> Self {
        match value {
            OpenStatus::LoadProfile => Self::LoadProfile,
            OpenStatus::CreateRuntime => Self::CreateRuntime,
            OpenStatus::LoadPlugin(name, i, len) => Self::LoadPlugin(name, i, len),
            OpenStatus::GamePlugin => Self::GamePlugin,
            OpenStatus::LoadResource => Self::LoadResource,
            OpenStatus::LoadParagraph => Self::LoadParagraph,
        }
    }
}

pub struct GameViewModel<M: SettingsManager> {
    context: Option<Context>,
    current_record: ActionRecord,
    current_raw_context: Option<RawContext>,
    settings_manager: M,
    settings: Option<Settings>,
    records: Vec<ActionRecord>,
    global_record: Option<GlobalRecord>,
}

impl<M: SettingsManager> GameViewModel<M> {
    pub fn new(settings_manager: M) -> Self {
        Self {
            settings_manager,
            context: None,
            current_record: ActionRecord::default(),
            current_raw_context: None,
            settings: None,
            records: vec![],
            global_record: None,
        }
    }

    #[stream(OpenGameStatus, lifetime = "'a")]
    pub async fn open_game<'a>(
        &'a mut self,
        paths: &'a [impl AsRef<Path>],
        frontend_type: FrontendType,
    ) -> Result<()> {
        let context = Context::open(paths, frontend_type);
        pin_mut!(context);
        while let Some(status) = context.next().await {
            yield status.into();
        }
        let context = context.await?;

        let settings = {
            yield OpenGameStatus::LoadSettings;
            unwrap_or_default_log!(
                self.settings_manager.load_settings(),
                "Load settings failed"
            )
        };
        self.settings = Some(settings);

        yield OpenGameStatus::LoadGlobalRecords;
        let global_record = unwrap_or_default_log!(
            self.settings_manager
                .load_global_record(&context.game().config.title),
            "Load global records failed"
        );
        self.global_record = Some(global_record);

        yield OpenGameStatus::LoadRecords;
        self.records = unwrap_or_default_log!(
            self.settings_manager
                .load_records(&context.game().config.title),
            "Load records failed"
        );
        self.context = Some(context);

        yield OpenGameStatus::Loaded;

        Ok(())
    }
}

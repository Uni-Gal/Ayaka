use ayaka_model::{locale, Context, FrontendType, OpenStatus, StreamExt};
use std::pin::Pin;

const CONFIG_PATH: &str = "tests/basic/config.yaml";

#[tokio::test(flavor = "current_thread")]
async fn progress() {
    let mut context = Context::open(&[CONFIG_PATH], FrontendType::Text);
    let progresses = unsafe { Pin::new_unchecked(&mut context) }
        .collect::<Vec<_>>()
        .await;
    context.await.unwrap();
    assert_eq!(
        &progresses,
        &[
            OpenStatus::LoadProfile,
            OpenStatus::CreateRuntime,
            OpenStatus::GamePlugin,
            OpenStatus::LoadResource,
            OpenStatus::LoadParagraph,
        ]
    );
}

#[tokio::test(flavor = "current_thread")]
async fn config() {
    let context = Context::open(&[CONFIG_PATH], FrontendType::Text);
    let context = context.await.unwrap();
    let config = &context.game().config;
    assert_eq!(config.title, "Basic");
    assert_eq!(config.author, "Berrysoft");
    assert_eq!(config.base_lang, locale!("en"));
    assert_eq!(config.paras, "paras");
    assert_eq!(config.start, "init");
}

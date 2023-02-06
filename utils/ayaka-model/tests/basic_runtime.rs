use ayaka_model::{
    locale, Action, ActionText, Context, FrontendType, Locale, OpenStatus, StreamExt,
};
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
    let context = Context::open(&[CONFIG_PATH], FrontendType::Text)
        .await
        .unwrap();
    let config = &context.game().config;
    assert_eq!(config.title, "Basic");
    assert_eq!(config.author, "Berrysoft");
    assert_eq!(config.base_lang, locale!("en"));
    assert_eq!(config.paras, "paras");
    assert_eq!(config.start, "init");
}

fn text_chars(s: impl Into<String>) -> Action {
    let mut text = ActionText::default();
    text.push_back_chars(s.into());
    Action::Text(text)
}

fn paras(mut context: Context, loc: Locale, expected_actions: &[Action]) {
    context.set_start_context();
    let actions = std::iter::from_fn(|| {
        let raw_ctx = context.next_run();
        raw_ctx.map(|raw_ctx| context.get_action(&loc, &raw_ctx).unwrap())
    })
    .collect::<Vec<_>>();
    assert_eq!(&actions, expected_actions);
}

#[tokio::test(flavor = "current_thread")]
async fn paras_en() {
    let context = Context::open(&[CONFIG_PATH], FrontendType::Text)
        .await
        .unwrap();
    let loc = locale!("en");
    paras(
        context,
        loc,
        &[
            text_chars("0"),
            text_chars("1"),
            text_chars("2"),
            text_chars("3"),
        ],
    );
}

#[tokio::test(flavor = "current_thread")]
async fn paras_zh() {
    let context = Context::open(&[CONFIG_PATH], FrontendType::Text)
        .await
        .unwrap();
    let loc = locale!("zh");
    paras(
        context,
        loc,
        &[
            text_chars("0"),
            text_chars("114514"),
            text_chars("2"),
            text_chars("abcdef"),
        ],
    );
}

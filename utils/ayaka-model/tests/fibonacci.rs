use std::collections::HashMap;

use ayaka_model::{locale, Action, ActionText, Context, FrontendType, RawValue};

const CONFIG_PATH: &str = "tests/fibonacci/config.yaml";

fn text_chars(s: impl Into<String>) -> Action {
    let mut text = ActionText::default();
    text.push_back_chars(s.into());
    Action::Text(text)
}

fn custom(c: i64) -> Action {
    Action::Custom(HashMap::from([("c".to_string(), RawValue::Num(c))]))
}

#[tokio::test(flavor = "current_thread")]
async fn calculate() {
    let mut context = Context::open(&[CONFIG_PATH], FrontendType::Text)
        .await
        .unwrap();
    context.set_start_context();
    let loc = locale!("en");
    let actions = std::iter::from_fn(|| {
        let raw_ctx = context.next_run();
        raw_ctx.map(|raw_ctx| context.get_action(&loc, &raw_ctx).unwrap())
    })
    .collect::<Vec<_>>();

    let mut expected_actions = vec![
        text_chars("1"),
        Action::Custom(Default::default()),
        text_chars("1"),
    ];
    let mut a = 1i64;
    let mut b = 1i64;
    for _i in 0..49 {
        let c = b;
        b += a;
        a = c;
        expected_actions.push(custom(c));
        expected_actions.push(text_chars(b.to_string()));
        expected_actions.push(Action::Custom(Default::default()));
    }

    for (i, (left, right)) in actions.into_iter().zip(expected_actions).enumerate() {
        println!("{}", i);
        assert_eq!(left, right);
    }
}

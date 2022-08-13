# Game plugin
Game plugins adjust some properties of the game before any record starts.

## Insert a global property
``` rust,ignore
use gal_bindings::*;

#[export]
fn plugin_type() -> PluginType {
    PluginType::GAME
}

#[export]
fn process_game(mut ctx: GameProcessContext) -> GameProcessResult {
    ctx.props.insert("hello".to_string(), "Hello world!".to_string());
    GameProcessResult { props: ctx.props }
}
```

## Existing plugins
| Plugin  | Description                                   |
| ------- | --------------------------------------------- |
| `media` | Get correct path of background image at home. |

# GAL
This is a project working-in-progress for OSPP 2022.

Gal is a voice novel engine which aims at 3 goals: simple, free, concentrate.

## Simple
Our config file is based on YAML. The structure is simple to author.

### The basic config structure
This is an example of a basic config file:
``` yaml
title: Title
author: Author
base_lang: en
paras:
  en:
    -
      tag: para1
      title: Paragraph 1
      texts:
        - This is the first line.
        - This is the second line.
        - |
          Choose a switch:
          \switch{To paragraph 2.}{$end = false}
          \switch{End.}{$end = true}
          \switch{Not enabled.}{}{false}
      next: \exec{if(!$end, "para2")}
    -
      tag: para2
      title: Paragraph 2
      texts:
        - The first line of the second paragraph.
```

### The TeX commands
The TeX-like commands are embedded in the lines.
Here are the basic commands:
| Command   | Description                 |
| --------- | --------------------------- |
| `\res`    | Get resources.              |
| `\ch`     | Set current character.      |
| `\exec`   | Execute scripts.            |
| `\switch` | Define an item of switches. |

### I18n
The i18n feature are supported by ICU:
| Platform | Library            |
| -------- | ------------------ |
| Windows  | `icu.dll`\*        |
| Linux    | `libicuuc.so`      |
| macOS    | `libicucore.dylib` |

\* Windows 10 1903+

The translation of the texts is always a difficult job. You don't need to copy all commands as is.
For example, the original text (`ja`)
```
\ch{rd}団長！車の用意できました！\bg{0}
\switch{おう！}{$end = false}
\switch{止まるんじゃねぇぞ！}{$end = true}
\switch{止まれ！}{}{false}
```
could be translated as (`zh_Hans`)
```
团长！车已经准备好了！
\switch{哦！}
\switch{不要停下来啊！}
\switch{停下！}
```

## Free
You are free to implement anything you'd like with our script and plugin system.

### Script
The script we use is dynamic typed.
The only supported types are unit `~`, bool, int, and string.
``` rust
pub enum RawValue {
    Unit,
    Bool(bool),
    Num(i64),
    Str(String),
}
```
With the config file, we can even calculate some math problems. For example, Fibonacci:
``` yaml
-
  tag: init
  texts:
    - 1
    - \exec{$n = 50; $a = 1; $b = 1; $i = 1; $b}
  next: loop
-
  tag: loop
  texts:
    - \exec{c = $b; $b += $a; $a = c; $i += 1; $b}
  next: \exec{if($i < $n, "loop")}
```

### Plugin system
The plugins are compiled to `wasm32-wasi` or `wasm32-unknown-unknown`. The plugin runtime is powered by Wasmer.

## Concentrate
The project is separated into 4 parts: runtime, plugins, config, frontend.
You can concentrate at one part without knowing about others.

### GUI frontend
Our GUI frontend is only an example. It uses Tauri with Vue.

# Supported platforms
The project is cross-platform, and the triples are supported with 3 tiers.

## Tier 1
The project is ensured to work well with these triples:

* x86_64-pc-windows-msvc
* x86_64-unknown-linux-gnu
* x86_64-apple-darwin

## Tier 2
The project should work well with these triples, but not tested:

* x86_64-pc-windows-gnu
* aarch64-unknown-linux-gnu
* aarch64-apple-darwin

## Tier 3
The project may not build or run because of dependencies:

* aarch64-pc-windows-msvc

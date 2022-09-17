# File structure

## Directory structure
``` ignore
config.yaml
├─paras
│ ├─ja
│ │ ├─start.yaml
│ │ └─end.yaml
│ └─zh-Hans
│   ├─start.yaml
│   └─end.yaml
└─res
  ├─ja.yaml
  └─zh-Hans.yaml
```

## Properties
The total config file is a `GameConfig` object.
Here shows the properties:

| Property    | Description                          |
| ----------- | ------------------------------------ |
| `title`     | The title of the game.               |
| `base_lang` | The base language.                   |
| `paras`     | The paragraph path.                  |
| `start`     | The start paragraph.                 |
| `author`    | Optional. The author of the game.    |
| `plugins`   | Optional. The `PluginConfig` object. |
| `res`       | Optional. The resource path.         |
| `props`     | Optional. The custom properties.     |

The `PluginConfig` object contains the base directory and the plugin names:

| Property  | Description       |
| --------- | ----------------- |
| `dir`     | The directory.    |
| `modules` | The plugin names. |

A `Paragraph` object is a collection of texts:

| Property | Description                           |
| -------- | ------------------------------------- |
| `tag`    | The tag and key of the paragraph.     |
| `texts`  | The texts.                            |
| `title`  | Optional. The title of the paragraph. |
| `next`   | Optional. The next paragraph.         |

## The visibility of paragraphs
Only the paragraph whose tag is the same as the file name(without extension) is public to all paragraphs.
The rest paragraphs in this file could only be referenced by the paragraphs in the same file.

For example, for `start.yaml`
``` ignore
- tag: start
  next: foo
- tag: foo
  next: bar
- tag: bar
  next: end
```
and `end.yaml`
``` ignore
- tag: end
  next: foo
- tag: foo
  next: bar
- tag: bar
```

The `foo` and `bar` referenced are the ones in the same file, while `start` and `end` could be referenced from other files.

## Basic example
This is a config example, with 2 paragraphs.
``` ignore
config.yaml
└─paras
  └─en
    ├─para1.yaml
    └─para2.yaml
```
`config.yaml`
``` yaml
title: Title
base_lang: en
paras: paras
start: para1
```
`para1.yaml`
``` yaml
- tag: para1
  texts:
    - This is the first line.
    - This is the second line.
  next: para2
```
`para2.yaml`
``` yaml
- tag: para2
  texts:
    - The first line of the second paragraph.
```
The output will be
``` ignore
This is the first line.
This is the second line.
The first line of the second paragraph.
```
You can see that the game starts at the first paragraph `para1`,
and it jumps to `para2` after `para1` ends.
The game exits after `para2` ends, because it doesn't specify the next paragraph.

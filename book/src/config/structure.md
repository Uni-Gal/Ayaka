# File structure

## Properties
The total config file is a `Game` object.
Here shows the properties:

| Property    | Description                                 |
| ----------- | ------------------------------------------- |
| `title`     | The title of the game.                      |
| `base_lang` | The base language.                          |
| `paras`     | The `Paragraph` objects, indexed by locale. |
| `author`    | Optional. The author of the game.           |
| `plugins`   | Optional. The `PluginConfig` object.        |
| `res`       | Optional. The resources, indexed by locale. |
| `props`     | Optional. The custom properties.            |

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

## Basic example
This is a config example, with 2 paragraphs.
``` yaml
title: Title
base_lang: en
paras:
  en:
    -
      tag: para1
      texts:
        - This is the first line.
        - This is the second line.
      next: para2
    -
      tag: para2
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

# Specify character
The dialogues should be marked with the speaking character.
We specify a character with `\ch{}{}` command.

## Add the character names
The character names should be placed in `res` at the front of the game.
Not only because the character name is too long to type again and again, but also to make i18n easy.

The key of the character name should be prefixed with `ch_`:
``` yaml
res:
  en:
    ch_foo: A. Foo
    ch_bar: B. Bar
```
You can then specify the character with the command:
``` yaml
- \ch{foo}This is the first line.
- \ch{bar}This is the second line.
```
These two lines will output as:
``` ignore
_A. Foo_This is the first line.
_B. Bar_This is the second line.
```

## Specify the alias of the character
Sometimes we need a temporary alias of the current character:
``` yaml
- \ch{foo}{Person 1st}This is the first line.
- \ch{bar}{Person 2nd}This is the second line.
```
The output will be
``` ignore
_Person 1st_This is the first line.
_Person 2nd_This is the second line.
```

## Simple grammar
We provide an easier grammar to specify the character:
``` yaml
- /foo//This is the first line.
- /bar/Person 2nd/This is the second line.
```
Note the double slashes in `/foo//` could not be simplified.

## Warning
The `\ch{}{}` command could be called many times, but only the last one affects.

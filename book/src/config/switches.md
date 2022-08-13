# Switches
The switches could be specified with `\switch{}{}{}` command.
The `\switch` command in one line of text are in the same series.

The first parameter is the display text;
the second parameter is the action script;
the third parameter is a boolean expression of whether the switch is enabled.
For the second and the third parameter, see [Script](./script.md).

``` yaml
- |
  Choose a switch:
  \switch{Switch 1}{$s = 1}
  \switch{Switch 2}{$s = 2}
  \switch{Not enabled}{}{false}
```

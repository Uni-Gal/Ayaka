# Switches
The switches could be specified with `switches` command.
The parameters in one line of text are separated by `|`.

The first parameter is the display text;
the third parameter is a boolean expression of whether the switch is enabled.
the second parameter is the action script;
For the second and the third parameter, see [Script](./script.md).

``` yaml
- 'Choose a switch:'
- switches:
  - Switch 1||$s = 1
  - Switch 2||$s = 2
  - Not enabled|false
```

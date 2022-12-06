# Switches
The switches could be specified with `switches` command.

Some special global variables are used.
`$?` is the index of selected switch (start from 0).
`$<num>` indicates whether a switch is enabled, and is cleared after one switch is selected.
If a specific `$<num>` variable is not defined or defined as `null` or `~`, it is treated as `true`.

``` yaml
- 'Choose a switch:'
- exec: $3 = false
- switches:
  - Switch 1
  - Switch 2
  - Not enabled
- You chose switch \var{?}
```

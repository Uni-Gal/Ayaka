# Script
Ayaka script is dynamic typed.
The only supported types are unit `~`, boolean, integer, and string.
``` rust
pub enum RawValue {
    Unit,
    Bool(bool),
    Num(i64),
    Str(String),
}
```

## Using `ayacript`
`ayacript` is the plugin that provides Ayaka script functionalities.
You need to add `ayascript` to the config file. See [Plugin](../plugin/summary.md).

## Execute scripts
Execute a piece of script(we call it *program*) with `exec` command:
``` yaml
- exec: $res = 1 + 1
- 1 + 1 = \var{res}
```
The output is
``` ignore
1 + 1 = 2
```
The script `$res = 1 + 1` is evaluated, and the result is `2`.
It is then converted to string and appended to the text.

## Example: Fibonacci
With the config file, we can even calculate some math problems. For example, Fibonacci:
``` yaml
- tag: init
  texts:
    - '1'
    - exec: $n = 50; $a = 1; $b = 1; $i = 1;
    - \var{b}
  next: loop
- tag: loop
  texts:
    - exec: c = $b; $b += $a; $a = c; $i += 1;
    - \var{b}
    - exec: $next = if($i < $n, "loop")
  next: \var{next}
```


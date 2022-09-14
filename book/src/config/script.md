# Script
The script we use is dynamic typed.
The only supported types are unit `~`, boolean, integer, and string.
``` rust
pub enum RawValue {
    Unit,
    Bool(bool),
    Num(i64),
    Str(String),
}
```
## Execute scripts
Execute a piece of script(we call it *program*) with `\exec{}` command:
``` yaml
- 1 + 1 = \exec{1 + 1}
```
The output is
``` ignore
1 + 1 = 2
```
The script `1 + 1` is evaluated, and the result is `2`.
It is then converted to string and appended to the text.

## Example: Fibonacci
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


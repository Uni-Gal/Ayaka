# Resources
The resources are indexed by locale:
``` ignore
config.yaml
└─res
  ├─en.yaml
  ├─ja.yaml
  └─zh.yaml
```
You can specify other locales, too.
The keys not specified in other locales will fallback to `base_lang` ones.

`en.yaml`
``` yaml
foo: Foo
bar: Bar
```
`zh.yaml`
``` yaml
foo: 天
bar: 地
```

## Reference resources
You can reference resources in texts with `\res{}` command.
``` yaml
- The foo value is \res{foo}
- The bar value is \res{bar}
```

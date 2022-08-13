# Resources
The resources are indexed by locale:
``` yaml
res:
  en:
    foo: Foo value
    bar: Bar value
```
You can specify other locales, too.
The keys not specified in other locales will fallback to `base_lang` ones.
``` yaml
base_lang: en
res:
  en:
    foo: Foo
    bar: Bar
  zh:
    foo: 天
    bar: 地
```

## Reference resources
You can reference resources in texts with `\res{}` command.
``` yaml
- 'The foo value: \res{foo}'
- 'The bar value: \res{bar}'
```

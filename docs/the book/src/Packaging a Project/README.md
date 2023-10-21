> Note: the packaging functionality is currently very minimal.

Current packaging functionality only serves to define what module is the root of the package, this makes it so you can run or build the project without having to specify a path, and makes absolute imports available.

A package requires a manifest file `dragon.yaml`, which uses the [YAML]() markup language.

For now, the manifest only includes a handful of fields, most of which do nothing:
```yaml
name: MyApp
author: Lorenzo Panieri
version: '0.1.0'
entry-point: main.dragon
```

All but `entry-point` don't do anything. Entry-point specifies the file which contains the `main` function, which is also the root of the package. Without an entry point, you must specify which file to run or build explicitly.

The only other reason to have a manifest is that it lets you import modules from elsewhere in your package using absolute paths from the root of the package, instead of relative paths.

This is how you do absolute imports:

```python
import ::foo
```

Without a manifest, each module can only import from external packages or from submodules.

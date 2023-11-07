# Package Manifest File

The manifest should provisionally be in TOML format, until a better configuration format becomes widely adopted.

The manifest file contains defaults for the `dragon build` and `dragon run` commands.

The manifest lets us specify multiple targets.

```toml
# top-level keys are inherited by all targets
version = "0.1.0"
author = "Bahamut"
toolchain = "wasm-0.1.0"

[package.default]
type = "executable"
root = "./main.dragon"

[package.lib]
type = "library"
root = "./main.dragon"

[package.test]
extend = "package.default"
root = "./test.dragon"
# this makes it so we cannot import this package elsewhere, typical of tests
internal = true
```

# Package Structure

A package has a root module, and all other modules are automatically imported.

Packages may be of two types: `library` and `executable`. Executables must have an entry-point, by default this is a `main` function in the top-level module.

Building a library:
```bash
dragon build ./lib.dragon --library
```

Building an executable
```bash
dragon build ./main.dragon --executable
```

It's also possible to run a single file as a script
```bash
dragon run ./main.dragon
```

If we configured a manifest file, we can simply call
```bash
dragon build
dragon run
```

Or we can specify one of the targets
```bash
dragon build --target test
```

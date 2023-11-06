# Hello World

## From the REPL

To start the REPL, just run

```bash
dragon
```

Then paste the following:

```python
print("Hello, World!")

```

You should see this output:

```
Hello, World!
```

## Compiling and Running a File

Make a new file `hello.dragon` and paste in the following code:
```python
print("Hello, World!")
```

Then run it from the terminal:
```bash
dragon run hello.dragon
```

This will compile and run the file, you should see this output:

```
Hello, World!
```

The built executable should be in `build/debug/hello.o` if you're on a UNIX system, or `build/debug/hello.exe` if you are on a Windows system.

If you just want to build, without running, you can write:

```bash
dragon build hello.dragon
```

# String Expressions

## Interpolation
Uses Python-style string interpolation. Unlike Python, it does not allow arbitrary code to be executed inside the string interpolation.

```r
foo := 42
my_string := "my favorite number is {foo}!"
```

Which is just syntax sugar for:
```r
my_string := "my favorite number is " ++ foo.to(str) ++ "!"
```

By default, it uses the generic `to(str)` conversion.

Debug formatting:

```r
"my favorite number is {foo:debug}!"

# becomes
"my favorite number is " ++ foo.debug_format() ++ "!"
```

`debug_format` is part of the `Debug` trait, which is auto-implemented for all types.

Pretty formatting:

```r
"my favorite number is {foo:pretty}!"

# becomes
"my favorite number is " ++ foo.pretty_format() ++ "!"
```

`pretty_format` is part of the `Pretty` trait.

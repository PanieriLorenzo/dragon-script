# Interactive Evaluation

## Line Structure

Dragon uses newline-terminated and semicolon-terminated statements interchangeably, however, there are scenarios where newlines should not be interpreted as statement terminators, such as in the middle of a parenthesised expression.

The REPL should be able to distinguish between when a newline should terminate a top-level statement, and when it should be ignored, allowing to spread an expression across multiple lines before submitting it.

For this purpose, the compiler should be fed one top-level statement at a time, with a pre-processing stage that buffers lines until the top-level statement is terminated.

Every newline creates a *physical line*, wheras each top-level statement creates a *logical line*. The compiler should be fed one full logical line at a time, whereas the REPL should be fed one physical line at a time, providing feedback to the user that they have entered a multi-line block.

```ruby
# these are separate logical lines
foo := 1
bar := 2

# this is a single logical line
{
    foo := 1
    bar := 2
}

# this is a single physical line, but two logical lines
foo := 1; bar := 2
```

## End of Input

`\u0003` *end of text* (^C) and `\u0004` *end of transmission* (^D) characters indicate the end of the program, and the REPL should terminate. Implementations may ask for confirmation or do some other special handling at this stage if necessary.

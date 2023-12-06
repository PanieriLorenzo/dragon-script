# Untyped Dragon

Untyped Dragon is the smallest non-trivial subset. It has only a single type: `int`, so we don't need to explicitly handle any type logic. The syntax is however fully consistent with typed dragon.

Note that untyped in this context doesn't mean *dynamic*. It is statically typed and adheres to all rules of the full type-system, it just has a single type, so it is trivial to implement the type system.

Most omissions then follow naturally, for example, we can't have any expressions that can return `none`, as we don't have `none`, so most control structures are removed.

In order to not be completely trivial, we should be able to do logic, so we make one concession that is not forward compatible: `int` has truthiness, where `0` is false and any other value is true. This will break in the future, as the conversion between `int` and `bool` cannot be implicit.

Another concession is that we can have non-terminating code without having to notate it with the `never` type. This is mostly not an issue, other than requiring some small changes in the backend once `never` is introduced.

Finally we have to allow some built-in functions, even though we do not have function types. These are handled by the compiler as special exceptions. This turns out to be okay, as built-ins are generally handled as exceptional functions in the compiler. Built-in functions that don't normally return an int are assumed to just return 0.

Some other omissions are made, to ensure forward compatibility:
- there is no type inference
- there are no type aliases
- all conditional branches must `yield` an `int`.

And finally, several omissions are made to simplify the language as much as possible, while still being a useful proof-of-concept:
- statements must be terminated with a semicolon
- there is no mutability
- there is no function application syntax
- the only types of statements are declarations and empty statements

Here's an example program:

```rust
foo: int = 42;
bar: int = 128;
_0: int = if foo < bar {
    _0: int = dbg(foo);
    _1: int = yield 0;
} else {
    _0: int = yield 0;
};
```

Here `_0` and `_1` are dummy variables, because we must always assign a value to a variable, and `_1` will actually never be declared, because while `yield` is modelled as returning 0, it actually never returns, as it just exist the block unconditionally.

The `else` branch here cannot be omitted, and it just unconditionally yields 0, which is discarded.

The `dbg` built-in prints a value to the console, using a canonical representation. `dbg` is always defined.

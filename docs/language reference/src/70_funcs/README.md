# Functions

Example:

```r
# declaration
function increment(mut self: int, a: int) -> {
    self += a
}

mut foo := 40

# a regular function call, all parameters passed by value (usually by copy)
bar := increment(foo, 2)
assert!(foo == 40)
assert!(bar == 42)

# an application, the parameters on the left of the dot ar mutated in-place
foo.increment(2)
assert!(foo == 42)
```

## Parameter Passing

Function parameters have two syntaxes: *receivers* and *arguments*. The receivers are mutated in-place instead of being passed by copy, whereas arguments are passed by value. We say that the function is *called* with the arguments, and it is *applied* to the receivers.

The *receivers* appear on the left side of the function call. The *arguments* appear on the right side of the function call.

```r
# a regular call
foo(a, b, c)

# this call can also be written with an explicit receiver list, this makes it
().foo(a, b, c)

# a call with one receiver
(a).foo(b, c)

# the parentheses can be omitted when ther is a single receiver
a.foo(b, c)

# a call with multiple receivers
(a, b).foo(c)
(a, b, c).foo()

# note that it is not allowed to omit the argument list entirely, otherwise there
# would be ambiguity with the syntax for accessing fields.
```

The purpose of this behavior is to allow functions to mutate parameters without introducing first-class references to the language. This forces mutability to always be local, thus making it easier to reason about, both for the programmer and the compiler.

Additionally, application lets us chain function calls, as applying a function to immutable receivers is also perfectly allowed, and behaves exactly the same as calling the function regularly.

```r
# nested function can get hard to read
baz(bar(foo(42)))

# application lets us chain them
42.foo().bar().baz()
```

## Returning Values

## Generics

## Associated Functions

## Side-effects

## Design by Contract

Contracts are predicates that are checked either at run-time or at compile-time. Compile-time contracts must be proven true in order for the compiler to succesfully compile. Run-time contracts cause a panic if they are violated at run-time.

There are two types of contracts that can be added to functions: pre-conditions and post-conditions. Pre-conditions are defined in terms of the parameters of the function, and must hold before the body of the function is called. Post-conditions are defined in terms of the parameters and the outputs of the function and are checked after the function returns, but before the return value is stored or used anywhere. For compile-time assertions, the function is not fully evaluated, and it must be possible to check the post-conditions based on limited knowledge.

```r
@const_pre(x != 0)
function foo(x: int) -> int {
    # .infallible() is like .unwrap() but suggests that the programmer knows
    # it will never fail. In this case, x cannot be 0 because of the pre-condition,
    # so it is perfectly fine. The compiler will raise an error if it cannot prove
    # that `.infallible()` is actually infallible.
    return (2 / x).infallible()
}
```

## Attributes and Decorators

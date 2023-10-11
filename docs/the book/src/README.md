# Introduction

Dragon is a [gradually typed]() scripting language designed for clarity and correctness. It's main goal is to make clear and correct code more accessible, when compared to other "highly correct" languages like Rust. To achieve this I am willing to sacrifice a bit of performance in exchange for usability. That said, I aim to have a decent enough performance to be able to use Dragon for a wide variety of problems and platforms (a performance on par or better than Python).

```python
println("What's your name?")
name := input()
println("Hello, ${name}!")
```

## Why a New Language

Firstly, because I think it's fun. I like having a long-term "backburner" project while I study computer engineering. Secondly, because I think there is a lack of a good language in this space: a high level language, a bit more complex than simple scripting languages like Lua, but less complex than system programming languages like Rust. Existing languages in this space include Python, Java, Kotlin and C#. But with exception for Kotlin, all these languages have significant technical debt, and languages like Java and C# are very verbose and promote a highly abstract (and arguably overengineered) style of programming, that is not very accessible.

## Stand-out Features

- *gradual typing*: the type system is not as strict as a full-fledged static type system, but more strict than a fully dynamic type system. Specifically, the type system guarantees type safety at compile time, while allowing implicit type conversions in cases where this is unambiguous and well-defined. The type system is inspired by that of static type checkers for Python, like [MyPy]().
- *value oriented*: data is always passed by value, there are no pointers or references in the core of the language (although some smart pointers are provided in the standard library). This prevents a large class of errors related to shared mutability. The compiler avoids unnecessary copying of data automatically.
- *mixed paradigm*: Dragon doesn't adhere to any specific paradigm, rather it takes a pragmatic approach, mixing features from various paradigms that have proven themselves to be effective over the decades. Some problems may better be represented by a type-centric OOP approach, whereas other problems are better solved using a more functional style. Similarly, imperative code is quite clear and perfectly fine for writing local internal implementations of functions and types, but a declarative style is better for defining public APIs.
- *traits*: Traits (or interfaces in OOP lingo), are an amazing way to write polymorphic code, as compared to more traditional OOP styles. So I included them.
- *algebraic data types*: ADTs and specifically sum types power the gradual type system and allow to express dynamic data in a type-safe way, by enumerating the allowed types at a specific variable, restricting the functions that can be called on that variable.

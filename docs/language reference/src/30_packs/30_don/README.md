# Dragon Object Notation

<div class="warning">

This feature is just an idea and is not meant to be implemented yet.


</div>

Dragon Object Notation (DON) is a subset of Dragon for serializing data and configuration. It is similar to JSON, but uses Dragon's syntax for objects and structs.

```ruby
# config.schema.don
struct Foo {
    foo: int
    bar: str
    baz: [int]
    qux: obj[str, int]
}
```

```ruby
# config.don
key_1: Foo := {
    foo := 1
    bar := "hello"
    baz := [1, 2, 3]
    qux := {
        "x" := 1
        "y" := 2
        "z" := 3
    }
}
```
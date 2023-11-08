# No Generics

Zig has succeeded in removing generics entirely from the language, everything generics can do are done with `comptime` instead.

```ruby
struct MyList[T] {
    data: [T]
}

function MyList[float]::new() -> Self {
    return {
        data := [float]::empty()
    }
}
```

Would instead become

```ruby
const function MyList(T: type) -> type {
    return struct {
        data: [T]
    }
}

function MyList(float)::new() -> Self {
    return {
        data := [float]::empty()
    }
}
```

Because we don't have generics, how do we restrict a meta-type to have specific traits? For instance, in this function, we would like the type `T` to implement `Numeric`, but the `type` type cannot be restricted, as this would mean that the `type` type itself would need to implement `Numeric`, which doesn't make sense.

```ruby
# this is wrong, because T does not implement Numeric, so it cannot be added
function add(T: type, x: T, y: T) -> T {
    return x + y
}

# this is wrong, because T is not a type, it's a value of the type Numeric
function add(T: Numeric, x: T, y: T) -> T {
    return x + y
}
```

A solution is to introduce an operator to define a meta-type out of a concrete type. Instead of having a type that is "the set of all values implementing `Numeric`", we would be able to construct a type that is "the set of all types whose values implement `Numeric`".

Let's notate this operation like this: `{Numeric}`.

Now, let's compare different parameter bindings to see what the difference is:

- `T: Numeric`: $T\in Numeric$ T is a member of a type, so it's a value.
- `T: type, x: T`: $T\in type, x\in T$ T is a member of a meta-type, so it's a type.
- `T: {Numeric}, x: T`: $T\in\{Numeric\}, x\in T\implies x\in Numeric$ T is a member of a meta-type so it's a type. The meta-type only includes the `Numeric` type, so $x$ is a member of `Numeric`, which is a type, so $x$ is a value.

This is similar to generics, but without breaking the abstractions of this language.
